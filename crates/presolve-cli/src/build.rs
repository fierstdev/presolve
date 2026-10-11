use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use presolve_bundle::{ComponentArtifactInput, PresolveBundle};
use presolve_contract::{ApplicationContract, WorkloadKind};
use presolve_core::ArtifactDigest;

use crate::{
    application::{ApplicationDocument, load_application},
    dev::build_rust_component_release,
};

const DEFAULT_OUTPUT_DIRECTORY: &str = "dist";
const PRESOLVED_EXTENSION: &str = "presolved";

#[derive(Debug)]
pub(crate) struct BuildComponentOutput {
    pub(crate) workload: String,
    pub(crate) component: PathBuf,
}

#[derive(Debug)]
pub(crate) struct BuildOutput {
    pub(crate) application: String,
    pub(crate) version: String,
    pub(crate) components: Vec<BuildComponentOutput>,
    pub(crate) release: PathBuf,
    pub(crate) release_digest: ArtifactDigest,
    pub(crate) encoded_size: u64,
}

#[derive(Debug)]
struct ResolvedComponentArtifact {
    workload: String,
    path: PathBuf,
    bytes: Vec<u8>,
}

pub(crate) fn build_release(
    start: &Path,
    component_override: Option<&Path>,
    component_mappings: &[String],
    output_override: Option<&Path>,
) -> Result<BuildOutput> {
    if component_override.is_some() && !component_mappings.is_empty() {
        bail!("`--component` and `--component-for` cannot be used together");
    }

    let document = load_application(start)?;
    let application_directory = application_directory(&document)?;
    let workloads = component_workload_names(&document.contract)?;

    let components = resolve_component_artifacts(
        start,
        application_directory,
        &workloads,
        component_override,
        component_mappings,
    )?;

    let inputs = components
        .iter()
        .map(|artifact| ComponentArtifactInput::new(&artifact.workload, &artifact.bytes))
        .collect::<Vec<_>>();

    let bundle = PresolveBundle::from_components(&document.contract, &inputs)
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

    if let Some(component) = components
        .iter()
        .find(|component| component.path == release)
    {
        bail!(
            "release output `{}` would overwrite component input for workload `{}`",
            release.display(),
            component.workload
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
        components: components
            .into_iter()
            .map(|component| BuildComponentOutput {
                workload: component.workload,
                component: component.path,
            })
            .collect(),
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

fn component_workload_names(contract: &ApplicationContract) -> Result<Vec<String>> {
    if contract.workloads().is_empty() {
        return Ok(vec!["main".to_owned()]);
    }

    let unsupported = contract
        .workloads()
        .iter()
        .filter(|workload| workload.kind() != WorkloadKind::Component)
        .map(|workload| format!("{} ({})", workload.name(), workload.kind()))
        .collect::<Vec<_>>();

    if !unsupported.is_empty() {
        bail!(
            "`presolve build` Chunk 19.2 accepts Component artifacts only; \
             unsupported workloads: {}",
            unsupported.join(", ")
        );
    }

    Ok(contract
        .workloads()
        .iter()
        .map(|workload| workload.name().to_owned())
        .collect())
}

fn resolve_component_artifacts(
    start: &Path,
    application_directory: &Path,
    workloads: &[String],
    component_override: Option<&Path>,
    component_mappings: &[String],
) -> Result<Vec<ResolvedComponentArtifact>> {
    if let Some(path) = component_override {
        if workloads.len() != 1 {
            bail!(
                "`--component PATH` is the single-workload shorthand; \
                 application declares {} workloads, so use one \
                 `--component-for WORKLOAD=PATH` per workload",
                workloads.len()
            );
        }

        return Ok(vec![read_component_artifact(start, &workloads[0], path)?]);
    }

    if !component_mappings.is_empty() {
        return resolve_explicit_component_mappings(start, workloads, component_mappings);
    }

    if workloads.len() != 1 {
        bail!(
            "application declares {} Component workloads; automatic source building currently \
             supports one workload, so provide one `--component-for WORKLOAD=PATH` per workload",
            workloads.len()
        );
    }

    let path = build_rust_component_release(application_directory)?;
    read_component_artifact(application_directory, &workloads[0], &path)
        .map(|artifact| vec![artifact])
}

fn resolve_explicit_component_mappings(
    start: &Path,
    workloads: &[String],
    mappings: &[String],
) -> Result<Vec<ResolvedComponentArtifact>> {
    let expected = workloads
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut supplied = BTreeMap::<String, PathBuf>::new();

    for mapping in mappings {
        let (workload, path) = parse_component_mapping(mapping)?;

        if !expected.contains(workload) {
            bail!(
                "`--component-for` names unknown workload `{workload}`; expected one of: {}",
                workloads.join(", ")
            );
        }

        if supplied
            .insert(workload.to_owned(), PathBuf::from(path))
            .is_some()
        {
            bail!("duplicate `--component-for` mapping for workload `{workload}`");
        }
    }

    let missing = workloads
        .iter()
        .filter(|workload| !supplied.contains_key(workload.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        bail!(
            "missing `--component-for` mapping for workload(s): {}",
            missing.join(", ")
        );
    }

    workloads
        .iter()
        .map(|workload| {
            let path = supplied
                .get(workload)
                .expect("validated component mapping should exist");
            read_component_artifact(start, workload, path)
        })
        .collect()
}

fn parse_component_mapping(mapping: &str) -> Result<(&str, &str)> {
    let (workload, path) = mapping.split_once('=').ok_or_else(|| {
        anyhow::anyhow!("invalid `--component-for` value `{mapping}`; expected WORKLOAD=PATH")
    })?;

    if workload.is_empty() {
        bail!("invalid `--component-for` value `{mapping}`; workload name cannot be empty");
    }

    if path.is_empty() {
        bail!("invalid `--component-for` value `{mapping}`; path cannot be empty");
    }

    Ok((workload, path))
}

fn read_component_artifact(
    start: &Path,
    workload: &str,
    path: &Path,
) -> Result<ResolvedComponentArtifact> {
    let path = resolve_input_path(start, path)?;
    let bytes = fs::read(&path)
        .with_context(|| format!("could not read component `{}`", path.display()))?;

    Ok(ResolvedComponentArtifact {
        workload: workload.to_owned(),
        path,
        bytes,
    })
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

    fn multi_component_contract() -> &'static str {
        r#"
[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "component"
"#
    }

    #[test]
    fn implicit_component_build_writes_default_dist_release() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), "");

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component-bytes").expect("component should be written");

        let output = build_release(directory.path(), Some(&component), &[], None)
            .expect("prebuilt component should package");

        assert_eq!(output.application, "orders");
        assert_eq!(output.version, "1.2.3");
        assert_eq!(output.components.len(), 1);
        assert_eq!(output.components[0].workload, "main");
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

        build_release(directory.path(), Some(&component), &[], None)
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
            &[],
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
    fn multiple_components_are_mapped_by_workload_name() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");
        fs::write(directory.path().join("worker.wasm"), b"worker-component")
            .expect("worker component should be written");

        let mappings = vec!["api=api.wasm".to_owned(), "worker=worker.wasm".to_owned()];

        let output = build_release(directory.path(), None, &mappings, None)
            .expect("multi-workload release should package");

        assert_eq!(output.components.len(), 2);
        assert_eq!(output.components[0].workload, "api");
        assert_eq!(output.components[1].workload, "worker");

        let encoded = fs::read(&output.release).expect("release should be readable");
        let decoded = PresolveBundle::decode(&encoded).expect("release should verify");

        let workloads = decoded
            .manifest()
            .workloads()
            .iter()
            .map(presolve_bundle::WorkloadManifest::name)
            .collect::<Vec<_>>();

        assert_eq!(workloads, ["api", "worker"]);
    }

    #[test]
    fn component_mapping_order_does_not_change_release_identity() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");
        fs::write(directory.path().join("worker.wasm"), b"worker-component")
            .expect("worker component should be written");

        let first = vec!["api=api.wasm".to_owned(), "worker=worker.wasm".to_owned()];
        let second = vec!["worker=worker.wasm".to_owned(), "api=api.wasm".to_owned()];

        let first_output = build_release(
            directory.path(),
            None,
            &first,
            Some(Path::new("dist/first.presolved")),
        )
        .expect("first release should package");

        let second_output = build_release(
            directory.path(),
            None,
            &second,
            Some(Path::new("dist/second.presolved")),
        )
        .expect("second release should package");

        assert_eq!(first_output.release_digest, second_output.release_digest);
        assert_eq!(
            fs::read(first_output.release).expect("first release should read"),
            fs::read(second_output.release).expect("second release should read")
        );
    }

    #[test]
    fn missing_component_mapping_is_rejected() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");

        let error = build_release(directory.path(), None, &["api=api.wasm".to_owned()], None)
            .expect_err("missing worker mapping should fail");

        assert!(error.to_string().contains("worker"));
    }

    #[test]
    fn unknown_component_mapping_is_rejected() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        let error = build_release(
            directory.path(),
            None,
            &["frontend=frontend.wasm".to_owned()],
            None,
        )
        .expect_err("unknown workload mapping should fail");

        assert!(error.to_string().contains("unknown workload `frontend`"));
    }

    #[test]
    fn duplicate_component_mapping_is_rejected() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        let error = build_release(
            directory.path(),
            None,
            &["api=first.wasm".to_owned(), "api=second.wasm".to_owned()],
            None,
        )
        .expect_err("duplicate workload mapping should fail");

        assert!(error.to_string().contains("duplicate"));
    }

    #[test]
    fn single_component_shorthand_is_rejected_for_multiple_workloads() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component").expect("component should be written");

        let error = build_release(directory.path(), Some(&component), &[], None)
            .expect_err("single shorthand should not guess a workload");

        assert!(error.to_string().contains("single-workload shorthand"));
    }

    #[test]
    fn container_workload_remains_deferred_to_19_3() {
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

        let error = component_workload_names(&contract).expect_err("container build should fail");

        assert!(error.to_string().contains("Chunk 19.2"));
    }

    #[test]
    fn component_mapping_parser_preserves_equals_in_path() {
        assert_eq!(
            parse_component_mapping("api=build/name=value.wasm").expect("mapping should parse"),
            ("api", "build/name=value.wasm")
        );
    }
}
