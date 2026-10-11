use thiserror::Error;

use crate::{EnvironmentSpecification, EnvironmentSpecificationError};

/// Failure while parsing an Environment Specification document.
#[derive(Debug, Error)]
pub enum EnvironmentParseError {
    /// TOML syntax or deserialization failed.
    #[error("environment specification TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// The parsed document violated Environment Specification semantics.
    #[error("invalid environment specification: {0}")]
    Validation(#[from] EnvironmentSpecificationError),
}

/// Failure while serializing an Environment Specification document.
#[derive(Debug, Error)]
pub enum EnvironmentEncodeError {
    /// The in-memory specification violated Environment Specification semantics.
    #[error("invalid environment specification: {0}")]
    Validation(#[from] EnvironmentSpecificationError),

    /// TOML serialization failed.
    #[error("environment specification TOML serialization error: {0}")]
    Toml(#[from] toml::ser::Error),
}

/// Parses, validates, and normalizes an Environment Specification from TOML.
///
/// Ordering of workload kinds, providers, provider capabilities, and capability
/// features is not environment semantics. Successful parsing therefore returns
/// the normalized representation.
///
/// # Errors
///
/// Returns [`EnvironmentParseError`] for malformed TOML, unknown fields,
/// unsupported enum values, or violated Environment Specification invariants.
pub fn parse_environment(input: &str) -> Result<EnvironmentSpecification, EnvironmentParseError> {
    let mut specification: EnvironmentSpecification = toml::from_str(input)?;

    specification.validate()?;
    specification.normalize();

    Ok(specification)
}

/// Serializes a validated Environment Specification into normalized TOML.
///
/// # Errors
///
/// Returns [`EnvironmentEncodeError`] when the specification is invalid or TOML
/// serialization fails.
pub fn encode_environment(
    specification: &EnvironmentSpecification,
) -> Result<String, EnvironmentEncodeError> {
    let normalized = specification.normalized()?;
    Ok(toml::to_string_pretty(&normalized)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENVIRONMENT_ID: &str = "env_00000000-0000-4000-8000-000000000001";
    const KV_PROVIDER_ID: &str = "prv_00000000-0000-4000-8000-000000000002";
    const OBJECT_PROVIDER_ID: &str = "prv_00000000-0000-4000-8000-000000000003";

    fn canonical_input() -> String {
        format!(
            r#"
specification_version = "0.1"

[environment]
id = "{ENVIRONMENT_ID}"
name = "local-dev"
description = "Local Presolve development environment"

[resources]
memory_mib = 4096
cpu_millis = 4000

[execution]
workloads = ["component"]

[[providers]]
id = "{KV_PROVIDER_ID}"
name = "kv-memory"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"

[[providers]]
id = "{OBJECT_PROVIDER_ID}"
name = "objects-memory"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["range-read", "user-metadata"]
"#
        )
    }

    #[test]
    fn complete_environment_toml_parses() {
        let specification =
            parse_environment(&canonical_input()).expect("environment should parse");

        assert_eq!(specification.environment().name(), "local-dev");
        assert_eq!(specification.resources().memory_mib(), 4096);
        assert_eq!(specification.resources().cpu_millis(), 4000);
        assert_eq!(specification.providers().len(), 2);
        assert_eq!(specification.providers()[0].name(), "kv-memory");
        assert_eq!(specification.providers()[1].name(), "objects-memory");
        assert_eq!(
            specification.providers()[1].capabilities()[0].features(),
            ["range-read", "user-metadata"]
        );
    }

    #[test]
    fn normalized_toml_round_trips() {
        let first = parse_environment(&canonical_input()).expect("environment should parse");
        let encoded = encode_environment(&first).expect("environment should encode");
        let second = parse_environment(&encoded).expect("encoded environment should parse");

        assert_eq!(second, first);
        assert_eq!(
            encode_environment(&second).expect("second environment should encode"),
            encoded
        );
    }

    #[test]
    fn semantic_collection_order_does_not_change_normalized_document() {
        let first = parse_environment(&canonical_input()).expect("first environment should parse");

        let reordered = format!(
            r#"
specification_version = "0.1"

[environment]
id = "{ENVIRONMENT_ID}"
name = "local-dev"
description = "Local Presolve development environment"

[resources]
memory_mib = 4096
cpu_millis = 4000

[execution]
workloads = ["component"]

[[providers]]
id = "{OBJECT_PROVIDER_ID}"
name = "objects-memory"

[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["user-metadata", "range-read"]

[[providers]]
id = "{KV_PROVIDER_ID}"
name = "kv-memory"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
"#
        );

        let second = parse_environment(&reordered).expect("second environment should parse");

        assert_eq!(second, first);
        assert_eq!(
            encode_environment(&second).expect("second environment should encode"),
            encode_environment(&first).expect("first environment should encode")
        );
    }

    #[test]
    fn unknown_root_field_is_rejected() {
        let input = canonical_input().replacen(
            "specification_version = \"0.1\"",
            "specification_version = \"0.1\"\nunknown = true",
            1,
        );

        let error = parse_environment(&input).expect_err("unknown field should fail");

        assert!(matches!(error, EnvironmentParseError::Toml(_)));
    }

    #[test]
    fn unknown_nested_field_is_rejected() {
        let input = canonical_input().replacen(
            "name = \"local-dev\"",
            "name = \"local-dev\"\nregion = \"local\"",
            1,
        );

        let error = parse_environment(&input).expect_err("unknown nested field should fail");

        assert!(matches!(error, EnvironmentParseError::Toml(_)));
    }

    #[test]
    fn invalid_semantic_document_is_rejected_after_deserialization() {
        let input = canonical_input().replacen(
            "features = [\"range-read\", \"user-metadata\"]",
            "features = [\"range-read\", \"range-read\"]",
            1,
        );

        let error = parse_environment(&input).expect_err("duplicate feature should fail");

        assert!(matches!(
            error,
            EnvironmentParseError::Validation(
                EnvironmentSpecificationError::DuplicateCapabilityFeature { .. }
            )
        ));
    }

    #[test]
    fn unsupported_schema_version_is_rejected_after_deserialization() {
        let input = canonical_input().replacen(
            "specification_version = \"0.1\"",
            "specification_version = \"9.0\"",
            1,
        );

        let error = parse_environment(&input).expect_err("unsupported version should fail");

        assert!(matches!(
            error,
            EnvironmentParseError::Validation(
                EnvironmentSpecificationError::UnsupportedSpecificationVersion { .. }
            )
        ));
    }

    #[test]
    fn unsupported_workload_kind_is_rejected_during_deserialization() {
        let input = canonical_input().replacen(
            "workloads = [\"component\"]",
            "workloads = [\"container\"]",
            1,
        );

        let error = parse_environment(&input).expect_err("unknown workload kind should fail");

        assert!(matches!(error, EnvironmentParseError::Toml(_)));
    }
}
