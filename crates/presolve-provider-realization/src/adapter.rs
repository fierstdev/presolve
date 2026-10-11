use std::{collections::BTreeMap, fmt, sync::Arc};

use presolve_core::ProviderId;
use presolve_provider_sdk::{KeyValueProvider, ObjectStoreProvider};
use semver::Version;
use thiserror::Error;

use crate::{ProviderAdapterId, ProviderRealizationProfile, SecretResolver};

/// One exact capability binding selected for a provider by a deployment plan.
///
/// This type is intentionally resolver-independent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCapabilityRequest {
    interface: String,
    version: Version,
    negotiated_features: Vec<String>,
}

impl ProviderCapabilityRequest {
    /// Creates one exact provider capability request.
    #[must_use]
    pub fn new<I, S>(interface: impl Into<String>, version: Version, negotiated_features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut negotiated_features = negotiated_features
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        negotiated_features.sort();
        negotiated_features.dedup();

        Self {
            interface: interface.into(),
            version,
            negotiated_features,
        }
    }

    /// Returns the semantic capability interface.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the exact provider version selected by resolution.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns negotiated semantic features.
    #[must_use]
    pub fn negotiated_features(&self) -> &[String] {
        &self.negotiated_features
    }
}

/// Concrete capability implementations produced by one provider adapter.
pub struct RealizedProvider {
    provider_id: ProviderId,
    key_value: Option<Arc<dyn KeyValueProvider>>,
    object_store: Option<Arc<dyn ObjectStoreProvider>>,
}

impl RealizedProvider {
    /// Creates an empty realized provider.
    #[must_use]
    pub const fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            key_value: None,
            object_store: None,
        }
    }

    /// Attaches a key/value implementation.
    #[must_use]
    pub fn with_key_value(mut self, provider: Arc<dyn KeyValueProvider>) -> Self {
        self.key_value = Some(provider);
        self
    }

    /// Attaches an object-storage implementation.
    #[must_use]
    pub fn with_object_store(mut self, provider: Arc<dyn ObjectStoreProvider>) -> Self {
        self.object_store = Some(provider);
        self
    }

    /// Returns the stable provider identity realized by the adapter.
    #[must_use]
    pub const fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the key/value implementation, if any.
    #[must_use]
    pub fn key_value(&self) -> Option<Arc<dyn KeyValueProvider>> {
        self.key_value.as_ref().map(Arc::clone)
    }

    /// Returns the object-storage implementation, if any.
    #[must_use]
    pub fn object_store(&self) -> Option<Arc<dyn ObjectStoreProvider>> {
        self.object_store.as_ref().map(Arc::clone)
    }
}

impl fmt::Debug for RealizedProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RealizedProvider")
            .field("provider_id", &self.provider_id)
            .field("key_value", &self.key_value.is_some())
            .field("object_store", &self.object_store.is_some())
            .finish()
    }
}

/// One installed node-side provider implementation factory.
pub trait ProviderAdapter: Send + Sync {
    /// Materializes one selected provider.
    ///
    /// # Errors
    ///
    /// Returns a categorized adapter failure.
    fn realize(
        &self,
        profile: &ProviderRealizationProfile,
        requested: &[ProviderCapabilityRequest],
        secrets: &dyn SecretResolver,
    ) -> Result<RealizedProvider, ProviderAdapterError>;
}

/// Installed provider adapters available to one Presolve node.
#[derive(Default)]
pub struct ProviderAdapterRegistry {
    adapters: BTreeMap<ProviderAdapterId, Arc<dyn ProviderAdapter>>,
}

impl ProviderAdapterRegistry {
    /// Creates an empty adapter registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            adapters: BTreeMap::new(),
        }
    }

    /// Registers one adapter implementation.
    ///
    /// # Errors
    ///
    /// Returns an error when `adapter_id` is already registered.
    pub fn register(
        &mut self,
        adapter_id: ProviderAdapterId,
        adapter: Arc<dyn ProviderAdapter>,
    ) -> Result<(), ProviderAdapterRegistryError> {
        if self.adapters.contains_key(&adapter_id) {
            return Err(ProviderAdapterRegistryError::DuplicateAdapter { adapter_id });
        }

        self.adapters.insert(adapter_id, adapter);
        Ok(())
    }

    /// Returns one installed adapter.
    #[must_use]
    pub fn get(&self, adapter_id: &ProviderAdapterId) -> Option<Arc<dyn ProviderAdapter>> {
        self.adapters.get(adapter_id).map(Arc::clone)
    }

    /// Returns whether an adapter is installed.
    #[must_use]
    pub fn contains(&self, adapter_id: &ProviderAdapterId) -> bool {
        self.adapters.contains_key(adapter_id)
    }
}

