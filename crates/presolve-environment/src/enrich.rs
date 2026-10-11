use presolve_capability::{
    CapabilityContractError, canonical_contract_for, is_known_capability_interface,
};
use semver::Version;
use thiserror::Error;

use crate::{
    CapabilityFeatureError, EnvironmentInventory, EnvironmentResources, EnvironmentSpecification,
    EnvironmentSpecificationError, ProvidedCapability, ProvidedCapabilitySpecification,
    ProviderDescriptor,
};

/// Failure while enriching an authored Environment Specification into the
/// normalized resolver-facing inventory.
#[derive(Debug, Error)]
pub enum EnvironmentInventoryError {
    /// The source Environment Specification is invalid.
    #[error("invalid environment specification: {0}")]
    Specification(#[from] EnvironmentSpecificationError),

    /// A built-in canonical capability contract could not be constructed.
    #[error("invalid canonical capability contract: {0}")]
    CapabilityContract(#[from] CapabilityContractError),

    /// A Presolve-owned capability interface uses a version whose semantics are
    /// not known to this Presolve release.
    #[error("capability `{interface}@{version}` is not understood by this Presolve release")]
    UnsupportedKnownCapabilityVersion {
        /// Built-in capability interface.
        interface: String,

        /// Exact unsupported capability version.
        version: Version,
    },

    /// Semantic feature claims require a canonical semantic contract.
    #[error("capability `{interface}` cannot advertise semantic features without a known contract")]
    FeaturesRequireKnownContract {
        /// Capability interface without a known contract.
        interface: String,

        /// Features that could not be semantically validated.
        features: Vec<String>,
    },

    /// A provider advertised a feature outside a known capability vocabulary.
    #[error(
        "provider `{provider}` capability `{interface}` has invalid semantic feature advertisement: {source}"
    )]
    InvalidFeatureAdvertisement {
        /// Provider containing the invalid advertisement.
        provider: String,

        /// Capability interface containing the invalid advertisement.
        interface: String,

        /// Feature-advertisement validation failure.
        #[source]
        source: CapabilityFeatureError,
    },
}

/// Converts an authored Environment Specification into normalized resolver input.
///
/// Built-in Presolve capabilities are enriched with their canonical semantic
/// contracts. Custom capabilities remain generic when they make no semantic
/// feature claims.
///
/// # Errors
///
/// Returns [`EnvironmentInventoryError`] when the source specification is
/// invalid, a built-in capability version is unknown, or feature claims cannot
/// be validated against a canonical contract.
pub fn inventory_from_specification(
    specification: &EnvironmentSpecification,
) -> Result<EnvironmentInventory, EnvironmentInventoryError> {
    specification.validate()?;

    let providers = specification
        .providers()
        .iter()
        .map(enrich_provider)
        .collect::<Result<Vec<_>, _>>()?;

    let inventory = EnvironmentInventory::new(
        *specification.environment().id(),
        EnvironmentResources::new(
            specification.resources().memory_mib(),
            specification.resources().cpu_millis(),
        ),
        providers,
    )
    .with_supported_workloads(specification.execution().workloads().to_vec());

    Ok(inventory)
}

fn enrich_provider(
    provider: &crate::ProviderSpecification,
) -> Result<ProviderDescriptor, EnvironmentInventoryError> {
    let capabilities = provider
        .capabilities()
        .iter()
        .map(|capability| enrich_capability(provider.name(), capability))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ProviderDescriptor::new(
        *provider.id(),
        provider.name(),
        capabilities,
    ))
}

