#![forbid(unsafe_code)]

//! In-memory implementation of the `Presolve` object-storage capability.
//!
//! This provider is a semantic reference implementation for development,
//! tests, and local environments. It deliberately implements only the baseline
//! object-storage contract; optional features such as range reads, conditional
//! writes, and user metadata are introduced separately.

use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
    sync::RwLock,
};

use presolve_provider_sdk::{ObjectInfo, ObjectRead, ObjectStoreError, ObjectStoreProvider};

/// Process-local object-storage provider backed by an ordered map.
#[derive(Debug, Default)]
pub struct InMemoryObjectStoreProvider {
    objects: RwLock<BTreeMap<String, Vec<u8>>>,
}

impl InMemoryObjectStoreProvider {
    /// Creates an empty in-memory object store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ObjectStoreProvider for InMemoryObjectStoreProvider {
    fn get(&self, key: &str) -> Result<Option<ObjectRead>, ObjectStoreError> {
        validate_key(key)?;

        let objects = self
            .objects
            .read()
            .map_err(|_| ObjectStoreError::internal("object-store read lock was poisoned"))?;

        let Some(bytes) = objects.get(key) else {
            return Ok(None);
        };

        let size = u64::try_from(bytes.len())
            .map_err(|_| ObjectStoreError::internal("object size cannot be represented"))?;

        Ok(Some(ObjectRead::new(
            ObjectInfo::new(key, size),
            Box::new(Cursor::new(bytes.clone())),
        )))
    }

    fn put(
        &self,
        key: &str,
        size: u64,
        body: &mut dyn Read,
    ) -> Result<ObjectInfo, ObjectStoreError> {
        validate_key(key)?;

        let bytes = read_exact_body(size, body)?;

        let mut objects = self
            .objects
            .write()
            .map_err(|_| ObjectStoreError::internal("object-store write lock was poisoned"))?;

        objects.insert(key.to_owned(), bytes);

        Ok(ObjectInfo::new(key, size))
    }

    fn head(&self, key: &str) -> Result<Option<ObjectInfo>, ObjectStoreError> {
        validate_key(key)?;

        let objects = self
            .objects
            .read()
            .map_err(|_| ObjectStoreError::internal("object-store read lock was poisoned"))?;

        let Some(bytes) = objects.get(key) else {
            return Ok(None);
        };

        let size = u64::try_from(bytes.len())
            .map_err(|_| ObjectStoreError::internal("object size cannot be represented"))?;

        Ok(Some(ObjectInfo::new(key, size)))
    }

    fn delete(&self, key: &str) -> Result<bool, ObjectStoreError> {
        validate_key(key)?;

        let mut objects = self
            .objects
            .write()
            .map_err(|_| ObjectStoreError::internal("object-store write lock was poisoned"))?;

        Ok(objects.remove(key).is_some())
    }

    fn list(&self, prefix: &str) -> Result<Vec<ObjectInfo>, ObjectStoreError> {
        let objects = self
            .objects
            .read()
            .map_err(|_| ObjectStoreError::internal("object-store read lock was poisoned"))?;

        objects
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, bytes)| {
                let size = u64::try_from(bytes.len())
                    .map_err(|_| ObjectStoreError::internal("object size cannot be represented"))?;

                Ok(ObjectInfo::new(key, size))
            })
            .collect()
    }
}

fn validate_key(key: &str) -> Result<(), ObjectStoreError> {
    if key.is_empty() {
        return Err(ObjectStoreError::invalid_key(
            "object key must not be empty",
        ));
    }

    Ok(())
}

