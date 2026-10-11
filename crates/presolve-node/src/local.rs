use std::sync::Arc;

use presolve_capability::{key_value_contract, object_store_contract};
use presolve_provider_kv_memory::InMemoryKeyValueProvider;
use presolve_provider_objects_memory::InMemoryObjectStoreProvider;
use presolve_provider_sdk::{KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE};
use presolve_resolver::DeploymentPlan;
use thiserror::Error;

use crate::{ProviderRegistry, ProviderRegistryError};

/// Failure while realizing a resolved plan with Presolve's built-in local providers.
#[derive(Debug, Error)]
pub enum LocalRealizationError {
    /// The local profile does not have an implementation for this capability.
    #[error("local development cannot realize capability `{interface}`")]
    UnsupportedCapability {
        /// Capability interface selected by the resolver.
        interface: String,
    },

    /// The selected exact capability version is not implemented locally.
    #[error(
        "local development implements `{interface}` at `{supported_version}`, \
         but the plan selected `{selected_version}`"
    )]
    UnsupportedVersion {
        /// Capability interface selected by the resolver.
        interface: String,

        /// Exact version selected by the resolver.
        selected_version: String,

        /// Exact version implemented by the local provider.
        supported_version: String,
    },

    /// The plan negotiated semantics that the built-in local provider does not implement.
    #[error(
        "local development provider for `{interface}` does not implement negotiated features: {features}"
    )]
    UnsupportedFeatures {
        /// Capability interface selected by the resolver.
        interface: String,

        /// Human-readable comma-separated feature list.
        features: String,
    },

    /// Registration of a concrete provider implementation failed.
    #[error(transparent)]
    Registry(#[from] ProviderRegistryError),

    /// A built-in canonical capability contract is internally invalid.
    #[error("could not load built-in capability contract `{interface}`: {message}")]
    InvalidBuiltinContract {
        /// Built-in interface whose contract could not be constructed.
        interface: &'static str,

        /// Underlying contract error.
        message: String,
    },
}

/// Builds a concrete provider registry for one resolved local deployment plan.
///
/// Local development currently realizes the baseline key/value and object-store
/// capabilities with Presolve's in-memory providers. Realization is deliberately
/// plan-scoped: only provider identities actually selected by the resolver are
/// instantiated.
///
/// # Errors
///
/// Returns an error when the plan contains a capability, exact version, or
/// negotiated semantic feature not implemented by the local provider profile,
/// or when provider registration fails.
pub fn realize_local_providers(
    plan: &DeploymentPlan,
) -> Result<ProviderRegistry, LocalRealizationError> {
    let mut registry = ProviderRegistry::new();

    for binding in plan.bindings() {
        match binding.interface() {
            KEY_VALUE_INTERFACE => {
                validate_baseline_binding(
                    binding.interface(),
                    binding.provider_version().to_string(),
                    binding.negotiated_features(),
                    key_value_contract()
                        .map_err(|error| LocalRealizationError::InvalidBuiltinContract {
                            interface: KEY_VALUE_INTERFACE,
                            message: error.to_string(),
                        })?
                        .version()
                        .to_string(),
                )?;

                let provider_id = *binding.provider_id();

                if registry.key_value(provider_id).is_none() {
                    registry.register_key_value(
                        provider_id,
                        Arc::new(InMemoryKeyValueProvider::new()),
                    )?;
                }
            }

            OBJECT_STORE_INTERFACE => {
                validate_baseline_binding(
                    binding.interface(),
                    binding.provider_version().to_string(),
                    binding.negotiated_features(),
                    object_store_contract()
                        .map_err(|error| LocalRealizationError::InvalidBuiltinContract {
                            interface: OBJECT_STORE_INTERFACE,
                            message: error.to_string(),
                        })?
                        .version()
                        .to_string(),
                )?;

                let provider_id = *binding.provider_id();

                if registry.object_store(provider_id).is_none() {
                    registry.register_object_store(
                        provider_id,
                        Arc::new(InMemoryObjectStoreProvider::new()),
                    )?;
                }
            }

            interface => {
                return Err(LocalRealizationError::UnsupportedCapability {
                    interface: interface.to_owned(),
                });
            }
        }
    }

    Ok(registry)
}

