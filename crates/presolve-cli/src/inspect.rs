use std::{
    fmt::{self, Write as _},
    path::Path,
};

use anyhow::Result;
use presolve_contract::{OutboundNetworkMode, WorkloadKind};

use crate::application::{ApplicationDocument, load_application};

pub(crate) fn inspect_application(start: &Path) -> Result<String> {
    let document = load_application(start)?;
    Ok(render_application(&document))
}

fn render_application(document: &ApplicationDocument) -> String {
    let mut output = String::new();
    let contract = &document.contract;
    let application = contract.application();

    line(&mut output, format_args!("Presolve inspect"));
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
        format_args!("  contract: {}", document.path.display()),
    );
    line(
        &mut output,
        format_args!("  contract version: {}", contract.contract_version()),
    );

    if let Some(description) = application.description() {
        line(&mut output, format_args!("  description: {description}"));
    }

    render_workloads(&mut output, document);
    render_capabilities(&mut output, document);
    render_topology(&mut output, document);
    render_resources(&mut output, document);
    render_network(&mut output, document);

    output
}

fn render_workloads(output: &mut String, document: &ApplicationDocument) {
    line(output, format_args!("  workloads:"));

    if document.contract.workloads().is_empty() {
        line(
            output,
            format_args!("    main -> {} (implicit)", WorkloadKind::Component),
        );
        return;
    }

    for workload in document.contract.workloads() {
        line(
            output,
            format_args!("    {} -> {}", workload.name(), workload.kind()),
        );
    }
}

fn render_capabilities(output: &mut String, document: &ApplicationDocument) {
    line(output, format_args!("  capabilities:"));

    if document.contract.capabilities().is_empty() {
        line(output, format_args!("    none"));
        return;
    }

    for capability in document.contract.capabilities() {
        let requirement = if capability.optional() {
            "optional"
        } else {
            "required"
        };

        line(
            output,
            format_args!(
                "    {} {} ({requirement})",
                capability.interface(),
                capability.version()
            ),
        );

        if !capability.required_features().is_empty() {
            line(
                output,
                format_args!(
                    "      required features: {}",
                    capability.required_features().join(", ")
                ),
            );
        }

        if !capability.preferred_features().is_empty() {
            line(
                output,
                format_args!(
                    "      preferred features: {}",
                    capability.preferred_features().join(", ")
                ),
            );
        }
    }
}

fn render_topology(output: &mut String, document: &ApplicationDocument) {
    line(output, format_args!("  interfaces:"));

    if document.contract.interfaces().is_empty() {
        line(output, format_args!("    none"));
    } else {
        for interface in document.contract.interfaces() {
            line(
                output,
                format_args!("    {} -> {}", interface.name(), interface.kind()),
            );
        }
    }

    line(output, format_args!("  relationships:"));

    if document.contract.relationships().is_empty() {
        line(output, format_args!("    none"));
    } else {
        for relationship in document.contract.relationships() {
            line(
                output,
                format_args!(
                    "    {} -> {} via {}",
                    relationship.from(),
                    relationship.to(),
                    relationship.interface()
                ),
            );
        }
    }
}

fn render_resources(output: &mut String, document: &ApplicationDocument) {
    let resources = document.contract.resources();

    line(output, format_args!("  resources:"));

    match resources.memory_mib() {
        Some(memory_mib) => {
            line(output, format_args!("    memory: {memory_mib} MiB"));
        }
        None => line(output, format_args!("    memory: unspecified")),
    }

    match resources.cpu_millis() {
        Some(cpu_millis) => {
            line(output, format_args!("    cpu: {cpu_millis}m"));
        }
        None => line(output, format_args!("    cpu: unspecified")),
    }
}

fn render_network(output: &mut String, document: &ApplicationDocument) {
    let network = document.contract.network();

    line(output, format_args!("  network:"));
    line(
        output,
        format_args!("    outbound: {}", outbound_mode(network.outbound())),
    );

    if !network.allow().is_empty() {
        line(
            output,
            format_args!("    allow: {}", network.allow().join(", ")),
        );
    }
}

const fn outbound_mode(mode: OutboundNetworkMode) -> &'static str {
    match mode {
        OutboundNetworkMode::Deny => "deny",
        OutboundNetworkMode::AllowList => "allow_list",
        OutboundNetworkMode::AllowAll => "allow_all",
    }
}

fn line(output: &mut String, arguments: fmt::Arguments<'_>) {
    output
        .write_fmt(arguments)
        .expect("writing application inspection to String should not fail");
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_contract::CONTRACT_FILE_NAME;

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
                "presolve-cli-inspect-{}-{timestamp}-{sequence}",
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
    fn inspect_renders_application_semantics() {
        let directory = TestDirectory::new();

        fs::write(
            directory.path().join(CONTRACT_FILE_NAME),
            r#"
contract_version = "0.1"

[application]
name = "orders"
version = "1.2.3"
description = "Order processing"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "container"

[[interfaces]]
name = "jobs"
kind = "event"

[[relationships]]
from = "api"
to = "worker"
interface = "jobs"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
preferred_features = ["user-metadata"]

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
optional = true

[resources]
memory_mib = 512
cpu_millis = 750

[network]
outbound = "allow_list"
allow = ["api.example.com"]
"#,
        )
        .expect("contract should be written");

        let rendered =
            inspect_application(directory.path()).expect("application inspection should succeed");

        assert!(rendered.contains("application: orders 1.2.3"));
        assert!(rendered.contains("description: Order processing"));
        assert!(rendered.contains("api -> component"));
        assert!(rendered.contains("worker -> container"));
        assert!(rendered.contains("presolve:objects/store ^0.1 (required)"));
        assert!(rendered.contains("required features: range-read"));
        assert!(rendered.contains("preferred features: user-metadata"));
        assert!(rendered.contains("presolve:kv/store ^0.1 (optional)"));
        assert!(rendered.contains("jobs -> event"));
        assert!(rendered.contains("api -> worker via jobs"));
        assert!(rendered.contains("memory: 512 MiB"));
        assert!(rendered.contains("cpu: 750m"));
        assert!(rendered.contains("outbound: allow_list"));
        assert!(rendered.contains("allow: api.example.com"));
    }

    #[test]
    fn inspect_renders_implicit_main_workload() {
        let directory = TestDirectory::new();

        fs::write(
            directory.path().join(CONTRACT_FILE_NAME),
            r#"
contract_version = "0.1"

[application]
name = "hello"
version = "1.0.0"
"#,
        )
        .expect("contract should be written");

        let rendered =
            inspect_application(directory.path()).expect("application inspection should succeed");

        assert!(rendered.contains("main -> component (implicit)"));
        assert!(rendered.contains("capabilities:\n    none"));
        assert!(rendered.contains("memory: unspecified"));
        assert!(rendered.contains("cpu: unspecified"));
        assert!(rendered.contains("outbound: deny"));
    }

    #[test]
    fn inspect_does_not_require_environment_specification() {
        let directory = TestDirectory::new();

        fs::write(
            directory.path().join(CONTRACT_FILE_NAME),
            r#"
contract_version = "0.1"

[application]
name = "standalone"
version = "1.0.0"
"#,
        )
        .expect("contract should be written");

        assert!(!directory.path().join("presolve.env.toml").exists());

        inspect_application(directory.path())
            .expect("application inspection must not require an environment");
    }
}
