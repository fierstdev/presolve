use presolve_core::ProviderId;
use presolve_provider_realization::{
    ProviderAdapterError, ProviderAdapterRegistry, ProviderCapabilityRequest,
    ProviderRealizationCatalog, RealizedProvider, SecretResolver,
};
use presolve_provider_sdk::{
    KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE, SQL_DATABASE_INTERFACE, WORK_QUEUE_INTERFACE,
};
use presolve_resolver::DeploymentPlan;
use thiserror::Error;

use crate::{ProviderRegistry, ProviderRegistryError};

/// Failure while materializing providers selected by one deployment plan.
#[derive(Debug, Error)]
pub enum ProviderMaterializationError {
    /// No realization profile exists for a selected provider.
    #[error("selected provider `{provider_id}` has no realization profile")]
    MissingProfile {
        /// Provider selected by resolution.
        provider_id: ProviderId,
    },

    /// The selected profile names an adapter not installed in this node.
    #[error("provider `{provider_id}` requires unavailable adapter `{adapter_id}`")]
    MissingAdapter {
        /// Provider selected by resolution.
        provider_id: ProviderId,

        /// Adapter named by the realization profile.
        adapter_id: String,
    },

    /// The installed adapter could not realize the selected provider.
    #[error("provider `{provider_id}` could not be realized by adapter `{adapter_id}`: {source}")]
    Adapter {
        /// Provider selected by resolution.
        provider_id: ProviderId,

        /// Adapter selected by the realization profile.
        adapter_id: String,

        /// Categorized adapter failure.
        #[source]
        source: Box<ProviderAdapterError>,
    },

    /// An adapter attempted to change stable provider identity.
    #[error(
        "adapter realized provider `{realized_provider_id}` for plan-selected provider \
         `{selected_provider_id}`"
    )]
    ProviderIdentityMismatch {
        /// Provider selected by resolution.
        selected_provider_id: ProviderId,

        /// Provider returned by the adapter.
        realized_provider_id: ProviderId,
    },

    /// The adapter returned no implementation for a selected capability.
    #[error("provider `{provider_id}` adapter did not realize selected capability `{interface}`")]
    MissingCapabilityImplementation {
        /// Provider selected by resolution.
        provider_id: ProviderId,

        /// Selected capability interface.
        interface: String,
    },

    /// The current node registry cannot hold this capability.
    #[error("node provider registry does not support capability `{interface}`")]
    UnsupportedRegistryCapability {
        /// Unsupported semantic capability interface.
        interface: String,
    },

    /// Concrete provider registration failed.
    #[error(transparent)]
    Registry(#[from] ProviderRegistryError),
}

/// Materializes only providers selected by `plan`.
///
/// Profiles not selected by the plan are ignored and their secrets are not
/// resolved.
///
/// # Errors
///
/// Returns an error when selected provider realization cannot be completed.
pub fn realize_plan_providers(
    plan: &DeploymentPlan,
    profiles: &ProviderRealizationCatalog,
    adapters: &ProviderAdapterRegistry,
    secrets: &dyn SecretResolver,
) -> Result<ProviderRegistry, ProviderMaterializationError> {
    let selected = selected_provider_requests(plan);
    let mut registry = ProviderRegistry::new();

    for (provider_id, requests) in selected {
        let profile = profiles
            .get(provider_id)
            .ok_or(ProviderMaterializationError::MissingProfile { provider_id })?;

        let adapter_id = profile.adapter();
        let adapter = adapters.get(adapter_id).ok_or_else(|| {
            ProviderMaterializationError::MissingAdapter {
                provider_id,
                adapter_id: adapter_id.to_string(),
            }
        })?;

        let realized = adapter
            .realize(profile, &requests, secrets)
            .map_err(|source| ProviderMaterializationError::Adapter {
                provider_id,
                adapter_id: adapter_id.to_string(),
                source: Box::new(source),
            })?;

        if *realized.provider_id() != provider_id {
            return Err(ProviderMaterializationError::ProviderIdentityMismatch {
                selected_provider_id: provider_id,
                realized_provider_id: *realized.provider_id(),
            });
        }

        register_selected_capabilities(&mut registry, provider_id, &requests, &realized)?;
    }

    Ok(registry)
}

