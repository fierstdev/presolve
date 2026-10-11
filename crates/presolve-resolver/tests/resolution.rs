use presolve_capability::{OBJECT_STORE_INTERFACE, object_store_contract};
use presolve_contract::parse_contract;
use presolve_core::{EnvironmentId, ProviderId, WorkloadKind};
use presolve_resolver::{
    EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor,
    ResolutionProblem, resolve,
};
use semver::Version;

fn contract(capabilities: &str, resources: &str) -> presolve_contract::ApplicationContract {
    let source = format!(
        r#"
contract_version = "0.1"

[application]
name = "resolver-test"
version = "0.0.1"

{capabilities}

[resources]
{resources}

[network]
outbound = "deny"
"#
    );

    parse_contract(&source).expect("test contract should parse")
}

fn workload_contract(workloads: &str) -> presolve_contract::ApplicationContract {
    let source = format!(
        r#"
contract_version = "0.1"

[application]
name = "resolver-workload-test"
version = "0.0.1"

{workloads}

[resources]

[network]
outbound = "deny"
"#
    );

    parse_contract(&source).expect("workload test contract should parse")
}

#[test]
fn resolves_supported_component_workload() {
    let contract = workload_contract(
        r#"
[[workloads]]
name = "api"
kind = "component"
"#,
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        Vec::new(),
    );

    let report = resolve(&contract, &environment);

    assert!(
        report.is_resolved(),
        "component workload should resolve in a component-capable environment"
    );
}

#[test]
fn rejects_unsupported_workload_execution_kind() {
    let contract = workload_contract(
        r#"
[[workloads]]
name = "api"
kind = "component"
"#,
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        Vec::new(),
    )
    .with_supported_workloads(Vec::new());

    let report = resolve(&contract, &environment);

    assert!(!report.is_resolved());

    assert!(matches!(
        report.problems(),
        [
            ResolutionProblem::UnsupportedWorkload {
                workload,
                kind: WorkloadKind::Component,
                supported_kinds,
            }
        ] if workload == "api" && supported_kinds.is_empty()
    ));

    assert_eq!(report.problems()[0].code(), "PS2005");
}

fn kv_provider(version: &str) -> ProviderDescriptor {
    ProviderDescriptor::new(
        ProviderId::new(),
        "memory-kv",
        vec![ProvidedCapability::new(
            "presolve:kv/store",
            Version::parse(version).expect("version should parse"),
        )],
    )
}

fn object_provider(provider_id: ProviderId, name: &str) -> ProviderDescriptor {
    ProviderDescriptor::new(
        provider_id,
        name,
        vec![ProvidedCapability::from_contract(
            object_store_contract().expect("object-store contract should be valid"),
        )],
    )
}

#[test]
fn resolves_matching_required_capability() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        r"
memory_mib = 64
cpu_millis = 100
",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![kv_provider("0.1.7")],
    );

    let report = resolve(&contract, &environment);

    assert!(report.is_resolved(), "matching environment should resolve");

    let plan = report.plan().expect("resolved report should contain plan");

    assert_eq!(plan.bindings().len(), 1);
    assert_eq!(plan.bindings()[0].interface(), "presolve:kv/store");
    assert_eq!(
        plan.bindings()[0].provider_version(),
        &Version::parse("0.1.7").expect("version should parse")
    );
}

#[test]
fn resolves_required_object_storage_capability() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
"#,
        "",
    );

    let provider_id = ProviderId::new();
    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![object_provider(provider_id, "memory-objects")],
    );

    let report = resolve(&contract, &environment);

    assert!(
        report.is_resolved(),
        "canonical object-storage capability should resolve"
    );

    let plan = report.plan().expect("resolved report should contain plan");

    assert_eq!(plan.bindings().len(), 1);
    assert_eq!(plan.bindings()[0].interface(), OBJECT_STORE_INTERFACE);
    assert_eq!(plan.bindings()[0].provider_id(), &provider_id);
    assert_eq!(plan.bindings()[0].provider_name(), "memory-objects");

    let expected_contract = object_store_contract().expect("object-store contract should be valid");

    assert_eq!(
        plan.bindings()[0].provider_version(),
        expected_contract.version()
    );
    assert_eq!(
        plan.bindings()[0].semantic_contract(),
        Some(&expected_contract)
    );
}

