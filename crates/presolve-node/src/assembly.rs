use presolve_core::ProviderId;
use presolve_provider_sdk::{KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE};
use presolve_resolver::DeploymentPlan;
use presolve_runtime::{Runtime, RuntimeError};
use thiserror::Error;

use crate::ProviderRegistry;

/// Failure while materializing a symbolic deployment plan.
#[derive(Debug, Error)]
pub enum AssemblyError {
    /// The deployment plan references a capability this node cannot yet bind.
    #[error("node cannot materialize capability `{interface}`")]
    UnsupportedCapability {
        /// Unsupported capability interface.
        interface: String,
    },

    /// The resolver selected a provider descriptor for which the node has no
    /// concrete implementation.
    #[error(
        "provider `{provider_id}` selected for capability `{interface}` \
         has no registered implementation"
    )]
    MissingProvider {
        /// Capability that requires the provider.
        interface: String,

        /// Provider selected by the resolver.
        provider_id: ProviderId,
    },

    /// Runtime construction failed.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

/// Materializes a symbolic deployment plan into an executable runtime.
///
/// The resolver remains unaware of host implementation objects. This function
/// is the boundary where symbolic [`ProviderId`] bindings are translated into
/// concrete provider handles supplied to the runtime.
///
/// # Errors
///
/// Returns [`AssemblyError::UnsupportedCapability`] when the deployment plan
/// contains a capability this node cannot materialize,
/// [`AssemblyError::MissingProvider`] when a selected provider implementation
/// is absent from the registry, or [`AssemblyError::Runtime`] when runtime
/// construction fails.
pub fn assemble_runtime(
    plan: &DeploymentPlan,
    registry: &ProviderRegistry,
) -> Result<Runtime, AssemblyError> {
    let mut builder = Runtime::builder();

    for binding in plan.bindings() {
        match binding.interface() {
            KEY_VALUE_INTERFACE => {
                let provider_id = *binding.provider_id();

                let provider = registry.key_value(provider_id).ok_or_else(|| {
                    AssemblyError::MissingProvider {
                        interface: binding.interface().to_owned(),
                        provider_id,
                    }
                })?;

                builder = builder.key_value_provider(provider);
            }

            OBJECT_STORE_INTERFACE => {
                let provider_id = *binding.provider_id();

                let provider = registry.object_store(provider_id).ok_or_else(|| {
                    AssemblyError::MissingProvider {
                        interface: binding.interface().to_owned(),
                        provider_id,
                    }
                })?;

                builder = builder.object_store_provider(provider);
            }

            interface => {
                return Err(AssemblyError::UnsupportedCapability {
                    interface: interface.to_owned(),
                });
            }
        }
    }

    builder.build().map_err(AssemblyError::Runtime)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use presolve_contract::parse_contract;
    use presolve_core::{EnvironmentId, ProviderId};
    use presolve_provider_objects_memory::InMemoryObjectStoreProvider;
    use presolve_resolver::{
        EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
    };
    use semver::Version;

    use super::*;

    fn object_storage_plan(provider_id: ProviderId) -> presolve_resolver::DeploymentPlan {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "node-object-test"
version = "0.0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"

[resources]

[network]
outbound = "deny"
"#,
        )
        .expect("test contract should parse");

        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(512, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "memory-objects",
                vec![ProvidedCapability::new(
                    OBJECT_STORE_INTERFACE,
                    Version::new(0, 1, 0),
                )],
            )],
        );

        resolve(&contract, &environment)
            .plan()
            .expect("object-storage requirement should resolve")
            .clone()
    }

    #[test]
    fn assembles_object_storage_provider_from_resolved_plan() {
        let provider_id = ProviderId::new();
        let plan = object_storage_plan(provider_id);
        let mut registry = ProviderRegistry::new();

        registry
            .register_object_store(provider_id, Arc::new(InMemoryObjectStoreProvider::new()))
            .expect("object-store registration should succeed");

        assemble_runtime(&plan, &registry).expect("object-store plan should materialize");
    }

    #[test]
    fn missing_object_storage_provider_is_reported() {
        let provider_id = ProviderId::new();
        let plan = object_storage_plan(provider_id);
        let registry = ProviderRegistry::new();

        let Err(error) = assemble_runtime(&plan, &registry) else {
            panic!("missing provider should be rejected");
        };

        assert!(matches!(
            error,
            AssemblyError::MissingProvider {
                interface,
                provider_id: missing_provider_id,
            } if interface == OBJECT_STORE_INTERFACE && missing_provider_id == provider_id
        ));
    }
}
