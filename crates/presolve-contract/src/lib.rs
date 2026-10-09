#![forbid(unsafe_code)]

//! `Presolve` Application Contract.
//!
//! The Application Contract describes what an application requires from an
//! execution environment while remaining independent from the infrastructure
//! used to satisfy those requirements.
//!
//! The human-authored representation is [`CONTRACT_FILE_NAME`].

mod model;
mod parse;
mod validate;
mod version;

pub use model::{
    ApplicationContract, ApplicationMetadata, CapabilityRequirement, InterfaceDefinition,
    NetworkPolicy, OutboundNetworkMode, RelationshipDefinition, ResourceRequirements,
    WorkloadDefinition,
};
pub use parse::{CONTRACT_FILE_NAME, ContractDiagnostics, parse_contract};
pub use validate::diagnostic_codes;
pub use version::{CONTRACT_VERSION, ContractVersion, ContractVersionParseError};

pub use presolve_core::{InterfaceKind, WorkloadKind};

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_CONTRACT: &str = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"
"#;

    #[test]
    fn minimal_contract_parses() {
        let contract = parse_contract(MINIMAL_CONTRACT).expect("contract should parse");

        assert_eq!(contract.contract_version(), CONTRACT_VERSION);
        assert_eq!(contract.application().name(), "hello-presolve");
        assert_eq!(contract.application().version().to_string(), "0.0.1");
        assert_eq!(contract.workloads(), []);
        assert_eq!(contract.interfaces(), []);
        assert_eq!(contract.relationships(), []);
        assert_eq!(contract.capabilities(), []);
        assert_eq!(contract.network().outbound(), OutboundNetworkMode::Deny);
    }

    #[test]
    fn full_contract_parses() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "fraud-analysis"
    version = "1.4.2"
    description = "Example Presolve application"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"

    [[capabilities]]
    interface = "presolve:sql/database"
    version = "^1.0"

    [[capabilities]]
    interface = "presolve:secrets/store"
    version = "^1.0"
    optional = true

    [resources]
    memory_mib = 256
    cpu_millis = 500

    [network]
    outbound = "allow_list"
    allow = ["api.stripe.com:443"]
    "#;

        let contract = parse_contract(source).expect("contract should parse");

        assert_eq!(contract.workloads().len(), 2);
        assert_eq!(contract.workloads()[0].name(), "api");
        assert_eq!(contract.workloads()[0].kind(), WorkloadKind::Component);
        assert_eq!(contract.workloads()[1].name(), "worker");
        assert_eq!(contract.interfaces().len(), 1);
        assert_eq!(contract.interfaces()[0].name(), "jobs");
        assert_eq!(
            contract.interfaces()[0].kind(),
            InterfaceKind::RequestResponse
        );

        assert_eq!(contract.relationships().len(), 1);
        assert_eq!(contract.relationships()[0].from(), "api");
        assert_eq!(contract.relationships()[0].to(), "worker");
        assert_eq!(contract.relationships()[0].interface(), "jobs");
        assert_eq!(contract.capabilities().len(), 2);
        assert_eq!(contract.resources().memory_mib(), Some(256));
        assert_eq!(contract.resources().cpu_millis(), Some(500));
        assert_eq!(
            contract.network().outbound(),
            OutboundNetworkMode::AllowList
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let source = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"
imaginary = true
"#;

        let error = parse_contract(source).expect_err("unknown field should fail");

        assert!(has_code(&error, diagnostic_codes::PARSE_ERROR));
    }

    #[test]
    fn unsupported_contract_version_is_rejected() {
        let source = r#"
contract_version = "99.0"

[application]
name = "hello-presolve"
version = "0.0.1"
"#;

        let error = parse_contract(source).expect_err("version should fail");

        assert!(has_code(&error, diagnostic_codes::UNSUPPORTED_VERSION));
    }

    #[test]
    fn invalid_application_name_is_rejected() {
        let source = r#"
contract_version = "0.1"

[application]
name = "Hello Presolve"
version = "0.0.1"
"#;

        let error = parse_contract(source).expect_err("name should fail");

        assert!(has_code(&error, diagnostic_codes::INVALID_APPLICATION_NAME));
    }

    #[test]
    fn invalid_workload_name_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "API Server"
    kind = "component"
    "#;

        let error = parse_contract(source).expect_err("invalid workload name should fail");

        assert!(has_code(&error, diagnostic_codes::INVALID_WORKLOAD_NAME));
    }

    #[test]
    fn duplicate_workloads_are_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"
    "#;

        let error = parse_contract(source).expect_err("duplicate workload should fail");

        assert!(has_code(&error, diagnostic_codes::DUPLICATE_WORKLOAD));
    }

    #[test]
    fn unsupported_workload_kind_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "native"
    "#;

        let error = parse_contract(source).expect_err("unsupported workload kind should fail");

        assert!(has_code(&error, diagnostic_codes::PARSE_ERROR));
    }

    #[test]
    fn invalid_interface_name_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[interfaces]]
    name = "Jobs API"
    kind = "request_response"
    "#;

        let error = parse_contract(source).expect_err("invalid interface name should fail");

        assert!(has_code(&error, diagnostic_codes::INVALID_INTERFACE_NAME));
    }

    #[test]
    fn duplicate_interfaces_are_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[interfaces]]
    name = "jobs"
    kind = "event"
    "#;

        let error = parse_contract(source).expect_err("duplicate interface should fail");

        assert!(has_code(&error, diagnostic_codes::DUPLICATE_INTERFACE));
    }

    #[test]
    fn unsupported_interface_kind_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[interfaces]]
    name = "jobs"
    kind = "http"
    "#;

        let error = parse_contract(source).expect_err("unsupported interface kind should fail");

        assert!(has_code(&error, diagnostic_codes::PARSE_ERROR));
    }

    #[test]
    fn relationship_unknown_workload_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"
    "#;

        let error = parse_contract(source).expect_err("unknown workload should fail");

        assert!(has_code(
            &error,
            diagnostic_codes::UNKNOWN_RELATIONSHIP_WORKLOAD
        ));
    }

    #[test]
    fn relationship_unknown_interface_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"
    "#;

        let error = parse_contract(source).expect_err("unknown interface should fail");

        assert!(has_code(
            &error,
            diagnostic_codes::UNKNOWN_RELATIONSHIP_INTERFACE
        ));
    }

    #[test]
    fn self_relationship_is_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "event"

    [[relationships]]
    from = "worker"
    to = "worker"
    interface = "jobs"
    "#;

        let error = parse_contract(source).expect_err("self relationship should fail");

        assert!(has_code(&error, diagnostic_codes::SELF_RELATIONSHIP));
    }

    #[test]
    fn duplicate_relationships_are_rejected() {
        let source = r#"
    contract_version = "0.1"

    [application]
    name = "example"
    version = "1.0.0"

    [[workloads]]
    name = "api"
    kind = "component"

    [[workloads]]
    name = "worker"
    kind = "component"

    [[interfaces]]
    name = "jobs"
    kind = "request_response"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"

    [[relationships]]
    from = "api"
    to = "worker"
    interface = "jobs"
    "#;

        let error = parse_contract(source).expect_err("duplicate relationship should fail");

        assert!(has_code(&error, diagnostic_codes::DUPLICATE_RELATIONSHIP));
    }

    #[test]
    fn invalid_capability_interface_is_rejected() {
        let source = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"

[[capabilities]]
interface = "database"
version = "^1.0"
"#;

        let error = parse_contract(source).expect_err("interface should fail");

        assert!(has_code(
            &error,
            diagnostic_codes::INVALID_CAPABILITY_INTERFACE
        ));
    }

    #[test]
    fn duplicate_capabilities_are_rejected() {
        let source = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"

[[capabilities]]
interface = "presolve:kv/store"
version = "^1.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^2.0"
"#;

        let error = parse_contract(source).expect_err("duplicate should fail");

        assert!(has_code(&error, diagnostic_codes::DUPLICATE_CAPABILITY));
    }

    #[test]
    fn zero_resource_requirement_is_rejected() {
        let source = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"

[resources]
memory_mib = 0
"#;

        let error = parse_contract(source).expect_err("resource should fail");

        assert!(has_code(
            &error,
            diagnostic_codes::INVALID_RESOURCE_REQUIREMENT
        ));
    }

    #[test]
    fn allow_list_requires_targets() {
        let source = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"

[network]
outbound = "allow_list"
"#;

        let error = parse_contract(source).expect_err("network should fail");

        assert!(has_code(&error, diagnostic_codes::INVALID_NETWORK_POLICY));
    }

    #[test]
    fn network_target_must_not_be_url() {
        let source = r#"
contract_version = "0.1"

[application]
name = "hello-presolve"
version = "0.0.1"

[network]
outbound = "allow_list"
allow = ["https://example.com/api"]
"#;

        let error = parse_contract(source).expect_err("target should fail");

        assert!(has_code(&error, diagnostic_codes::INVALID_NETWORK_POLICY));
    }

    #[test]
    fn normalized_toml_round_trips() {
        let contract = parse_contract(MINIMAL_CONTRACT).expect("contract should parse");

        let encoded = contract.to_toml().expect("contract should serialize");

        let decoded = parse_contract(&encoded).expect("serialized contract should parse");

        assert_eq!(decoded, contract);
    }

    fn has_code(diagnostics: &ContractDiagnostics, number: u16) -> bool {
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code().number() == number)
    }
}
