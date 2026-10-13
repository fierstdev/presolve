use std::path::Path;

use anyhow::{Context, Result};
use presolve_capability::CapabilityContract;
use presolve_contract::{OutboundNetworkMode, WorkloadKind};
use presolve_resolver::{CapabilityBinding, DeploymentPlan, ResolutionProblem};
use serde_json::{Value, json};

use crate::{
    application::{ApplicationDocument, load_application},
    check::{CompatibilityApplication, CompatibilityCheck},
    release_inspect::ReleaseDocument,
};

const CLI_JSON_SCHEMA: &str = "presolve.cli";
const CLI_JSON_SCHEMA_VERSION: &str = "0.1";

pub(crate) fn render_application_json(start: &Path) -> Result<String> {
    let document = load_application(start)?;
    encode(&application_inspection_json(&document))
}

pub(crate) fn render_release_json(document: &ReleaseDocument) -> Result<String> {
    let manifest = document.bundle.manifest();
    let application = manifest.application();
    let requirements = manifest.requirements();
    let release_digest = document
        .bundle
        .release_digest()
        .context("could not compute canonical application release digest")?;

    encode(&json!({
        "schema": CLI_JSON_SCHEMA,
        "schema_version": CLI_JSON_SCHEMA_VERSION,
        "kind": "release_inspection",
        "release": {
            "path": document.path.display().to_string(),
            "encoded_size": document.encoded_size,
            "bundle_version": manifest.bundle_version().to_string(),
            "digest": release_digest.to_string(),
        },
        "application": {
            "name": application.name(),
            "version": application.version().to_string(),
            "description": application.description(),
        },
        "requirements": {
            "contract_version": requirements.contract_version().to_string(),
            "capabilities": requirements.capabilities().iter().map(|capability| json!({
                "interface": capability.interface(),
                "version": capability.version(),
                "optional": capability.optional(),
                "required_features": capability.required_features(),
                "preferred_features": capability.preferred_features(),
            })).collect::<Vec<_>>(),
            "resources": {
                "memory_mib": requirements.resources().memory_mib(),
                "cpu_millis": requirements.resources().cpu_millis(),
            },
            "network": {
                "outbound": outbound_mode(requirements.network().outbound()),
                "allow": requirements.network().allow(),
            },
        },
        "workloads": manifest.workloads().iter().map(|workload| json!({
            "name": workload.name(),
            "kind": workload.kind().to_string(),
            "artifacts": workload.artifacts().iter().map(|artifact| json!({
                "path": artifact.path(),
                "digest": artifact.digest().to_string(),
                "size": artifact.size(),
                "media_type": artifact.media_type(),
            })).collect::<Vec<_>>(),
            "oci_image": workload.oci_image().map(|image| json!({
                "media_type": image.media_type(),
                "digest": image.digest(),
                "size": image.size(),
            })),
        })).collect::<Vec<_>>(),
        "interfaces": manifest.interfaces().iter().map(|interface| json!({
            "name": interface.name(),
            "kind": interface.kind().to_string(),
        })).collect::<Vec<_>>(),
        "relationships": manifest.relationships().iter().map(|relationship| json!({
            "from": relationship.from(),
            "to": relationship.to(),
            "interface": relationship.interface(),
        })).collect::<Vec<_>>(),
    }))
}

pub(crate) fn render_check_json(check: &CompatibilityCheck) -> Result<String> {
    encode(&json!({
        "schema": CLI_JSON_SCHEMA,
        "schema_version": CLI_JSON_SCHEMA_VERSION,
        "kind": "compatibility_check",
        "application": compatibility_application_context_json(&check.application),
        "environment": environment_context_json(check),
        "status": if check.report.is_resolved() { "compatible" } else { "incompatible" },
        "problems": problems_json(check),
    }))
}