fn enrich_capability(
    provider: &str,
    capability: &ProvidedCapabilitySpecification,
) -> Result<ProvidedCapability, EnvironmentInventoryError> {
    let semantic_contract = canonical_contract_for(capability.interface(), capability.version())?;

    match semantic_contract {
        Some(contract) => {
            let enriched = ProvidedCapability::from_contract(contract);

            if capability.features().is_empty() {
                return Ok(enriched);
            }

            enriched
                .with_supported_features(capability.features().iter().cloned())
                .map_err(
                    |source| EnvironmentInventoryError::InvalidFeatureAdvertisement {
                        provider: provider.to_owned(),
                        interface: capability.interface().to_owned(),
                        source,
                    },
                )
        }
        None if is_known_capability_interface(capability.interface()) => Err(
            EnvironmentInventoryError::UnsupportedKnownCapabilityVersion {
                interface: capability.interface().to_owned(),
                version: capability.version().clone(),
            },
        ),
        None if !capability.features().is_empty() => {
            Err(EnvironmentInventoryError::FeaturesRequireKnownContract {
                interface: capability.interface().to_owned(),
                features: capability.features().to_vec(),
            })
        }
        None => Ok(ProvidedCapability::new(
            capability.interface(),
            capability.version().clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use presolve_capability::{
        KEY_VALUE_INTERFACE, OBJECT_STORE_FEATURE_RANGE_READ, OBJECT_STORE_FEATURE_USER_METADATA,
        OBJECT_STORE_INTERFACE,
    };
    use presolve_core::WorkloadKind;

    use crate::parse_environment;

    use super::*;

    const ENVIRONMENT_ID: &str = "env_00000000-0000-4000-8000-000000000001";
    const PROVIDER_ID: &str = "prv_00000000-0000-4000-8000-000000000002";

    fn source(capability: &str) -> String {
        format!(
            r#"
specification_version = "0.1"

[environment]
id = "{ENVIRONMENT_ID}"
name = "local-dev"

[resources]
memory_mib = 2048
cpu_millis = 2000

[execution]
workloads = ["component"]

[[providers]]
id = "{PROVIDER_ID}"
name = "test-provider"

{capability}
"#
        )
    }

    #[test]
    fn builtin_object_capability_is_enriched_with_semantic_contract() {
        let specification = parse_environment(&source(
            r#"
[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["user-metadata", "range-read"]
"#,
        ))
        .expect("environment should parse");

        let inventory =
            inventory_from_specification(&specification).expect("inventory should build");

        assert_eq!(inventory.resources().memory_mib(), 2048);
        assert_eq!(inventory.resources().cpu_millis(), 2000);
        assert_eq!(inventory.supported_workloads(), [WorkloadKind::Component]);

        let capability = &inventory.providers()[0].capabilities()[0];

        assert_eq!(capability.interface(), OBJECT_STORE_INTERFACE);
        assert!(capability.semantic_contract().is_some());
        assert_eq!(
            capability.supported_features(),
            [
                OBJECT_STORE_FEATURE_RANGE_READ,
                OBJECT_STORE_FEATURE_USER_METADATA
            ]
        );
    }

    #[test]
    fn builtin_kv_capability_is_enriched_with_semantic_contract() {
        let specification = parse_environment(&source(
            r#"
[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
"#,
        ))
        .expect("environment should parse");

        let inventory =
            inventory_from_specification(&specification).expect("inventory should build");

        let capability = &inventory.providers()[0].capabilities()[0];

        assert_eq!(capability.interface(), KEY_VALUE_INTERFACE);
        assert!(capability.semantic_contract().is_some());
        let expected: &[String] = &[];
        assert_eq!(capability.supported_features(), expected);
    }

    #[test]
    fn custom_baseline_capability_remains_generic() {
        let specification = parse_environment(&source(
            r#"
[[providers.capabilities]]
interface = "example:custom/service"
version = "1.2.3"
"#,
        ))
        .expect("environment should parse");

        let inventory =
            inventory_from_specification(&specification).expect("inventory should build");

        let capability = &inventory.providers()[0].capabilities()[0];

        assert_eq!(capability.interface(), "example:custom/service");
        assert_eq!(capability.version(), &Version::new(1, 2, 3));
        assert!(capability.semantic_contract().is_none());
        let expected: &[String] = &[];
        assert_eq!(capability.supported_features(), expected);
    }

    #[test]
    fn custom_capability_cannot_claim_unverifiable_features() {
        let specification = parse_environment(&source(
            r#"
[[providers.capabilities]]
interface = "example:custom/service"
version = "1.0.0"
features = ["fast-mode"]
"#,
        ))
        .expect("environment should parse");

        let error = inventory_from_specification(&specification)
            .expect_err("unverifiable semantic features should fail");

        assert!(matches!(
            error,
            EnvironmentInventoryError::FeaturesRequireKnownContract {
                interface,
                features,
            } if interface == "example:custom/service" && features == ["fast-mode"]
        ));
    }

    #[test]
    fn unknown_builtin_version_is_rejected() {
        let specification = parse_environment(&source(
            r#"
[[providers.capabilities]]
interface = "presolve:objects/store"
version = "9.0.0"
"#,
        ))
        .expect("environment should parse");

        let error = inventory_from_specification(&specification)
            .expect_err("unknown built-in version should fail");

        assert!(matches!(
            error,
            EnvironmentInventoryError::UnsupportedKnownCapabilityVersion {
                interface,
                version,
            } if interface == OBJECT_STORE_INTERFACE && version == Version::new(9, 0, 0)
        ));
    }

    #[test]
    fn builtin_feature_must_belong_to_canonical_vocabulary() {
        let specification = parse_environment(&source(
            r#"
[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["multipart-upload"]
"#,
        ))
        .expect("environment should parse");

        let error = inventory_from_specification(&specification)
            .expect_err("unknown built-in feature should fail");

        assert!(matches!(
            error,
            EnvironmentInventoryError::InvalidFeatureAdvertisement {
                interface,
                source: CapabilityFeatureError::UnknownFeature { feature, .. },
                ..
            } if interface == OBJECT_STORE_INTERFACE && feature == "multipart-upload"
        ));
    }
}