fn validate_baseline_binding(
    interface: &str,
    selected_version: String,
    negotiated_features: &[String],
    supported_version: String,
) -> Result<(), LocalRealizationError> {
    if selected_version != supported_version {
        return Err(LocalRealizationError::UnsupportedVersion {
            interface: interface.to_owned(),
            selected_version,
            supported_version,
        });
    }

    if !negotiated_features.is_empty() {
        return Err(LocalRealizationError::UnsupportedFeatures {
            interface: interface.to_owned(),
            features: negotiated_features.join(", "),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use presolve_capability::{
        KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE, key_value_contract, object_store_contract,
    };
    use presolve_contract::parse_contract;
    use presolve_core::{EnvironmentId, ProviderId};
    use presolve_resolver::{
        EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
    };
    use semver::Version;

    use crate::assemble_runtime;

    use super::*;

    fn resolved_plan(
        contract_source: &str,
        provider_id: ProviderId,
        capabilities: Vec<ProvidedCapability>,
    ) -> DeploymentPlan {
        let contract = parse_contract(contract_source).expect("test contract should parse");
        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(1_024, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "local-provider",
                capabilities,
            )],
        );

        resolve(&contract, &environment)
            .plan()
            .expect("test application should resolve")
            .clone()
    }

    #[test]
    fn realizes_baseline_kv_and_objects_under_selected_provider_identity() {
        let provider_id = ProviderId::new();
        let kv_contract = key_value_contract().expect("KV contract should be valid");
        let object_contract = object_store_contract().expect("object contract should be valid");

        let plan = resolved_plan(
            r#"
contract_version = "0.1"

[application]
name = "local-baseline"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
"#,
            provider_id,
            vec![
                ProvidedCapability::from_contract(kv_contract),
                ProvidedCapability::from_contract(object_contract),
            ],
        );

        let registry =
            realize_local_providers(&plan).expect("baseline capabilities should realize");

        assert!(registry.key_value(provider_id).is_some());
        assert!(registry.object_store(provider_id).is_some());

        assemble_runtime(&plan, &registry)
            .expect("locally realized plan should assemble into a runtime");
    }

    #[test]
    fn rejects_negotiated_object_store_features_not_implemented_in_memory() {
        let provider_id = ProviderId::new();
        let capability = ProvidedCapability::from_contract(
            object_store_contract().expect("contract should work"),
        )
        .with_supported_features(["range-read"])
        .expect("declared feature should be accepted");

        let plan = resolved_plan(
            r#"
contract_version = "0.1"

[application]
name = "local-feature"
version = "1.0.0"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
"#,
            provider_id,
            vec![capability],
        );

        let Err(error) = realize_local_providers(&plan) else {
            panic!("unsupported local feature should fail");
        };

        assert!(matches!(
            error,
            LocalRealizationError::UnsupportedFeatures { interface, features }
                if interface == OBJECT_STORE_INTERFACE && features == "range-read"
        ));
    }

    #[test]
    fn rejects_custom_capability_selected_by_resolver() {
        let provider_id = ProviderId::new();

        let plan = resolved_plan(
            r#"
contract_version = "0.1"

[application]
name = "local-custom"
version = "1.0.0"

[[capabilities]]
interface = "example:search/index"
version = "^1.0"
"#,
            provider_id,
            vec![ProvidedCapability::new(
                "example:search/index",
                Version::new(1, 2, 0),
            )],
        );

        let Err(error) = realize_local_providers(&plan) else {
            panic!("custom capability should not realize");
        };

        assert!(matches!(
            error,
            LocalRealizationError::UnsupportedCapability { interface }
                if interface == "example:search/index"
        ));
    }

    #[test]
    fn rejects_builtin_version_not_implemented_by_local_profile() {
        let provider_id = ProviderId::new();

        let plan = resolved_plan(
            r#"
contract_version = "0.1"

[application]
name = "future-kv"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^9.0"
"#,
            provider_id,
            vec![ProvidedCapability::new(
                KEY_VALUE_INTERFACE,
                Version::new(9, 0, 0),
            )],
        );

        let Err(error) = realize_local_providers(&plan) else {
            panic!("unsupported local version should fail");
        };

        assert!(matches!(
            error,
            LocalRealizationError::UnsupportedVersion {
                interface,
                selected_version,
                supported_version,
            } if interface == KEY_VALUE_INTERFACE
                && selected_version == "9.0.0"
                && supported_version == "0.1.0"
        ));
    }
}
