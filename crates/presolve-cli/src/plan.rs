use std::fmt::{self, Write as _};

use anyhow::{Context, Result};
use presolve_capability::CapabilityContract;
use presolve_resolver::CapabilityBinding;

use crate::check::CompatibilityCheck;

pub(crate) fn render_deployment_plan(check: &CompatibilityCheck) -> Result<String> {
    let plan = check
        .report
        .plan()
        .context("cannot render DeploymentPlan because application resolution failed")?;

    let application = check.application.contract.application();
    let environment = check.environment.specification.environment();
    let mut output = String::new();

    line(&mut output, format_args!("Presolve plan"));
    line(
        &mut output,
        format_args!(
            "  application: {} {}",
            application.name(),
            application.version()
        ),
    );
    line(
        &mut output,
        format_args!(
            "  {}: {}",
            check.application.source_label(),
            check.application.path.display()
        ),
    );
    if let Some(digest) = check.application.release_digest() {
        line(&mut output, format_args!("  release digest: {digest}"));
    }
    line(
        &mut output,
        format_args!(
            "  environment: {} ({})",
            environment.name(),
            plan.environment_id()
        ),
    );
    line(
        &mut output,
        format_args!("  specification: {}", check.environment.path.display()),
    );

    render_workloads(&mut output, plan);
    render_bindings(&mut output, plan);

    if plan.unbound_optional_capabilities().is_empty() {
        line(
            &mut output,
            format_args!("  optional capabilities unbound: none"),
        );
    } else {
        line(
            &mut output,
            format_args!(
                "  optional capabilities unbound: {}",
                plan.unbound_optional_capabilities().join(", ")
            ),
        );
    }

    Ok(output)
}

fn render_workloads(output: &mut String, plan: &presolve_resolver::DeploymentPlan) {
    line(output, format_args!("  workloads:"));

    for workload in plan.workloads() {
        line(
            output,
            format_args!("    {} -> {}", workload.name(), workload.kind()),
        );
    }
}

fn render_bindings(output: &mut String, plan: &presolve_resolver::DeploymentPlan) {
    line(output, format_args!("  capability bindings:"));

    if plan.bindings().is_empty() {
        line(output, format_args!("    none"));
        return;
    }

    for binding in plan.bindings() {
        render_binding(output, binding);
    }
}

fn render_binding(output: &mut String, binding: &CapabilityBinding) {
    line(
        output,
        format_args!("    {}@{}", binding.interface(), binding.provider_version()),
    );
    line(
        output,
        format_args!("      provider: {}", binding.provider_name()),
    );
    line(
        output,
        format_args!("      provider id: {}", binding.provider_id()),
    );

    render_members(
        output,
        "      negotiated features",
        binding.negotiated_features(),
    );

    match binding.semantic_contract() {
        Some(contract) => render_semantic_contract(output, contract),
        None => line(output, format_args!("      semantic contract: unavailable")),
    }
}

fn render_semantic_contract(output: &mut String, contract: &CapabilityContract) {
    line(
        output,
        format_args!(
            "      semantic contract: {}@{}",
            contract.interface(),
            contract.version()
        ),
    );
    render_members(output, "        operations", contract.operations());
    render_members(output, "        guarantees", contract.guarantees());
    render_members(
        output,
        "        optional features",
        contract.optional_features(),
    );
}

fn render_members(output: &mut String, label: &str, members: &[String]) {
    if members.is_empty() {
        line(output, format_args!("{label}: none"));
    } else {
        line(output, format_args!("{label}: {}", members.join(", ")));
    }
}

fn line(output: &mut String, arguments: fmt::Arguments<'_>) {
    output
        .write_fmt(arguments)
        .expect("writing DeploymentPlan inspection to String should not fail");
    output.push('\n');
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

    use crate::{
        check::check_compatibility, environment::create_environment, provider::add_provider,
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
                "presolve-cli-plan-{}-{timestamp}-{sequence}",
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

    fn write_application(directory: &Path) {
        fs::write(
            directory.join(CONTRACT_FILE_NAME),
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

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
optional = true
"#,
        )
        .expect("application contract should be written");
    }

    #[test]
    fn plan_renders_exact_resolver_selections() {
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
        write_application(directory.path());

        let check =
            check_compatibility(directory.path()).expect("compatibility resolution should run");
        let rendered = render_deployment_plan(&check).expect("resolved plan should render");

        assert!(rendered.contains("Presolve plan"));
        assert!(rendered.contains("application: orders 1.0.0"));
        assert!(rendered.contains("environment: local (env_"));
        assert!(rendered.contains("api -> component"));
        assert!(rendered.contains("presolve:kv/store@0.1.0"));
        assert!(rendered.contains("provider: memory-services"));
        assert!(rendered.contains("provider id: prv_"));
        assert!(rendered.contains("negotiated features: none"));
        assert!(rendered.contains("semantic contract: presolve:kv/store@0.1.0"));
        assert!(rendered.contains("operations: delete, get, set"));
        assert!(rendered.contains("guarantees:"));
        assert!(rendered.contains("optional capabilities unbound: presolve:objects/store"));
    }

    #[test]
    fn plan_cannot_render_rejected_resolution() {
        let directory = TestDirectory::new();

        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");
        write_application(directory.path());

        let check =
            check_compatibility(directory.path()).expect("compatibility resolution should run");

        assert!(!check.report.is_resolved());

        let error =
            render_deployment_plan(&check).expect_err("rejected resolution must not have a plan");

        assert!(error.to_string().contains("resolution failed"));
    }
}
