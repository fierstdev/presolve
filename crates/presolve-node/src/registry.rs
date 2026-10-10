use std::sync::Arc;

use presolve_core::ProviderId;
use presolve_provider_sdk::{KeyValueProvider, ObjectStoreProvider};
use thiserror::Error;

/// Concrete provider implementations available to a `Presolve` node.
///
/// The resolver operates exclusively on provider descriptors. This registry
/// contains the host-side implementations identified by those same stable
/// [`ProviderId`] values.
///
/// The registry is intentionally owned by the node rather than the resolver or
/// runtime. Resolution decides which provider should be used; materialization
/// retrieves the corresponding implementation.
#[derive(Default)]
pub struct ProviderRegistry {
    key_value: Vec<(ProviderId, Arc<dyn KeyValueProvider>)>,
    object_store: Vec<(ProviderId, Arc<dyn ObjectStoreProvider>)>,
}

impl ProviderRegistry {
    /// Creates an empty provider registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a key/value provider implementation.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderRegistryError::DuplicateKeyValueProvider`] when an
    /// implementation has already been registered for `provider_id`.
    pub fn register_key_value(
        &mut self,
        provider_id: ProviderId,
        provider: Arc<dyn KeyValueProvider>,
    ) -> Result<(), ProviderRegistryError> {
        if self
            .key_value
            .iter()
            .any(|(registered_id, _)| *registered_id == provider_id)
        {
            return Err(ProviderRegistryError::DuplicateKeyValueProvider { provider_id });
        }

        self.key_value.push((provider_id, provider));

        Ok(())
    }

    /// Returns the registered key/value implementation for `provider_id`.
    #[must_use]
    pub fn key_value(&self, provider_id: ProviderId) -> Option<Arc<dyn KeyValueProvider>> {
        self.key_value
            .iter()
            .find(|(registered_id, _)| *registered_id == provider_id)
            .map(|(_, provider)| Arc::clone(provider))
    }

    /// Registers an object-storage provider implementation.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderRegistryError::DuplicateObjectStoreProvider`] when an
    /// implementation has already been registered for `provider_id`.
    pub fn register_object_store(
        &mut self,
        provider_id: ProviderId,
        provider: Arc<dyn ObjectStoreProvider>,
    ) -> Result<(), ProviderRegistryError> {
        if self
            .object_store
            .iter()
            .any(|(registered_id, _)| *registered_id == provider_id)
        {
            return Err(ProviderRegistryError::DuplicateObjectStoreProvider { provider_id });
        }

        self.object_store.push((provider_id, provider));

        Ok(())
    }

    /// Returns the registered object-storage implementation for `provider_id`.
    #[must_use]
    pub fn object_store(&self, provider_id: ProviderId) -> Option<Arc<dyn ObjectStoreProvider>> {
        self.object_store
            .iter()
            .find(|(registered_id, _)| *registered_id == provider_id)
            .map(|(_, provider)| Arc::clone(provider))
    }
}

/// Error returned while modifying a [`ProviderRegistry`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProviderRegistryError {
    /// The same provider has already been registered for key/value service.
    #[error(
        "provider `{provider_id}` is already registered for \
         `presolve:kv/store`"
    )]
    DuplicateKeyValueProvider {
        /// Duplicate provider identifier.
        provider_id: ProviderId,
    },

    /// The same provider has already been registered for object storage.
    #[error(
        "provider `{provider_id}` is already registered for \\
         `presolve:objects/store`"
    )]
    DuplicateObjectStoreProvider {
        /// Duplicate provider identifier.
        provider_id: ProviderId,
    },
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use presolve_provider_kv_memory::InMemoryKeyValueProvider;
    use presolve_provider_objects_memory::InMemoryObjectStoreProvider;

    use super::*;

    #[test]
    fn registered_key_value_provider_can_be_retrieved() {
        let provider_id = ProviderId::new();
        let mut registry = ProviderRegistry::new();

        registry
            .register_key_value(provider_id, Arc::new(InMemoryKeyValueProvider::new()))
            .expect("registration should succeed");

        assert!(registry.key_value(provider_id).is_some());
    }

    #[test]
    fn duplicate_key_value_provider_is_rejected() {
        let provider_id = ProviderId::new();
        let mut registry = ProviderRegistry::new();

        registry
            .register_key_value(provider_id, Arc::new(InMemoryKeyValueProvider::new()))
            .expect("first registration should succeed");

        let error = registry
            .register_key_value(provider_id, Arc::new(InMemoryKeyValueProvider::new()))
            .expect_err("duplicate registration should fail");

        assert_eq!(
            error,
            ProviderRegistryError::DuplicateKeyValueProvider { provider_id }
        );
    }

    #[test]
    fn registered_object_store_provider_can_be_retrieved() {
        let provider_id = ProviderId::new();
        let mut registry = ProviderRegistry::new();

        registry
            .register_object_store(provider_id, Arc::new(InMemoryObjectStoreProvider::new()))
            .expect("registration should succeed");

        assert!(registry.object_store(provider_id).is_some());
    }

    #[test]
    fn duplicate_object_store_provider_is_rejected() {
        let provider_id = ProviderId::new();
        let mut registry = ProviderRegistry::new();

        registry
            .register_object_store(provider_id, Arc::new(InMemoryObjectStoreProvider::new()))
            .expect("first registration should succeed");

        let error = registry
            .register_object_store(provider_id, Arc::new(InMemoryObjectStoreProvider::new()))
            .expect_err("duplicate registration should fail");

        assert_eq!(
            error,
            ProviderRegistryError::DuplicateObjectStoreProvider { provider_id }
        );
    }
}