pub(crate) fn render_plan_json(check: &CompatibilityCheck) -> Result<String> {
    encode(&json!({
        "schema": CLI_JSON_SCHEMA,
        "schema_version": CLI_JSON_SCHEMA_VERSION,
        "kind": "deployment_plan",
        "application": compatibility_application_context_json(&check.application),
        "environment": environment_context_json(check),
        "status": if check.report.is_resolved() { "resolved" } else { "rejected" },
        "plan": check.report.plan().map(deployment_plan_json),
        "problems": problems_json(check),
    }))
}

fn application_inspection_json(document: &ApplicationDocument) -> Value {
    let contract = &document.contract;

    json!({
        "schema": CLI_JSON_SCHEMA,
        "schema_version": CLI_JSON_SCHEMA_VERSION,
        "kind": "application_inspection",
        "application": application_context_json(document),
        "description": contract.application().description(),
        "workloads": inspection_workloads_json(document),
        "capabilities": contract.capabilities().iter().map(|capability| json!({
            "interface": capability.interface(),
            "version": capability.version().to_string(),
            "optional": capability.optional(),
            "required_features": capability.required_features(),
            "preferred_features": capability.preferred_features(),
        })).collect::<Vec<_>>(),
        "interfaces": contract.interfaces().iter().map(|interface| json!({
            "name": interface.name(),
            "kind": interface.kind().to_string(),
        })).collect::<Vec<_>>(),
        "relationships": contract.relationships().iter().map(|relationship| json!({
            "from": relationship.from(),
            "to": relationship.to(),
            "interface": relationship.interface(),
        })).collect::<Vec<_>>(),
        "resources": {
            "memory_mib": contract.resources().memory_mib(),
            "cpu_millis": contract.resources().cpu_millis(),
        },
        "network": {
            "outbound": outbound_mode(contract.network().outbound()),
            "allow": contract.network().allow(),
        },
    })
}

fn application_context_json(document: &ApplicationDocument) -> Value {
    let contract = &document.contract;
    let application = contract.application();

    json!({
        "name": application.name(),
        "version": application.version().to_string(),
        "contract": document.path.display().to_string(),
        "contract_version": contract.contract_version().to_string(),
    })
}

fn compatibility_application_context_json(document: &CompatibilityApplication) -> Value {
    let contract = &document.contract;
    let application = contract.application();
    let source_kind = document.source_label();
    let source_path = document.path.display().to_string();

    json!({
        "name": application.name(),
        "version": application.version().to_string(),
        "contract": if source_kind == "contract" { Some(source_path.as_str()) } else { None },
        "release": if source_kind == "release" { Some(source_path.as_str()) } else { None },
        "release_digest": document.release_digest(),
        "contract_version": contract.contract_version().to_string(),
    })
}

fn inspection_workloads_json(document: &ApplicationDocument) -> Vec<Value> {
    if document.contract.workloads().is_empty() {
        return vec![json!({
            "name": "main",
            "kind": WorkloadKind::Component.to_string(),
            "implicit": true,
        })];
    }

    document
        .contract
        .workloads()
        .iter()
        .map(|workload| {
            json!({
                "name": workload.name(),
                "kind": workload.kind().to_string(),
                "implicit": false,
            })
        })
        .collect()
}

fn environment_context_json(check: &CompatibilityCheck) -> Value {
    let environment = check.environment.specification.environment();

    json!({
        "name": environment.name(),
        "id": environment.id().to_string(),
        "specification": check.environment.path.display().to_string(),
    })
}

fn deployment_plan_json(plan: &DeploymentPlan) -> Value {
    json!({
        "environment_id": plan.environment_id().to_string(),
        "workloads": plan.workloads().iter().map(|workload| json!({
            "name": workload.name(),
            "kind": workload.kind().to_string(),
        })).collect::<Vec<_>>(),
        "bindings": plan.bindings().iter().map(binding_json).collect::<Vec<_>>(),
        "unbound_optional_capabilities": plan.unbound_optional_capabilities(),
    })
}

