use presolve_contract::parse_contract;
use presolve_core::{EnvironmentId, ProviderId};
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
