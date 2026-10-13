use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use presolve_bundle::application_contract_for_resolution;
use presolve_contract::ApplicationContract;
use presolve_environment::inventory_from_specification;
use presolve_resolver::{ResolutionReport, resolve};

use crate::{
    application::load_application,
    environment::{EnvironmentDocument, load_environment},
    release_inspect::load_release,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompatibilityApplicationSource {
    Contract,
    Release { digest: String },
}

#[derive(Debug)]
pub(crate) struct CompatibilityApplication {
    pub(crate) path: PathBuf,
    pub(crate) contract: ApplicationContract,
    pub(crate) source: CompatibilityApplicationSource,
}

impl CompatibilityApplication {
    pub(crate) const fn source_label(&self) -> &'static str {
        match self.source {
            CompatibilityApplicationSource::Contract => "contract",
            CompatibilityApplicationSource::Release { .. } => "release",
        }
    }

    pub(crate) fn release_digest(&self) -> Option<&str> {
        match &self.source {
            CompatibilityApplicationSource::Contract => None,
            CompatibilityApplicationSource::Release { digest } => Some(digest),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CompatibilityCheck {
    pub(crate) application: CompatibilityApplication,
    pub(crate) environment: EnvironmentDocument,
    pub(crate) report: ResolutionReport,
}

pub(crate) fn check_compatibility(start: &Path) -> Result<CompatibilityCheck> {
    let document = load_application(start)?;
    let application = CompatibilityApplication {
        path: document.path,
        contract: document.contract,
        source: CompatibilityApplicationSource::Contract,
    };

    resolve_compatibility(start, application)
}

pub(crate) fn check_release_compatibility(
    start: &Path,
    requested_release: &Path,
) -> Result<CompatibilityCheck> {
    let release = load_release(start, requested_release)?;
    let contract = application_contract_for_resolution(release.bundle.manifest())
        .context("could not reconstruct resolver input from immutable release")?;
    let digest = release
        .bundle
        .release_digest()
        .context("could not compute immutable release digest for resolution")?
        .to_string();

    let application = CompatibilityApplication {
        path: release.path,
        contract,
        source: CompatibilityApplicationSource::Release { digest },
    };

    resolve_compatibility(start, application)
}

fn resolve_compatibility(
    start: &Path,
    application: CompatibilityApplication,
) -> Result<CompatibilityCheck> {
    let environment = load_environment(start)?;

    let inventory =
        inventory_from_specification(&environment.specification).with_context(|| {
            format!(
                "could not normalize environment `{}` for resolution",
                environment.path.display()
            )
        })?;

    let report = resolve(&application.contract, &inventory);

    Ok(CompatibilityCheck {
        application,
        environment,
        report,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_bundle::PresolveBundle;
    use presolve_contract::{CONTRACT_FILE_NAME, parse_contract};

    use crate::{environment::create_environment, provider::add_provider};

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
                "presolve-cli-check-{}-{timestamp}-{sequence}",
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

    fn write_application(directory: &Path, memory_mib: u64, cpu_millis: u32) {
        fs::create_dir_all(directory).expect("application directory should be created");
        fs::write(
            directory.join(CONTRACT_FILE_NAME),
            format!(
                r#"
contract_version = "0.1"

[application]
name = "orders"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "component"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = {memory_mib}
cpu_millis = {cpu_millis}
"#
            ),
        )
        .expect("application contract should be written");
    }

    #[test]
    fn check_resolves_nearest_application_against_parent_environment() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        add_provider(
            directory.path(),
            "memory-services",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect("provider should be added");

        let application_directory = directory.path().join("apps/orders");
        let nested = application_directory.join("src");
        write_application(&application_directory, 256, 500);
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let check = check_compatibility(&nested).expect("compatibility check should run");

        assert_eq!(check.application.contract.application().name(), "orders");
        assert_eq!(check.application.source_label(), "contract");
        assert_eq!(
            check.environment.specification.environment().name(),
            "local"
        );
        assert!(check.report.is_resolved());

        let plan = check
            .report
            .plan()
            .expect("resolved report should have a plan");
        assert_eq!(plan.bindings().len(), 1);
        assert_eq!(plan.bindings()[0].provider_name(), "memory-services");
    }

    #[test]
    fn detached_release_resolves_without_source_contract() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "portable", 1_024, 1_000)
            .expect("environment should be created");
        add_provider(
            directory.path(),
            "release-kv",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect("provider should be added");

        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "detached-orders"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 64
cpu_millis = 100
"#,
        )
        .expect("fixture contract should parse");
        let bundle = PresolveBundle::from_component(&contract, b"component bytes")
            .expect("fixture release should build");
        let release_path = directory.path().join("detached-orders.presolved");
        fs::write(&release_path, bundle.encode().expect("release should encode"))
            .expect("release should be written");

        assert!(!directory.path().join(CONTRACT_FILE_NAME).exists());

        let check = check_release_compatibility(directory.path(), &release_path)
            .expect("detached release compatibility should run");

        assert!(check.report.is_resolved());
        assert_eq!(check.application.source_label(), "release");
        assert_eq!(check.application.path, release_path.canonicalize().unwrap());
        assert_eq!(
            check.application.release_digest(),
            Some(bundle.release_digest().unwrap().to_string().as_str())
        );
        assert_eq!(
            check.report.plan().unwrap().bindings()[0].provider_name(),
            "release-kv"
        );
    }

    #[test]
    fn check_reports_missing_capability_with_stable_code() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let application_directory = directory.path().join("app");
        write_application(&application_directory, 256, 500);

        let check =
            check_compatibility(&application_directory).expect("compatibility check should run");

        assert!(!check.report.is_resolved());
        assert_eq!(check.report.problems().len(), 1);
        assert_eq!(check.report.problems()[0].code(), "PS2001");
    }

    #[test]
    fn check_reports_all_resource_shortfalls() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        add_provider(
            directory.path(),
            "memory-services",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect("provider should be added");

        let application_directory = directory.path().join("app");
        write_application(&application_directory, 2_048, 2_000);

        let check =
            check_compatibility(&application_directory).expect("compatibility check should run");

        let codes = check
            .report
            .problems()
            .iter()
            .map(presolve_resolver::ResolutionProblem::code)
            .collect::<Vec<_>>();

        assert_eq!(codes, ["PS2002", "PS2003"]);
    }
}