impl fmt::Debug for ProviderAdapterRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderAdapterRegistry")
            .field("adapter_ids", &self.adapters.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Failure returned by a provider adapter.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProviderAdapterError {
    /// A required non-secret setting is absent.
    #[error("required provider realization setting `{key}` is missing")]
    MissingSetting {
        /// Adapter-defined setting key.
        key: String,
    },

    /// A required secret binding is absent.
    #[error("required provider realization secret `{name}` is missing")]
    MissingSecret {
        /// Adapter-defined secret binding name.
        name: String,
    },

    /// A secret reference could not be resolved.
    #[error("could not resolve provider secret `{name}`: {message}")]
    SecretResolution {
        /// Adapter-defined secret binding name.
        name: String,

        /// Redaction-safe resolution failure.
        message: String,
    },

    /// The adapter does not implement a selected capability.
    #[error("provider adapter does not implement capability `{interface}`")]
    UnsupportedCapability {
        /// Selected capability interface.
        interface: String,
    },

    /// The adapter does not implement the selected exact version.
    #[error(
        "provider adapter implements `{interface}` at `{supported_version}`, \
         but the plan selected `{selected_version}`"
    )]
    UnsupportedVersion {
        /// Capability interface.
        interface: String,

        /// Version selected by resolution.
        selected_version: Version,

        /// Exact adapter-supported version.
        supported_version: Version,
    },

    /// The adapter cannot supply negotiated features.
    #[error(
        "provider adapter for `{interface}` does not implement negotiated features: {features}"
    )]
    UnsupportedFeatures {
        /// Capability interface.
        interface: String,

        /// Human-readable feature list.
        features: String,
    },

    /// Adapter-specific construction failed.
    #[error("provider adapter configuration failed: {message}")]
    Configuration {
        /// Redaction-safe configuration failure.
        message: String,
    },
}

/// Failure while modifying the installed adapter registry.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProviderAdapterRegistryError {
    /// Adapter ID is already installed.
    #[error("provider adapter `{adapter_id}` is already registered")]
    DuplicateAdapter {
        /// Duplicate adapter identity.
        adapter_id: ProviderAdapterId,
    },
}

#[cfg(test)]
mod tests {
    use presolve_provider_sdk::UnavailableKeyValueProvider;

    use super::*;

    #[derive(Debug)]
    struct TestAdapter;

    impl ProviderAdapter for TestAdapter {
        fn realize(
            &self,
            profile: &ProviderRealizationProfile,
            _requested: &[ProviderCapabilityRequest],
            _secrets: &dyn SecretResolver,
        ) -> Result<RealizedProvider, ProviderAdapterError> {
            Ok(RealizedProvider::new(*profile.provider_id())
                .with_key_value(Arc::new(UnavailableKeyValueProvider)))
        }
    }

    #[test]
    fn registry_resolves_adapter_by_stable_identity() {
        let adapter_id = ProviderAdapterId::new("example:kv").expect("adapter ID should be valid");
        let mut registry = ProviderAdapterRegistry::new();

        registry
            .register(adapter_id.clone(), Arc::new(TestAdapter))
            .expect("adapter should register");

        assert!(registry.contains(&adapter_id));
        assert!(registry.get(&adapter_id).is_some());
    }

    #[test]
    fn duplicate_adapter_identity_is_rejected() {
        let adapter_id = ProviderAdapterId::new("example:kv").expect("adapter ID should be valid");
        let mut registry = ProviderAdapterRegistry::new();

        registry
            .register(adapter_id.clone(), Arc::new(TestAdapter))
            .expect("adapter should register");

        let error = registry
            .register(adapter_id.clone(), Arc::new(TestAdapter))
            .expect_err("duplicate adapter should fail");

        assert_eq!(
            error,
            ProviderAdapterRegistryError::DuplicateAdapter { adapter_id }
        );
    }

    #[test]
    fn capability_requests_normalize_feature_order() {
        let request = ProviderCapabilityRequest::new(
            "example:search/index",
            Version::new(1, 2, 3),
            ["zeta", "alpha", "alpha"],
        );

        assert_eq!(request.negotiated_features(), ["alpha", "zeta"]);
    }
}
