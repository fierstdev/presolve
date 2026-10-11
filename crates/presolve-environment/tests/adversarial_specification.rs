use presolve_environment::{
    EnvironmentEncodeError, EnvironmentInventoryError, EnvironmentParseError,
    EnvironmentSpecification, EnvironmentSpecificationError, encode_environment,
    inventory_from_specification, parse_environment,
};
use serde_json::json;

const ENVIRONMENT_ID: &str = "env_00000000-0000-4000-8000-000000000001";
const PROVIDER_A_ID: &str = "prv_00000000-0000-4000-8000-000000000002";
const PROVIDER_B_ID: &str = "prv_00000000-0000-4000-8000-000000000003";

fn base_environment(providers: &str) -> String {
    format!(
        r#"
specification_version = "0.1"

[environment]
id = "{ENVIRONMENT_ID}"
name = "adversarial-test"

[resources]
memory_mib = 2048
cpu_millis = 2000

[execution]
workloads = ["component"]

{providers}
"#
    )
}

#[test]
fn duplicate_provider_id_from_toml_is_rejected() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-a"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"

[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-b"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
"#
    ));

    let error = parse_environment(&source).expect_err("duplicate provider id must fail");

    assert!(matches!(
        error,
        EnvironmentParseError::Validation(EnvironmentSpecificationError::DuplicateProviderId(_))
    ));
}

#[test]
fn duplicate_provider_name_from_toml_is_rejected() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "same-provider"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"

[[providers]]
id = "{PROVIDER_B_ID}"
name = "same-provider"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
"#
    ));

    let error = parse_environment(&source).expect_err("duplicate provider name must fail");

    assert!(matches!(
        error,
        EnvironmentParseError::Validation(
            EnvironmentSpecificationError::DuplicateProviderName(name)
        ) if name == "same-provider"
    ));
}

#[test]
fn provider_without_capabilities_from_toml_is_rejected() {
    let source = base_environment(&format!(
        r#"
    [[providers]]
    id = "{PROVIDER_A_ID}"
    name = "empty-provider"
    capabilities = []
    "#
    ));

    let error = parse_environment(&source).expect_err("empty provider must fail");

    assert!(matches!(
        error,
        EnvironmentParseError::Validation(
            EnvironmentSpecificationError::MissingProviderCapabilities(name)
        ) if name == "empty-provider"
    ));
}

#[test]
fn duplicate_exact_capability_from_toml_is_rejected() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "duplicate-capability"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
"#
    ));

    let error = parse_environment(&source).expect_err("duplicate exact capability must fail");

    assert!(matches!(
        error,
        EnvironmentParseError::Validation(
            EnvironmentSpecificationError::DuplicateProviderCapability {
                interface,
                ..
            }
        ) if interface == "presolve:objects/store"
    ));
}

#[test]
fn duplicate_workload_kind_from_toml_is_rejected() {
    let source = base_environment("").replace(
        r#"workloads = ["component"]"#,
        r#"workloads = ["component", "component"]"#,
    );

    let error = parse_environment(&source).expect_err("duplicate workload kind must fail");

    assert!(matches!(
        error,
        EnvironmentParseError::Validation(EnvironmentSpecificationError::DuplicateWorkloadKind(_))
    ));
}

#[test]
fn malformed_environment_id_is_rejected_during_deserialization() {
    let source = base_environment("").replace(ENVIRONMENT_ID, "env_not-a-valid-uuid");

    let error = parse_environment(&source).expect_err("malformed environment id must fail");

    assert!(matches!(error, EnvironmentParseError::Toml(_)));
}

#[test]
fn malformed_provider_id_is_rejected_during_deserialization() {
    let source = base_environment(
        r#"
[[providers]]
id = "prv_not-a-valid-uuid"
name = "bad-provider"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
"#,
    );

    let error = parse_environment(&source).expect_err("malformed provider id must fail");

    assert!(matches!(error, EnvironmentParseError::Toml(_)));
}

#[test]
fn malformed_capability_version_is_rejected_during_deserialization() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "bad-version"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "not-semver"
"#
    ));

    let error = parse_environment(&source).expect_err("malformed semver must fail");

    assert!(matches!(error, EnvironmentParseError::Toml(_)));
}

#[test]
fn unknown_provider_field_is_rejected() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-a"
endpoint = "https://should-not-be-here.invalid"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
"#
    ));

    let error = parse_environment(&source).expect_err("unknown provider field must fail");

    assert!(matches!(error, EnvironmentParseError::Toml(_)));
}

