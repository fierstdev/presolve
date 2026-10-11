use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use presolve_bundle::PresolveBundle;
use presolve_contract::{ApplicationContract, WorkloadKind};
use presolve_core::ArtifactDigest;

use crate::{
    application::{ApplicationDocument, load_application},
    dev::build_rust_component_release,
};

const DEFAULT_OUTPUT_DIRECTORY: &str = "dist";
const PRESOLVED_EXTENSION: &str = "presolved";

#[derive(Debug)]
pub(crate) struct BuildOutput {
    pub(crate) application: String,
    pub(crate) version: String,
    pub(crate) workload: String,
    pub(crate) component: PathBuf,
    pub(crate) release: PathBuf,
    pub(crate) release_digest: ArtifactDigest,
    pub(crate) encoded_size: u64,
}

pub(crate) fn build_release(
    start: &Path,
    component_override: Option<&Path>,
    output_override: Option<&Path>,
) -> Result<BuildOutput> {
    let document = load_application(start)?;
    let application_directory = application_directory(&document)?;
    let workload = single_component_workload_name(&document.contract)?;

    let component = match component_override {
        Some(path) => resolve_input_path(start, path)?,
        None => build_rust_component_release(application_directory)?,
    };

    let component_bytes = fs::read(&component)
        .with_context(|| format!("could not read component `{}`", component.display()))?;

    let bundle = PresolveBundle::from_component(&document.contract, &component_bytes)
        .context("could not construct `.presolved` application release")?;

    let release_digest = bundle
        .release_digest()
        .context("could not compute application release digest")?;

    let encoded = bundle
        .encode()
        .context("could not encode `.presolved` application release")?;

    let decoded =
        PresolveBundle::decode(&encoded).context("encoded `.presolved` release did not verify")?;

    let decoded_digest = decoded
        .release_digest()
        .context("could not compute verified release digest")?;

    if decoded_digest != release_digest {
        bail!(
            "application release identity changed after encode/decode verification: \
             {release_digest} != {decoded_digest}"
        );
    }

    let release = release_output_path(
        start,
        application_directory,
        document.contract.application().name(),
        output_override,
    )?;

    if component == release {
        bail!(
            "release output `{}` would overwrite its component input",
            release.display()
        );
    }

    if let Some(parent) = release.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create output directory `{}`", parent.display()))?;
    }

    fs::write(&release, &encoded)
        .with_context(|| format!("could not write release `{}`", release.display()))?;

    let encoded_size = u64::try_from(encoded.len())
        .context("encoded `.presolved` release is too large to represent")?;

    Ok(BuildOutput {
        application: document.contract.application().name().to_owned(),
        version: document.contract.application().version().to_string(),
        workload,
        component,
        release,
        release_digest,
        encoded_size,
    })
}

fn application_directory(document: &ApplicationDocument) -> Result<&Path> {
    document
        .path
        .parent()
        .context("Application Contract path has no parent directory")
}

fn single_component_workload_name(contract: &ApplicationContract) -> Result<String> {
    match contract.workloads() {
        [] => Ok("main".to_owned()),
        [workload] if workload.kind() == WorkloadKind::Component => Ok(workload.name().to_owned()),
        [workload] => bail!(
            "`presolve build` currently supports automatic packaging for Component workloads; \
             workload `{}` is `{}`",
            workload.name(),
            workload.kind()
        ),
        workloads => bail!(
            "`presolve build` Chunk 19.1 supports one workload; application declares {}",
            workloads.len()
        ),
    }
}

fn resolve_input_path(start: &Path, path: &Path) -> Result<PathBuf> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        start.join(path)
    };

    let metadata = fs::metadata(&resolved)
        .with_context(|| format!("could not inspect component `{}`", resolved.display()))?;

    if !metadata.is_file() {
        bail!("component `{}` is not a regular file", resolved.display());
    }

    Ok(resolved)
}

fn release_output_path(
    start: &Path,
    application_directory: &Path,
    application_name: &str,
    output_override: Option<&Path>,
) -> Result<PathBuf> {
    let output = match output_override {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => start.join(path),
        None => application_directory
            .join(DEFAULT_OUTPUT_DIRECTORY)
            .join(format!("{application_name}.{PRESOLVED_EXTENSION}")),
    };

    if output.extension().and_then(|extension| extension.to_str()) != Some(PRESOLVED_EXTENSION) {
        bail!(
            "release output `{}` must use the `.{PRESOLVED_EXTENSION}` extension",
            output.display()
        );
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should follow Unix epoch")
                .as_nanos();

            let path = env::temp_dir().join(format!(
                "presolve-cli-build-{}-{timestamp}-{sequence}",
                std::process::id()
            ));

            fs::create_dir_all(&path).expect("test directory should be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_contract(directory: &Path, workloads: &str) {
        fs::write(
            directory.join("presolve.toml"),
            format!(
                r#"
contract_version = "0.1"

[application]
name = "orders"
version = "1.2.3"

{workloads}
"#
            ),
        )
        .expect("contract should be written");
    }

    #[test]
    fn implicit_component_build_writes_default_dist_release() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), "");

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component-bytes").expect("component should be written");

        let output = build_release(directory.path(), Some(&component), None)
            .expect("prebuilt component should package");

        assert_eq!(output.application, "orders");
        assert_eq!(output.version, "1.2.3");
        assert_eq!(output.workload, "main");
        assert_eq!(
            output.release,
            directory.path().join("dist").join("orders.presolved")
        );
        assert!(output.release.is_file());

        let encoded = fs::read(&output.release).expect("release should be readable");
        let decoded = PresolveBundle::decode(&encoded).expect("release should verify");

        assert_eq!(decoded.manifest().application().name(), "orders");
        assert_eq!(
            decoded
                .release_digest()
                .expect("release digest should compute"),
            output.release_digest
        );
    }

    #[test]
    fn build_does_not_require_environment_specification() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), "");

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"environment-independent").expect("component should be written");

        assert!(!directory.path().join("presolve.env.toml").exists());

        build_release(directory.path(), Some(&component), None)
            .expect("release construction must not require an environment");
    }

    #[test]
    fn explicit_output_is_resolved_from_invocation_directory() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), "");

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component").expect("component should be written");

        let output = build_release(
            directory.path(),
            Some(&component),
            Some(Path::new("release/orders-custom.presolved")),
        )
        .expect("explicit output should work");

        assert_eq!(
            output.release,
            directory
                .path()
                .join("release")
                .join("orders-custom.presolved")
        );
    }

    #[test]
    fn container_workload_is_rejected_by_initial_component_builder() {
        let contract = presolve_contract::parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "container-app"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "container"
"#,
        )
        .expect("contract should parse");

        let error =
            single_component_workload_name(&contract).expect_err("container build should fail");

        assert!(error.to_string().contains("Component workloads"));
    }

    #[test]
    fn multiple_workloads_are_deferred_beyond_19_1() {
        let contract = presolve_contract::parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "multi"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "component"
"#,
        )
        .expect("contract should parse");

        let error = single_component_workload_name(&contract)
            .expect_err("multi-workload build should fail");

        assert!(error.to_string().contains("supports one workload"));
    }
}