fn read_exact_body(size: u64, body: &mut dyn Read) -> Result<Vec<u8>, ObjectStoreError> {
    let mut bytes = Vec::new();

    {
        let mut limited = (&mut *body).take(size);

        limited.read_to_end(&mut bytes).map_err(|error| {
            ObjectStoreError::invalid_body(format!("failed to read body: {error}"))
        })?;
    }

    let actual = u64::try_from(bytes.len())
        .map_err(|_| ObjectStoreError::invalid_body("object body is too large"))?;

    if actual != size {
        return Err(ObjectStoreError::invalid_body(format!(
            "object body length mismatch: expected {size} bytes, found {actual}"
        )));
    }

    let mut extra = [0_u8; 1];

    let trailing = body
        .read(&mut extra)
        .map_err(|error| ObjectStoreError::invalid_body(format!("failed to read body: {error}")))?;

    if trailing != 0 {
        return Err(ObjectStoreError::invalid_body(format!(
            "object body length exceeds declared size of {size} bytes"
        )));
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use presolve_provider_sdk::ObjectStoreErrorCode;

    use super::*;

    fn put_bytes(
        provider: &InMemoryObjectStoreProvider,
        key: &str,
        bytes: &[u8],
    ) -> Result<ObjectInfo, ObjectStoreError> {
        let size = u64::try_from(bytes.len()).expect("test body size should fit in u64");
        let mut body = Cursor::new(bytes.to_vec());

        provider.put(key, size, &mut body)
    }

    fn get_bytes(provider: &InMemoryObjectStoreProvider, key: &str) -> Option<Vec<u8>> {
        provider
            .get(key)
            .expect("get should succeed")
            .map(|mut object| {
                let mut bytes = Vec::new();

                object
                    .body()
                    .read_to_end(&mut bytes)
                    .expect("object body should be readable");

                bytes
            })
    }

    #[test]
    fn binary_object_round_trips() {
        let provider = InMemoryObjectStoreProvider::new();
        let bytes = [0_u8, 1, 2, 0xff, 0x00];

        let info = put_bytes(&provider, "images/example.bin", &bytes).expect("put should succeed");

        assert_eq!(info.key(), "images/example.bin");
        assert_eq!(info.size(), 5);
        assert_eq!(
            get_bytes(&provider, "images/example.bin"),
            Some(bytes.to_vec())
        );
    }

    #[test]
    fn successful_put_replaces_complete_object() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "example", b"first").expect("first put should succeed");
        put_bytes(&provider, "example", b"second").expect("second put should succeed");

        assert_eq!(get_bytes(&provider, "example"), Some(b"second".to_vec()));
    }

    #[test]
    fn zero_length_object_is_distinct_from_missing_object() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "empty", b"").expect("empty object put should succeed");

        assert_eq!(get_bytes(&provider, "empty"), Some(Vec::new()));
        assert_eq!(get_bytes(&provider, "missing"), None);
    }

    #[test]
    fn head_reports_key_and_size_without_body() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "assets/logo", b"presolve").expect("put should succeed");

        let info = provider
            .head("assets/logo")
            .expect("head should succeed")
            .expect("object should exist");

        assert_eq!(info, ObjectInfo::new("assets/logo", 8));
    }

    #[test]
    fn delete_is_idempotent() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "example", b"value").expect("put should succeed");

        assert!(
            provider
                .delete("example")
                .expect("first delete should succeed")
        );
        assert!(
            !provider
                .delete("example")
                .expect("second delete should succeed")
        );
    }

    #[test]
    fn list_filters_by_exact_key_prefix() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "images/a.png", b"a").expect("first put should succeed");
        put_bytes(&provider, "images/b.png", b"bb").expect("second put should succeed");
        put_bytes(&provider, "documents/a.txt", b"doc").expect("third put should succeed");

        assert_eq!(
            provider.list("images/").expect("list should succeed"),
            vec![
                ObjectInfo::new("images/a.png", 1),
                ObjectInfo::new("images/b.png", 2),
            ]
        );

        assert_eq!(
            provider.list("missing/").expect("list should succeed"),
            Vec::<ObjectInfo>::new()
        );
    }

    #[test]
    fn short_body_is_rejected_without_replacing_existing_object() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "example", b"existing").expect("initial put should succeed");

        let mut body = Cursor::new(b"new".to_vec());
        let error = provider
            .put("example", 4, &mut body)
            .expect_err("short body should fail");

        assert_eq!(error.code(), ObjectStoreErrorCode::InvalidBody);
        assert_eq!(get_bytes(&provider, "example"), Some(b"existing".to_vec()));
    }

    #[test]
    fn body_longer_than_declared_size_is_rejected_without_mutation() {
        let provider = InMemoryObjectStoreProvider::new();

        put_bytes(&provider, "example", b"existing").expect("initial put should succeed");

        let mut body = Cursor::new(b"new".to_vec());
        let error = provider
            .put("example", 2, &mut body)
            .expect_err("long body should fail");

        assert_eq!(error.code(), ObjectStoreErrorCode::InvalidBody);
        assert_eq!(get_bytes(&provider, "example"), Some(b"existing".to_vec()));
    }

    #[test]
    fn empty_keys_are_rejected() {
        let provider = InMemoryObjectStoreProvider::new();
        let mut body = Cursor::new(b"value".to_vec());

        let error = provider
            .put("", 5, &mut body)
            .expect_err("empty object key should fail");

        assert_eq!(error.code(), ObjectStoreErrorCode::InvalidKey);
    }
}

#[cfg(test)]
mod conformance_tests {
    use presolve_provider_conformance::assert_object_store_conformance;

    use super::InMemoryObjectStoreProvider;

    #[test]
    fn presolve_baseline_conformance() {
        assert_object_store_conformance(InMemoryObjectStoreProvider::new);
    }
}