fn binding_json(binding: &CapabilityBinding) -> Value {
    json!({
        "interface": binding.interface(),
        "provider_id": binding.provider_id().to_string(),
        "provider_name": binding.provider_name(),
        "provider_version": binding.provider_version().to_string(),
        "negotiated_features": binding.negotiated_features(),
        "semantic_contract": binding.semantic_contract().map(semantic_contract_json),
    })
}

fn semantic_contract_json(contract: &CapabilityContract) -> Value {
    json!({
        "interface": contract.interface(),
        "version": contract.version().to_string(),
        "operations": contract.operations(),
        "guarantees": contract.guarantees(),
        "optional_features": contract.optional_features(),
    })
}

fn problems_json(check: &CompatibilityCheck) -> Vec<Value> {
    check
        .report
        .problems()
        .iter()
        .map(resolution_problem_json)
        .collect()
}

fn resolution_problem_json(problem: &ResolutionProblem) -> Value {
    match problem {
        ResolutionProblem::UnsupportedWorkload {
            workload,
            kind,
            supported_kinds,
        } => json!({
            "type": "unsupported_workload",
            "code": problem.code(),
            "workload": workload,
            "required_kind": kind.to_string(),
            "supported_kinds": supported_kinds.iter().map(ToString::to_string).collect::<Vec<_>>(),
        }),
        ResolutionProblem::MissingCapability { interface, version } => json!({
            "type": "missing_capability",
            "code": problem.code(),
            "interface": interface,
            "version": version.to_string(),
        }),
        ResolutionProblem::MissingCapabilityFeatures {
            interface,
            version,
            required_features,
            available_features,
        } => json!({
            "type": "missing_capability_features",
            "code": problem.code(),
            "interface": interface,
            "version": version.to_string(),
            "required_features": required_features,
            "available_features": available_features,
        }),
        ResolutionProblem::InsufficientMemory {
            requested_mib,
            available_mib,
        } => json!({
            "type": "insufficient_memory",
            "code": problem.code(),
            "requested_mib": requested_mib,
            "available_mib": available_mib,
        }),
        ResolutionProblem::InsufficientCpu {
            requested_millis,
            available_millis,
        } => json!({
            "type": "insufficient_cpu",
            "code": problem.code(),
            "requested_millis": requested_millis,
            "available_millis": available_millis,
        }),
    }
}

const fn outbound_mode(mode: OutboundNetworkMode) -> &'static str {
    match mode {
        OutboundNetworkMode::Deny => "deny",
        OutboundNetworkMode::AllowList => "allow_list",
        OutboundNetworkMode::AllowAll => "allow_all",
    }
}