fn selected_provider_requests(
    plan: &DeploymentPlan,
) -> Vec<(ProviderId, Vec<ProviderCapabilityRequest>)> {
    let mut selected: Vec<(ProviderId, Vec<ProviderCapabilityRequest>)> = Vec::new();

    for binding in plan.bindings() {
        let provider_id = *binding.provider_id();
        let request = ProviderCapabilityRequest::new(
            binding.interface(),
            binding.provider_version().clone(),
            binding.negotiated_features().iter().cloned(),
        );

        if let Some((_, requests)) = selected
            .iter_mut()
            .find(|(selected_id, _)| *selected_id == provider_id)
        {
            requests.push(request);
        } else {
            selected.push((provider_id, vec![request]));
        }
    }

    selected.sort_by_key(|(provider_id, _)| *provider_id);

    for (_, requests) in &mut selected {
        requests.sort_by(|left, right| {
            left.interface()
                .cmp(right.interface())
                .then_with(|| left.version().cmp(right.version()))
                .then_with(|| left.negotiated_features().cmp(right.negotiated_features()))
        });
    }

    selected
}

fn register_selected_capabilities(
    registry: &mut ProviderRegistry,
    provider_id: ProviderId,
    requests: &[ProviderCapabilityRequest],
    realized: &RealizedProvider,
) -> Result<(), ProviderMaterializationError> {
    for request in requests {
        match request.interface() {
            KEY_VALUE_INTERFACE => {
                if registry.key_value(provider_id).is_some() {
                    continue;
                }

                let provider = realized.key_value().ok_or_else(|| {
                    ProviderMaterializationError::MissingCapabilityImplementation {
                        provider_id,
                        interface: request.interface().to_owned(),
                    }
                })?;

                registry.register_key_value(provider_id, provider)?;
            }

            OBJECT_STORE_INTERFACE => {
                if registry.object_store(provider_id).is_some() {
                    continue;
                }

                let provider = realized.object_store().ok_or_else(|| {
                    ProviderMaterializationError::MissingCapabilityImplementation {
                        provider_id,
                        interface: request.interface().to_owned(),
                    }
                })?;

                registry.register_object_store(provider_id, provider)?;
            }

            SQL_DATABASE_INTERFACE => {
                if registry.sql_database(provider_id).is_some() {
                    continue;
                }

                let provider = realized.sql_database().ok_or_else(|| {
                    ProviderMaterializationError::MissingCapabilityImplementation {
                        provider_id,
                        interface: request.interface().to_owned(),
                    }
                })?;

                registry.register_sql_database(provider_id, provider)?;
            }

            WORK_QUEUE_INTERFACE => {
                if registry.work_queue(provider_id).is_some() {
                    continue;
                }

                let provider = realized.work_queue().ok_or_else(|| {
                    ProviderMaterializationError::MissingCapabilityImplementation {
                        provider_id,
                        interface: request.interface().to_owned(),
                    }
                })?;

                registry.register_work_queue(provider_id, provider)?;
            }

            interface => {
                return Err(
                    ProviderMaterializationError::UnsupportedRegistryCapability {
                        interface: interface.to_owned(),
                    },
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use presolve_capability::{key_value_contract, work_queue_contract};
    use presolve_contract::parse_contract;
    use presolve_core::{EnvironmentId, ProviderId};
    use presolve_provider_kv_cloudflare::{
        CLOUDFLARE_ACCOUNT_ID_SETTING, CLOUDFLARE_API_TOKEN_SECRET, CLOUDFLARE_KV_ADAPTER_ID,
        CLOUDFLARE_NAMESPACE_ID_SETTING, CloudflareKvAdapter,
    };
    use presolve_provider_queue_memory::{InMemoryWorkQueueAdapter, MEMORY_QUEUE_ADAPTER_ID};
    use presolve_provider_realization::{
        ProviderAdapterId, ProviderRealizationProfile, SecretReference, SecretResolutionError,
        SecretValue,
    };
    use presolve_resolver::{
        EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
    };

    use super::*;

    struct CountingSecrets {
        calls: AtomicUsize,
    }

    impl CountingSecrets {
        const fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl SecretResolver for CountingSecrets {
        fn resolve(
            &self,
            _reference: &SecretReference,
        ) -> Result<SecretValue, SecretResolutionError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(SecretValue::new("test-token"))
        }
    }

    fn resolved_kv_plan(provider_id: ProviderId) -> DeploymentPlan {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "cloud-materialization"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
"#,
        )
        .expect("contract should parse");

        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(1_024, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "cloud-kv",
                vec![ProvidedCapability::from_contract(
                    key_value_contract().expect("KV contract should be valid"),
                )],
            )],
        );

        resolve(&contract, &environment)
            .plan()
            .expect("application should resolve")
            .clone()
    }

    fn resolved_queue_plan(provider_id: ProviderId) -> DeploymentPlan {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "queue-materialization"
version = "1.0.0"

[[capabilities]]
interface = "presolve:queue/work-queue"
version = "^0.1"
"#,
        )
        .expect("queue contract should parse");

        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(1_024, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "memory-queue",
                vec![ProvidedCapability::from_contract(
                    work_queue_contract().expect("queue contract should be valid"),
                )],
            )],
        );

        resolve(&contract, &environment)
            .plan()
            .expect("queue application should resolve")
            .clone()
    }

    fn cloudflare_profile(provider_id: ProviderId) -> ProviderRealizationProfile {
        ProviderRealizationProfile::new(
            provider_id,
            ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID).expect("adapter ID should be valid"),
        )
        .with_setting(
            CLOUDFLARE_ACCOUNT_ID_SETTING,
            "0123456789abcdef0123456789abcdef",
        )
        .expect("account setting should work")
        .with_setting(
            CLOUDFLARE_NAMESPACE_ID_SETTING,
            "abcdef0123456789abcdef0123456789",
        )
        .expect("namespace setting should work")
        .with_secret(
            CLOUDFLARE_API_TOKEN_SECRET,
            SecretReference::environment_variable("PRESOLVE_TEST_CLOUDFLARE_TOKEN")
                .expect("secret reference should work"),
        )
        .expect("secret binding should work")
    }

    #[test]
    fn plan_selected_cloudflare_provider_materializes_under_exact_provider_id() {
        let provider_id = ProviderId::new();
        let plan = resolved_kv_plan(provider_id);

        let mut profiles = ProviderRealizationCatalog::new();
        profiles
            .register(cloudflare_profile(provider_id))
            .expect("profile should register");

        let mut adapters = ProviderAdapterRegistry::new();
        adapters
            .register(
                ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID).expect("adapter ID should work"),
                Arc::new(CloudflareKvAdapter),
            )
            .expect("adapter should register");

        let secrets = CountingSecrets::new();
        let registry = realize_plan_providers(&plan, &profiles, &adapters, &secrets)
            .expect("selected cloud provider should realize");

        assert!(registry.key_value(provider_id).is_some());
        assert_eq!(secrets.calls(), 1);
    }

    #[test]
    fn plan_selected_memory_queue_materializes_under_exact_provider_id() {
        let provider_id = ProviderId::new();
        let plan = resolved_queue_plan(provider_id);

        let mut profiles = ProviderRealizationCatalog::new();
        profiles
            .register(ProviderRealizationProfile::new(
                provider_id,
                ProviderAdapterId::new(MEMORY_QUEUE_ADAPTER_ID).expect("adapter ID should work"),
            ))
            .expect("queue profile should register");

        let mut adapters = ProviderAdapterRegistry::new();
        adapters
            .register(
                ProviderAdapterId::new(MEMORY_QUEUE_ADAPTER_ID).expect("adapter ID should work"),
                Arc::new(InMemoryWorkQueueAdapter),
            )
            .expect("queue adapter should register");

        let secrets = CountingSecrets::new();
        let registry = realize_plan_providers(&plan, &profiles, &adapters, &secrets)
            .expect("selected queue provider should realize");

        assert!(registry.work_queue(provider_id).is_some());
        assert_eq!(secrets.calls(), 0);
    }

    #[test]
    fn unselected_provider_profile_is_not_materialized() {
        let selected_provider_id = ProviderId::new();
        let unused_provider_id = ProviderId::new();
        let plan = resolved_kv_plan(selected_provider_id);

        let mut profiles = ProviderRealizationCatalog::new();
        profiles
            .register(cloudflare_profile(selected_provider_id))
            .expect("selected profile should register");

        profiles
            .register(ProviderRealizationProfile::new(
                unused_provider_id,
                ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID).expect("adapter ID should work"),
            ))
            .expect("unused incomplete profile should register");

        let mut adapters = ProviderAdapterRegistry::new();
        adapters
            .register(
                ProviderAdapterId::new(CLOUDFLARE_KV_ADAPTER_ID).expect("adapter ID should work"),
                Arc::new(CloudflareKvAdapter),
            )
            .expect("adapter should register");

        let secrets = CountingSecrets::new();
        let registry = realize_plan_providers(&plan, &profiles, &adapters, &secrets)
            .expect("unused profile must not affect selected plan");

        assert!(registry.key_value(selected_provider_id).is_some());
        assert!(registry.key_value(unused_provider_id).is_none());
        assert_eq!(secrets.calls(), 1);
    }

    #[test]
    fn selected_provider_without_profile_fails_before_adapter_use() {
        let provider_id = ProviderId::new();
        let plan = resolved_kv_plan(provider_id);
        let profiles = ProviderRealizationCatalog::new();
        let adapters = ProviderAdapterRegistry::new();
        let secrets = CountingSecrets::new();

        let Err(error) = realize_plan_providers(&plan, &profiles, &adapters, &secrets) else {
            panic!("missing selected profile should fail");
        };

        assert!(matches!(
            error,
            ProviderMaterializationError::MissingProfile {
                provider_id: missing
            } if missing == provider_id
        ));
        assert_eq!(secrets.calls(), 0);
    }
}
