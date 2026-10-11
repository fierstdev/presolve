use std::sync::Arc;

use presolve_contract::parse_contract;
use presolve_core::ProviderId;
use presolve_environment::{inventory_from_specification, parse_environment};
use presolve_node::{AssemblyError, ProviderRegistry, assemble_runtime};
use presolve_provider_kv_memory::InMemoryKeyValueProvider;
use presolve_provider_objects_memory::InMemoryObjectStoreProvider;
use presolve_provider_sdk::{KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE};
use presolve_resolver::resolve;

const ENVIRONMENT_SOURCE: &str = include_str!("fixtures/materialization.environment.toml");
const PROVIDER_ID: &str = "prv_00000000-0000-4000-8000-000000000102";

fn application() -> presolve_contract::ApplicationContract {
    parse_contract(
        r#"
contract_version = "0.1"

[application]
name = "node-materialization-test"
version = "0.0.1"

[[workloads]]
name = "api"
kind = "component"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"

[resources]
memory_mib = 128
cpu_millis = 100

[network]
outbound = "deny"
"#,
    )
    .expect("application contract should parse")
}

fn resolved_plan() -> presolve_resolver::DeploymentPlan {
    let specification =
        parse_environment(ENVIRONMENT_SOURCE).expect("environment fixture should parse");
    let inventory =
        inventory_from_specification(&specification).expect("environment should enrich");
    let report = resolve(&application(), &inventory);

    assert!(
        report.is_resolved(),
        "environment fixture should resolve the application"
    );

    report
        .plan()
        .expect("resolved report should contain a plan")
        .clone()
}

#[test]
fn environment_file_drives_resolution_and_node_materialization() {
    let provider_id: ProviderId = PROVIDER_ID.parse().expect("provider id should parse");
    let plan = resolved_plan();

    assert_eq!(plan.bindings().len(), 2);

    for binding in plan.bindings() {
        assert_eq!(binding.provider_id(), &provider_id);
        assert!(
            binding.semantic_contract().is_some(),
            "built-in environment capabilities should retain semantic contracts"
        );
    }

    assert_eq!(plan.bindings()[0].interface(), KEY_VALUE_INTERFACE);
    assert_eq!(plan.bindings()[1].interface(), OBJECT_STORE_INTERFACE);

    let mut registry = ProviderRegistry::new();

    registry
        .register_key_value(provider_id, Arc::new(InMemoryKeyValueProvider::new()))
        .expect("KV implementation should register");
    registry
        .register_object_store(provider_id, Arc::new(InMemoryObjectStoreProvider::new()))
        .expect("object implementation should register");

    assemble_runtime(&plan, &registry)
        .expect("resolved environment plan should materialize into a runtime");
}

#[test]
fn materialization_uses_exact_provider_identity_selected_from_environment_file() {
    let selected_provider_id: ProviderId = PROVIDER_ID
        .parse()
        .expect("selected provider id should parse");
    let wrong_provider_id = ProviderId::new();
    let plan = resolved_plan();
    let mut registry = ProviderRegistry::new();

    registry
        .register_key_value(wrong_provider_id, Arc::new(InMemoryKeyValueProvider::new()))
        .expect("wrong KV implementation should still register");
    registry
        .register_object_store(
            wrong_provider_id,
            Arc::new(InMemoryObjectStoreProvider::new()),
        )
        .expect("wrong object implementation should still register");

    let Err(error) = assemble_runtime(&plan, &registry) else {
        panic!("registry entries under another provider id must not materialize the plan");
    };

    assert!(matches!(
        error,
        AssemblyError::MissingProvider {
            provider_id,
            ..
        } if provider_id == selected_provider_id
    ));
}