#[test]
fn object_storage_requirement_does_not_bind_unrelated_capability() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![kv_provider("0.1.9")],
    );

    let report = resolve(&contract, &environment);

    assert!(!report.is_resolved());
    assert!(matches!(
        report.problems(),
        [ResolutionProblem::MissingCapability { interface, .. }]
            if interface == OBJECT_STORE_INTERFACE
    ));
}

#[test]
fn object_storage_provider_is_selected_by_capability_identity() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
"#,
        "",
    );

    let object_provider_id = ProviderId::new();
    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![
            kv_provider("0.1.9"),
            object_provider(object_provider_id, "memory-objects"),
        ],
    );

    let report = resolve(&contract, &environment);
    let plan = report
        .plan()
        .expect("object-storage environment should resolve");

    assert_eq!(plan.bindings().len(), 1);
    assert_eq!(plan.bindings()[0].interface(), OBJECT_STORE_INTERFACE);
    assert_eq!(plan.bindings()[0].provider_id(), &object_provider_id);
}

#[test]
fn rejects_missing_required_capability() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        Vec::new(),
    );

    let report = resolve(&contract, &environment);

    assert!(!report.is_resolved());

    assert!(matches!(
        report.problems(),
        [
            ResolutionProblem::MissingCapability {
                interface,
                ..
            }
        ] if interface == "presolve:kv/store"
    ));

    assert_eq!(report.problems()[0].code(), "PS2001");
}

#[test]
fn allows_missing_optional_capability() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
optional = true
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        Vec::new(),
    );

    let report = resolve(&contract, &environment);

    assert!(report.is_resolved());

    let plan = report.plan().expect("resolved report should contain plan");

    assert_eq!(plan.unbound_optional_capabilities(), ["presolve:kv/store"]);
}

#[test]
fn rejects_incompatible_provider_version() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![kv_provider("0.2.0")],
    );

    let report = resolve(&contract, &environment);

    assert!(!report.is_resolved());

    assert!(matches!(
        report.problems(),
        [ResolutionProblem::MissingCapability { .. }]
    ));
}

#[test]
fn chooses_highest_compatible_provider_version() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![
            kv_provider("0.1.2"),
            kv_provider("0.1.9"),
            kv_provider("0.1.4"),
        ],
    );

    let report = resolve(&contract, &environment);

    let plan = report.plan().expect("environment should resolve");

    assert_eq!(
        plan.bindings()[0].provider_version(),
        &Version::parse("0.1.9").expect("version should parse")
    );
}

#[test]
fn rejects_insufficient_memory() {
    let contract = contract(
        "",
        r"
        memory_mib = 256
        ",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(128, 1_000),
        Vec::new(),
    );

    let report = resolve(&contract, &environment);

    assert!(matches!(
        report.problems(),
        [ResolutionProblem::InsufficientMemory {
            requested_mib: 256,
            available_mib: 128,
        }]
    ));

    assert_eq!(report.problems()[0].code(), "PS2002");
}

#[test]
fn rejects_insufficient_cpu() {
    let contract = contract(
        "",
        r"
        cpu_millis = 500
        ",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 250),
        Vec::new(),
    );

    let report = resolve(&contract, &environment);

    assert!(matches!(
        report.problems(),
        [ResolutionProblem::InsufficientCpu {
            requested_millis: 500,
            available_millis: 250,
        }]
    ));

    assert_eq!(report.problems()[0].code(), "PS2003");
}

#[test]
fn reports_multiple_unsatisfied_requirements() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        r"
memory_mib = 256
cpu_millis = 500
",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(128, 250),
        Vec::new(),
    );

    let report = resolve(&contract, &environment);

    assert!(!report.is_resolved());
    assert_eq!(report.problems().len(), 3);

    assert_eq!(
        report
            .problems()
            .iter()
            .map(ResolutionProblem::code)
            .collect::<Vec<_>>(),
        ["PS2002", "PS2003", "PS2001",]
    );
}

