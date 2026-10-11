use presolve_capability::{
    OBJECT_STORE_FEATURE_CONDITIONAL_WRITE, OBJECT_STORE_FEATURE_RANGE_READ,
    OBJECT_STORE_FEATURE_USER_METADATA, OBJECT_STORE_INTERFACE,
};
use presolve_contract::{ApplicationContract, parse_contract};
use presolve_core::{EnvironmentId, ProviderId};
use presolve_environment::{inventory_from_specification, parse_environment};
use presolve_resolver::{ResolutionProblem, resolve};
use semver::Version;

const ENVIRONMENT_ID: &str = "env_00000000-0000-4000-8000-000000000001";
const BASIC_OBJECTS_PROVIDER_ID: &str = "prv_00000000-0000-4000-8000-000000000002";
const RICH_OBJECTS_PROVIDER_ID: &str = "prv_00000000-0000-4000-8000-000000000003";

fn application_contract(capabilities: &str, resources: &str) -> ApplicationContract {
    let source = format!(
        r#"
contract_version = "0.1"

[application]
name = "environment-driven-test"
version = "0.0.1"

[[workloads]]
name = "api"
kind = "component"

{capabilities}

[resources]
{resources}

[network]
outbound = "deny"
"#
    );

    parse_contract(&source).expect("application contract should parse")
}

fn environment_specification(
    memory_mib: u64,
    cpu_millis: u32,
    providers: &str,
) -> presolve_environment::EnvironmentSpecification {
    let source = format!(
        r#"
specification_version = "0.1"

[environment]
id = "{ENVIRONMENT_ID}"
name = "local-test"

[resources]
memory_mib = {memory_mib}
cpu_millis = {cpu_millis}

[execution]
workloads = ["component"]

{providers}
"#
    );

    parse_environment(&source).expect("environment specification should parse")
}

#[test]
fn specification_drives_semantic_provider_selection_and_plan() {
    let contract = application_contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
preferred_features = ["user-metadata"]
"#,
        r"
memory_mib = 256
cpu_millis = 500
",
    );

    let specification = environment_specification(
        2_048,
        2_000,
        &format!(
            r#"
[[providers]]
id = "{BASIC_OBJECTS_PROVIDER_ID}"
name = "objects-basic"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["range-read"]

[[providers]]
id = "{RICH_OBJECTS_PROVIDER_ID}"
name = "objects-rich"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["user-metadata", "range-read"]
"#
        ),
    );

    let inventory =
        inventory_from_specification(&specification).expect("inventory should be enriched");
    let report = resolve(&contract, &inventory);

    assert!(
        report.is_resolved(),
        "declarative environment should satisfy the application"
    );

    let plan = report
        .plan()
        .expect("resolved report should contain a plan");
    let expected_environment_id: EnvironmentId =
        ENVIRONMENT_ID.parse().expect("environment id should parse");
    let expected_provider_id: ProviderId = RICH_OBJECTS_PROVIDER_ID
        .parse()
        .expect("provider id should parse");

    assert_eq!(plan.environment_id(), &expected_environment_id);
    assert_eq!(plan.bindings().len(), 1);

    let binding = &plan.bindings()[0];

    assert_eq!(binding.interface(), OBJECT_STORE_INTERFACE);
    assert_eq!(binding.provider_id(), &expected_provider_id);
    assert_eq!(binding.provider_name(), "objects-rich");
    assert_eq!(binding.provider_version(), &Version::new(0, 1, 0));
    assert_eq!(
        binding.negotiated_features(),
        [
            OBJECT_STORE_FEATURE_RANGE_READ,
            OBJECT_STORE_FEATURE_USER_METADATA
        ]
    );

    let semantic_contract = binding
        .semantic_contract()
        .expect("built-in capability should retain semantic contract");

    assert_eq!(semantic_contract.interface(), OBJECT_STORE_INTERFACE);
    assert_eq!(semantic_contract.version(), &Version::new(0, 1, 0));
}

#[test]
fn specification_feature_supply_drives_required_feature_rejection() {
    let contract = application_contract(
        r#"
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["conditional-write"]
"#,
        "",
    );

    let specification = environment_specification(
        1_024,
        1_000,
        &format!(
            r#"
[[providers]]
id = "{BASIC_OBJECTS_PROVIDER_ID}"
name = "objects-basic"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["range-read"]
"#
        ),
    );

    let inventory =
        inventory_from_specification(&specification).expect("inventory should be enriched");
    let report = resolve(&contract, &inventory);

    assert!(!report.is_resolved());

    assert!(matches!(
        report.problems(),
        [
            ResolutionProblem::MissingCapabilityFeatures {
                interface,
                required_features,
                available_features,
                ..
            }
        ] if interface == OBJECT_STORE_INTERFACE
            && required_features.len() == 1
            && required_features[0] == OBJECT_STORE_FEATURE_CONDITIONAL_WRITE
            && available_features.len() == 1
            && available_features[0] == OBJECT_STORE_FEATURE_RANGE_READ
    ));

    assert_eq!(report.problems()[0].code(), "PS2004");
}

#[test]
fn specification_resource_capacity_drives_rejection() {
    let contract = application_contract(
        "",
        r"
memory_mib = 512
cpu_millis = 250
",
    );

    let specification = environment_specification(256, 1_000, "");
    let inventory =
        inventory_from_specification(&specification).expect("inventory should be enriched");
    let report = resolve(&contract, &inventory);

    assert!(!report.is_resolved());

    assert_eq!(
        report.problems(),
        [ResolutionProblem::InsufficientMemory {
            requested_mib: 512,
            available_mib: 256,
        }]
    );
    assert_eq!(report.problems()[0].code(), "PS2002");
}

#[test]
fn specification_can_leave_optional_capability_unbound() {
    let contract = application_contract(
        r#"
[[capabilities]]
interface = "example:optional/service"
version = "^1.0"
optional = true
"#,
        "",
    );

    let specification = environment_specification(512, 1_000, "");
    let inventory =
        inventory_from_specification(&specification).expect("inventory should be enriched");
    let report = resolve(&contract, &inventory);

    assert!(report.is_resolved());

    let plan = report
        .plan()
        .expect("resolved report should contain a plan");

    assert_eq!(
        plan.unbound_optional_capabilities(),
        ["example:optional/service"]
    );
}
