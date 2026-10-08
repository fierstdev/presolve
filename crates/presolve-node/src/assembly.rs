use presolve_core::ProviderId;
use presolve_provider_sdk::KEY_VALUE_INTERFACE;
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

            interface => {
                return Err(AssemblyError::UnsupportedCapability {
                    interface: interface.to_owned(),
                });
            }
        }
    }

    builder.build().map_err(AssemblyError::Runtime)
}