fn encode(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).context("could not serialize Presolve CLI JSON output")
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
                "presolve-cli-json-{}-{timestamp}-{sequence}",
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

    fn write_application(directory: &Path, optional_objects: bool) {
        let optional = if optional_objects {
            r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
optional = true
"#
        } else {
            ""
        };

        fs::write(
            directory.join(CONTRACT_FILE_NAME),
            format!(
                r#"
contract_version = "0.1"

[application]
name = "orders"
version = "1.0.0"
description = "Orders"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

{optional}
"#
            ),
        )
        .expect("application contract should be written");
    }

    fn parse(rendered: &str) -> Value {
        serde_json::from_str(rendered).expect("CLI JSON should parse")
    }

    #[test]
    fn inspect_json_normalizes_implicit_workload_without_environment() {
        let directory = TestDirectory::new();
        write_application(directory.path(), false);

        assert!(!directory.path().join("presolve.env.toml").exists());

        let rendered =
            render_application_json(directory.path()).expect("inspection JSON should render");
        let value = parse(&rendered);

        assert_eq!(value["schema"], "presolve.cli");
        assert_eq!(value["schema_version"], "0.1");
        assert_eq!(value["kind"], "application_inspection");
        assert_eq!(value["application"]["name"], "orders");
        assert_eq!(value["description"], "Orders");
        assert_eq!(value["workloads"][0]["name"], "main");
        assert_eq!(value["workloads"][0]["kind"], "component");
        assert_eq!(value["workloads"][0]["implicit"], true);
    }

    #[test]
    fn release_json_exposes_verified_immutable_identity() {
        use presolve_bundle::PresolveBundle;
        use presolve_contract::parse_contract;

        let directory = TestDirectory::new();
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "release-json"
version = "1.0.0"

[resources]
memory_mib = 64
"#,
        )
        .expect("contract should parse");

        let bundle = PresolveBundle::from_component(&contract, b"component bytes")
            .expect("release should build");
        let release = directory.path().join("release-json.presolved");
        fs::write(&release, bundle.encode().expect("release should encode"))
            .expect("release should be written");

        let document = crate::release_inspect::load_release(
            directory.path(),
            Path::new("release-json.presolved"),
        )
        .expect("release should load");

        let value = parse(&render_release_json(&document).expect("release JSON should render"));

        assert_eq!(value["kind"], "release_inspection");
        assert_eq!(value["release"]["bundle_version"], "0.2");
        assert!(
            value["release"]["digest"]
                .as_str()
                .expect("digest should be a string")
                .starts_with("sha256:")
        );
        assert_eq!(value["application"]["name"], "release-json");
        assert_eq!(value["requirements"]["resources"]["memory_mib"], 64);
        assert_eq!(value["workloads"][0]["name"], "main");
        assert_eq!(value["workloads"][0]["kind"], "component");
        assert_eq!(
            value["workloads"][0]["artifacts"][0]["path"],
            "artifacts/main.wasm"
        );
        assert_eq!(
            value["workloads"][0]["artifacts"][0]["media_type"],
            "application/wasm"
        );
        assert!(value["workloads"][0]["oci_image"].is_null());
    }

    #[test]
    fn check_json_renders_stable_problem_fields() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");
        write_application(directory.path(), false);

        let check =
            check_compatibility(directory.path()).expect("compatibility resolution should run");
        let value = parse(&render_check_json(&check).expect("check JSON should render"));

        assert_eq!(value["kind"], "compatibility_check");
        assert_eq!(value["status"], "incompatible");
        assert_eq!(value["problems"][0]["type"], "missing_capability");
        assert_eq!(value["problems"][0]["code"], "PS2001");
        assert_eq!(value["problems"][0]["interface"], "presolve:kv/store");
        assert_eq!(value["problems"][0]["version"], "^0.1");
    }

    #[test]
    fn plan_json_contains_exact_resolver_output() {
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
        write_application(directory.path(), true);

        let check =
            check_compatibility(directory.path()).expect("compatibility resolution should run");
        let value = parse(&render_plan_json(&check).expect("plan JSON should render"));

        assert_eq!(value["kind"], "deployment_plan");
        assert_eq!(value["status"], "resolved");
        assert_eq!(value["problems"], json!([]));
        assert_eq!(
            value["plan"]["bindings"][0]["interface"],
            "presolve:kv/store"
        );
        assert_eq!(
            value["plan"]["bindings"][0]["provider_name"],
            "memory-services"
        );
        assert!(
            value["plan"]["bindings"][0]["provider_id"]
                .as_str()
                .expect("provider id should be a string")
                .starts_with("prv_")
        );
        assert_eq!(
            value["plan"]["bindings"][0]["semantic_contract"]["version"],
            "0.1.0"
        );
        assert_eq!(
            value["plan"]["unbound_optional_capabilities"],
            json!(["presolve:objects/store"])
        );
    }

    #[test]
    fn rejected_plan_json_has_null_plan_and_structured_problems() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");
        write_application(directory.path(), false);

        let check =
            check_compatibility(directory.path()).expect("compatibility resolution should run");
        let value = parse(&render_plan_json(&check).expect("plan JSON should render"));

        assert_eq!(value["status"], "rejected");
        assert!(value["plan"].is_null());
        assert_eq!(value["problems"][0]["code"], "PS2001");
    }
}