#[test]
fn resolves_provider_with_required_semantic_feature() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
"#,
        "",
    );

    let provider_id = ProviderId::new();
    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![object_provider_with_features(
            provider_id,
            "range-objects",
            &["range-read"],
        )],
    );

    let report = resolve(&contract, &environment);
    let plan = report
        .plan()
        .expect("required object semantic feature should resolve");

    assert_eq!(plan.bindings()[0].provider_id(), &provider_id);
    assert_eq!(plan.bindings()[0].negotiated_features(), ["range-read"]);
}

#[test]
fn rejects_provider_missing_required_semantic_feature() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["multipart-upload"]
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![object_provider(ProviderId::new(), "memory-objects")],
    );

    let report = resolve(&contract, &environment);

    assert!(!report.is_resolved());
    assert!(matches!(
        report.problems(),
        [ResolutionProblem::MissingCapabilityFeatures {
            interface,
            required_features,
            available_features,
            ..
        }] if interface == OBJECT_STORE_INTERFACE
            && required_features == &["multipart-upload"]
            && available_features.is_empty()
    ));
    assert_eq!(report.problems()[0].code(), "PS2004");
}

#[test]
fn preferred_semantic_feature_outranks_higher_provider_version() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
preferred_features = ["range-read"]
"#,
        "",
    );

    let preferred_provider_id = ProviderId::new();
    let higher_version_provider_id = ProviderId::new();

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![
            ProviderDescriptor::new(
                higher_version_provider_id,
                "generic-objects",
                vec![ProvidedCapability::new(
                    OBJECT_STORE_INTERFACE,
                    Version::new(0, 1, 9),
                )],
            ),
            object_provider_with_features(
                preferred_provider_id,
                "semantic-objects",
                &["range-read"],
            ),
        ],
    );

    let report = resolve(&contract, &environment);
    let plan = report
        .plan()
        .expect("preferred semantic feature should rank providers");

    assert_eq!(plan.bindings()[0].provider_id(), &preferred_provider_id);
    assert_eq!(plan.bindings()[0].negotiated_features(), ["range-read"]);
}

#[test]
fn optional_capability_with_unsatisfied_required_feature_remains_unbound() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
optional = true
required_features = ["multipart-upload"]
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![object_provider(ProviderId::new(), "memory-objects")],
    );

    let report = resolve(&contract, &environment);
    let plan = report
        .plan()
        .expect("optional semantic mismatch should not reject deployment");

    assert_eq!(plan.bindings(), []);
    assert_eq!(
        plan.unbound_optional_capabilities(),
        [OBJECT_STORE_INTERFACE]
    );
}

fn object_provider_with_features(
    provider_id: ProviderId,
    name: &str,
    features: &[&str],
) -> ProviderDescriptor {
    let capability = ProvidedCapability::from_contract(
        object_store_contract().expect("object-store contract should be valid"),
    )
    .with_supported_features(features.iter().copied())
    .expect("test feature advertisement should be valid");

    ProviderDescriptor::new(provider_id, name, vec![capability])
}

#[test]
fn canonical_feature_vocabulary_is_not_inferred_as_provider_support() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
"#,
        "",
    );

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![object_provider(ProviderId::new(), "baseline-objects")],
    );

    let report = resolve(&contract, &environment);

    assert!(matches!(
        report.problems(),
        [ResolutionProblem::MissingCapabilityFeatures {
            required_features,
            available_features,
            ..
        }] if required_features == &["range-read"] && available_features.is_empty()
    ));
}

#[test]
fn required_semantic_feature_outranks_higher_version_without_feature() {
    let contract = contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
"#,
        "",
    );

    let semantic_provider_id = ProviderId::new();

    let environment = EnvironmentInventory::new(
        EnvironmentId::new(),
        EnvironmentResources::new(512, 1_000),
        vec![
            ProviderDescriptor::new(
                ProviderId::new(),
                "higher-version-baseline",
                vec![ProvidedCapability::new(
                    OBJECT_STORE_INTERFACE,
                    Version::new(0, 1, 9),
                )],
            ),
            object_provider_with_features(semantic_provider_id, "range-provider", &["range-read"]),
        ],
    );

    let report = resolve(&contract, &environment);
    let plan = report
        .plan()
        .expect("provider satisfying required semantics should resolve");

    assert_eq!(plan.bindings()[0].provider_id(), &semantic_provider_id);
    assert_eq!(plan.bindings()[0].negotiated_features(), ["range-read"]);
}
