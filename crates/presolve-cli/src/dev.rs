use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use presolve_node::{assemble_runtime, realize_local_providers};
use serde_json::Value;

use crate::check::CompatibilityCheck;

const RUST_COMPONENT_TARGET: &str = "wasm32-wasip2";

#[derive(Debug)]
pub(crate) struct DevExecution {
    pub(crate) workload: String,
    pub(crate) component: PathBuf,
    pub(crate) output: String,
}

#[derive(Debug, PartialEq, Eq)]
struct CargoProject {
    target_name: String,
    target_directory: PathBuf,
}

pub(crate) fn run_local_dev(
    check: &CompatibilityCheck,
    start: &Path,
    component_override: Option<&Path>,
    input: &str,
) -> Result<DevExecution> {
    let plan = check
        .report
        .plan()
        .context("local development requires a resolved DeploymentPlan")?;

    let workload = single_workload_name(&check.application.contract)?;
    let component = match component_override {
        Some(path) => resolve_component_override(start, path)?,
        None => build_rust_component(application_directory(check)?)?,
    };

    let bytes = fs::read(&component)
        .with_context(|| format!("could not read component `{}`", component.display()))?;

    let registry = realize_local_providers(plan)
        .context("could not realize the resolved plan with local providers")?;
    let runtime =
        assemble_runtime(plan, &registry).context("could not assemble the local runtime")?;
    let output = runtime
        .run_bytes(&bytes, input)
        .with_context(|| format!("could not execute component `{}`", component.display()))?;

    Ok(DevExecution {
        workload,
        component,
        output,
    })
}

fn application_directory(check: &CompatibilityCheck) -> Result<&Path> {
    check
        .application
        .path
        .parent()
        .context("Application Contract path has no parent directory")
}

fn single_workload_name(contract: &presolve_contract::ApplicationContract) -> Result<String> {
    match contract.workloads() {
        [] => Ok("main".to_owned()),
        [workload] => Ok(workload.name().to_owned()),
        workloads => bail!(
            "`presolve dev` currently supports one workload; application declares {}",
            workloads.len()
        ),
    }
}

fn resolve_component_override(start: &Path, path: &Path) -> Result<PathBuf> {
    let component = if path.is_absolute() {
        path.to_path_buf()
    } else {
        start.join(path)
    };

    let metadata = fs::metadata(&component)
        .with_context(|| format!("could not inspect component `{}`", component.display()))?;

    if !metadata.is_file() {
        bail!("component `{}` is not a regular file", component.display());
    }

    Ok(component)
}

fn build_rust_component(application_directory: &Path) -> Result<PathBuf> {
    let manifest = application_directory.join("Cargo.toml");

    if !manifest.is_file() {
        bail!(
            "no component was supplied and `{}` does not exist; \
             provide `--component PATH` or use a Rust application crate with `Cargo.toml` beside `presolve.toml`",
            manifest.display()
        );
    }

    let project = cargo_project(&manifest)?;

    eprintln!("Building Rust component for {RUST_COMPONENT_TARGET}...");

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target")
        .arg(RUST_COMPONENT_TARGET)
        .current_dir(application_directory)
        .status()
        .with_context(|| format!("could not run Cargo for `{}`", manifest.display()))?;

    if !status.success() {
        bail!(
            "Cargo build failed for `{}` with status {status}",
            manifest.display()
        );
    }

    let artifact_name = format!("{}.wasm", project.target_name.replace('-', "_"));
    let component = project
        .target_directory
        .join(RUST_COMPONENT_TARGET)
        .join("debug")
        .join(artifact_name);

    if !component.is_file() {
        bail!(
            "Cargo build succeeded but expected component `{}` was not produced",
            component.display()
        );
    }

    Ok(component)
}

fn cargo_project(manifest: &Path) -> Result<CargoProject> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(manifest)
        .output()
        .with_context(|| {
            format!(
                "could not run `cargo metadata` for `{}`",
                manifest.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "`cargo metadata` failed for `{}`: {stderr}",
            manifest.display()
        );
    }

    let metadata: Value =
        serde_json::from_slice(&output.stdout).context("Cargo returned invalid metadata JSON")?;

    cargo_project_from_metadata(&metadata, manifest)
}

