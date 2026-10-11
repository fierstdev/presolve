#![forbid(unsafe_code)]

//! In-memory implementation of the `Presolve` key/value capability.
//!
//! This provider exists primarily for development, tests, and local
//! environments. Applications interact with the same capability interface
//! regardless of which provider is selected.

use std::{collections::HashMap, sync::RwLock};

use presolve_provider_sdk::{KeyValueError, KeyValueProvider};

/// Process-local key/value provider backed by a hash map.
#[derive(Debug, Default)]
pub struct InMemoryKeyValueProvider {
    values: RwLock<HashMap<String, String>>,
}

impl InMemoryKeyValueProvider {
    /// Creates an empty in-memory provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeyValueProvider for InMemoryKeyValueProvider {
    fn get(&self, key: &str) -> Result<Option<String>, KeyValueError> {
        validate_key(key)?;

        let values = self
            .values
            .read()
            .map_err(|_| KeyValueError::internal("key/value read lock was poisoned"))?;

        Ok(values.get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), KeyValueError> {
        validate_key(key)?;

        let mut values = self
            .values
            .write()
            .map_err(|_| KeyValueError::internal("key/value write lock was poisoned"))?;

        values.insert(key.to_owned(), value.to_owned());

        Ok(())
    }

    fn delete(&self, key: &str) -> Result<bool, KeyValueError> {
        validate_key(key)?;

        let mut values = self
            .values
            .write()
            .map_err(|_| KeyValueError::internal("key/value write lock was poisoned"))?;

        Ok(values.remove(key).is_some())
    }
}

fn validate_key(key: &str) -> Result<(), KeyValueError> {
    if key.is_empty() {
        return Err(KeyValueError::invalid_key("key must not be empty"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_can_be_stored_and_loaded() {
        let provider = InMemoryKeyValueProvider::new();

        provider.set("answer", "42").expect("set should succeed");

        assert_eq!(
            provider.get("answer").expect("get should succeed"),
            Some("42".to_owned())
        );
    }

    #[test]
    fn deleting_missing_key_returns_false() {
        let provider = InMemoryKeyValueProvider::new();

        assert!(!provider.delete("missing").expect("delete should succeed"));
    }

    #[test]
    fn empty_keys_are_rejected() {
        let provider = InMemoryKeyValueProvider::new();

        assert!(provider.set("", "value").is_err());
    }
}

#[cfg(test)]
mod conformance_tests {
    use presolve_provider_conformance::assert_key_value_conformance;

    use super::InMemoryKeyValueProvider;

    #[test]
    fn presolve_baseline_conformance() {
        assert_key_value_conformance(InMemoryKeyValueProvider::new);
    }
}
