use std::path::Path;

use anyhow::{Context, Result};
use presolve_environment::inventory_from_specification;
use presolve_resolver::{ResolutionReport, resolve};

use crate::{
    application::{ApplicationDocument, load_application},
    environment::{EnvironmentDocument, load_environment},
};

#[derive(Debug)]
pub(crate) struct CompatibilityCheck {
    pub(crate) application: ApplicationDocument,
    pub(crate) environment: EnvironmentDocument,
    pub(crate) report: ResolutionReport,
}

pub(crate) fn check_compatibility(start: &Path) -> Result<CompatibilityCheck> {
    let application = load_application(start)?;
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

    use presolve_contract::CONTRACT_FILE_NAME;

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