fn cargo_project_from_metadata(metadata: &Value, manifest: &Path) -> Result<CargoProject> {
    let manifest = manifest
        .canonicalize()
        .with_context(|| format!("could not canonicalize `{}`", manifest.display()))?;

    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .context("Cargo metadata does not contain a `packages` array")?;

    let package = packages
        .iter()
        .find(|package| {
            package
                .get("manifest_path")
                .and_then(Value::as_str)
                .map(Path::new)
                .and_then(|path| path.canonicalize().ok())
                .is_some_and(|path| path == manifest)
        })
        .context("Cargo metadata does not contain the application package")?;

    let targets = package
        .get("targets")
        .and_then(Value::as_array)
        .context("Cargo metadata package does not contain a `targets` array")?;

    let cdylib_targets = targets
        .iter()
        .filter(|target| {
            target
                .get("crate_types")
                .and_then(Value::as_array)
                .is_some_and(|crate_types| {
                    crate_types
                        .iter()
                        .any(|crate_type| crate_type.as_str() == Some("cdylib"))
                })
        })
        .collect::<Vec<_>>();

    let target = match cdylib_targets.as_slice() {
        [target] => *target,
        [] => bail!(
            "`{}` must expose exactly one `cdylib` target for automatic `presolve dev` builds",
            manifest.display()
        ),
        targets => bail!(
            "`{}` exposes {} `cdylib` targets; use `--component PATH` to choose an artifact explicitly",
            manifest.display(),
            targets.len()
        ),
    };

    let target_name = target
        .get("name")
        .and_then(Value::as_str)
        .context("Cargo metadata cdylib target has no name")?
        .to_owned();

    let target_directory = metadata
        .get("target_directory")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .context("Cargo metadata has no target directory")?;

    Ok(CargoProject {
        target_name,
        target_directory,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_contract::parse_contract;

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
                "presolve-cli-dev-{}-{timestamp}-{sequence}",
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

    #[test]
    fn implicit_single_workload_is_main() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "hello"
version = "1.0.0"
"#,
        )
        .expect("contract should parse");

        assert_eq!(
            single_workload_name(&contract).expect("implicit workload should work"),
            "main"
        );
    }

    #[test]
    fn multiple_workloads_are_rejected_for_initial_dev_runner() {
        let contract = parse_contract(
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

        let Err(error) = single_workload_name(&contract) else {
            panic!("multi-workload dev should fail");
        };

        assert!(
            error
                .to_string()
                .contains("currently supports one workload")
        );
    }

    #[test]
    fn relative_component_override_is_resolved_from_invocation_directory() {
        let directory = TestDirectory::new();
        let component = directory.path().join("app.wasm");
        fs::write(&component, b"wasm").expect("component fixture should be written");

        assert_eq!(
            resolve_component_override(directory.path(), Path::new("app.wasm"))
                .expect("relative component should resolve"),
            component
        );
    }

    #[test]
    fn cargo_metadata_selects_cdylib_for_application_manifest() {
        let directory = TestDirectory::new();
        let manifest = directory.path().join("Cargo.toml");
        fs::write(&manifest, "[package]\nname = \"demo\"\n")
            .expect("manifest fixture should be written");

        let manifest_string = manifest
            .canonicalize()
            .expect("manifest should canonicalize")
            .to_string_lossy()
            .into_owned();
        let target_directory = directory.path().join("target");
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": manifest_string,
                "targets": [
                    {
                        "name": "demo",
                        "crate_types": ["cdylib"]
                    }
                ]
            }],
            "target_directory": target_directory.to_string_lossy()
        });

        assert_eq!(
            cargo_project_from_metadata(&metadata, &manifest)
                .expect("metadata should select project"),
            CargoProject {
                target_name: "demo".to_owned(),
                target_directory,
            }
        );
    }

    #[test]
    fn cargo_metadata_rejects_package_without_cdylib() {
        let directory = TestDirectory::new();
        let manifest = directory.path().join("Cargo.toml");
        fs::write(&manifest, "[package]\nname = \"demo\"\n")
            .expect("manifest fixture should be written");

        let manifest_string = manifest
            .canonicalize()
            .expect("manifest should canonicalize")
            .to_string_lossy()
            .into_owned();
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": manifest_string,
                "targets": [
                    {
                        "name": "demo",
                        "crate_types": ["lib"]
                    }
                ]
            }],
            "target_directory": directory.path().join("target").to_string_lossy()
        });

        let Err(error) = cargo_project_from_metadata(&metadata, &manifest) else {
            panic!("missing cdylib should fail");
        };

        assert!(error.to_string().contains("exactly one `cdylib`"));
    }
}