#[test]
fn unknown_capability_field_is_rejected() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-a"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
implementation = "s3"
"#
    ));

    let error = parse_environment(&source).expect_err("unknown capability field must fail");

    assert!(matches!(error, EnvironmentParseError::Toml(_)));
}

#[test]
fn semantically_equivalent_documents_encode_identically() {
    let first = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_B_ID}"
name = "provider-b"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["user-metadata", "range-read"]

[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-a"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["range-read", "user-metadata"]

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
"#
    ));

    let second = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-a"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["user-metadata", "range-read"]

[[providers]]
id = "{PROVIDER_B_ID}"
name = "provider-b"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["range-read", "user-metadata"]
"#
    ));

    let first = parse_environment(&first).expect("first document should parse");
    let second = parse_environment(&second).expect("second document should parse");

    assert_eq!(first, second);
    assert_eq!(
        encode_environment(&first).expect("first document should encode"),
        encode_environment(&second).expect("second document should encode"),
    );
}

#[test]
fn normalized_encoding_is_idempotent() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "provider-a"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["user-metadata", "range-read"]
"#
    ));

    let specification = parse_environment(&source).expect("document should parse");
    let first = encode_environment(&specification).expect("document should encode");
    let reparsed = parse_environment(&first).expect("encoded document should parse");
    let second = encode_environment(&reparsed).expect("document should re-encode");

    assert_eq!(second, first);
}

#[test]
fn direct_deserialization_cannot_bypass_encode_validation() {
    let value = json!({
        "specification_version": "0.1",
        "environment": {
            "id": ENVIRONMENT_ID,
            "name": "adversarial-test"
        },
        "resources": {
            "memory_mib": 2048,
            "cpu_millis": 2000
        },
        "execution": {
            "workloads": ["component"]
        },
        "providers": [{
            "id": PROVIDER_A_ID,
            "name": "provider-a",
            "capabilities": [{
                "interface": "presolve:objects/store",
                "version": "0.1.0",
                "features": ["range-read", "range-read"]
            }]
        }]
    });

    let unvalidated: EnvironmentSpecification =
        serde_json::from_value(value).expect("raw serde should construct the shape");

    let error = encode_environment(&unvalidated)
        .expect_err("encoding must revalidate directly deserialized values");

    assert!(matches!(
        error,
        EnvironmentEncodeError::Validation(
            EnvironmentSpecificationError::DuplicateCapabilityFeature { .. }
        )
    ));
}

#[test]
fn direct_deserialization_cannot_bypass_inventory_validation() {
    let value = json!({
        "specification_version": "0.1",
        "environment": {
            "id": ENVIRONMENT_ID,
            "name": "adversarial-test"
        },
        "resources": {
            "memory_mib": 2048,
            "cpu_millis": 2000
        },
        "execution": {
            "workloads": ["component", "component"]
        },
        "providers": []
    });

    let unvalidated: EnvironmentSpecification =
        serde_json::from_value(value).expect("raw serde should construct the shape");

    let error = inventory_from_specification(&unvalidated)
        .expect_err("inventory conversion must revalidate directly deserialized values");

    assert!(matches!(
        error,
        EnvironmentInventoryError::Specification(
            EnvironmentSpecificationError::DuplicateWorkloadKind(_)
        )
    ));
}

#[test]
fn raw_spec_with_unknown_builtin_version_fails_during_enrichment() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "future-objects"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "9.0.0"
"#
    ));

    let specification = parse_environment(&source).expect("schema-valid document should parse");
    let error = inventory_from_specification(&specification)
        .expect_err("unknown built-in semantic version must not degrade to generic");

    assert!(matches!(
        error,
        EnvironmentInventoryError::UnsupportedKnownCapabilityVersion {
            interface,
            ..
        } if interface == "presolve:objects/store"
    ));
}

#[test]
fn raw_custom_feature_claim_fails_during_enrichment() {
    let source = base_environment(&format!(
        r#"
[[providers]]
id = "{PROVIDER_A_ID}"
name = "custom-provider"

[[providers.capabilities]]
interface = "example:custom/service"
version = "1.0.0"
features = ["fast-mode"]
"#
    ));

    let specification = parse_environment(&source).expect("schema-valid document should parse");
    let error = inventory_from_specification(&specification)
        .expect_err("unverifiable custom semantic features must fail");

    assert!(matches!(
        error,
        EnvironmentInventoryError::FeaturesRequireKnownContract {
            interface,
            features,
        } if interface == "example:custom/service" && features == ["fast-mode"]
    ));
}
