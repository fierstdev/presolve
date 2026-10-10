use std::io::Read;

pub use presolve_capability::OBJECT_STORE_INTERFACE;
use thiserror::Error;

/// Stable categories for object-storage provider failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectStoreErrorCode {
    /// The provider cannot currently service requests.
    Unavailable,

    /// The supplied object key is not valid for the capability.
    InvalidKey,

    /// A supplied object body is malformed or cannot be consumed as declared.
    InvalidBody,

    /// The provider encountered an internal failure.
    Internal,
}

/// Failure returned by an object-storage provider.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ObjectStoreError {
    code: ObjectStoreErrorCode,
    message: String,
}

impl ObjectStoreError {
    /// Creates an unavailable-provider error.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: ObjectStoreErrorCode::Unavailable,
            message: message.into(),
        }
    }

    /// Creates an invalid-key error.
    #[must_use]
    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self {
            code: ObjectStoreErrorCode::InvalidKey,
            message: message.into(),
        }
    }

    /// Creates an invalid-body error.
    #[must_use]
    pub fn invalid_body(message: impl Into<String>) -> Self {
        Self {
            code: ObjectStoreErrorCode::InvalidBody,
            message: message.into(),
        }
    }

    /// Creates an internal-provider error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ObjectStoreErrorCode::Internal,
            message: message.into(),
        }
    }

    /// Returns the stable provider error category.
    #[must_use]
    pub const fn code(&self) -> ObjectStoreErrorCode {
        self.code
    }

    /// Returns the human-readable provider error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Portable system metadata for one object.
///
/// Baseline object-storage semantics expose only metadata every conforming
/// provider can represent without provider-specific translation. Application
/// metadata, provider revisions, `ETag` values, generations, and similar
/// provider concepts are intentionally excluded from this baseline type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    key: String,
    size: u64,
}

impl ObjectInfo {
    /// Creates portable object metadata.
    #[must_use]
    pub fn new(key: impl Into<String>, size: u64) -> Self {
        Self {
            key: key.into(),
            size,
        }
    }

    /// Returns the exact object key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the object body length in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }
}

/// Streaming result of an object read.
///
/// The reader owns the provider's body stream so callers do not need to buffer
/// complete objects in memory before consuming them.
pub struct ObjectRead {
    info: ObjectInfo,
    body: Box<dyn Read + Send>,
}

impl ObjectRead {
    /// Creates a streaming object read result.
    #[must_use]
    pub fn new(info: ObjectInfo, body: Box<dyn Read + Send>) -> Self {
        Self { info, body }
    }

    /// Returns portable metadata for the object being read.
    #[must_use]
    pub const fn info(&self) -> &ObjectInfo {
        &self.info
    }

    /// Returns mutable access to the object body stream.
    #[must_use]
    pub fn body(&mut self) -> &mut (dyn Read + Send + 'static) {
        self.body.as_mut()
    }

    /// Splits the result into portable metadata and its body stream.
    #[must_use]
    pub fn into_parts(self) -> (ObjectInfo, Box<dyn Read + Send>) {
        (self.info, self.body)
    }
}

/// Host implementation of the `presolve:objects/store` capability.
///
/// This trait represents the baseline semantic contract only. Optional
/// semantics such as range reads, conditional writes, and user metadata are
/// negotiated separately rather than being silently assumed for every
/// provider.
///
/// Implementations may be in-memory, filesystem-backed, remote, managed, or
/// backed by infrastructure supplied by the customer's environment.
pub trait ObjectStoreProvider: Send + Sync {
    /// Opens the complete object stored at `key`.
    ///
    /// A missing key returns `Ok(None)`. A present zero-length object returns
    /// `Ok(Some(...))`, preserving the capability's missing-versus-empty
    /// distinction.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectStoreError`] when the provider cannot service the
    /// request or the key is invalid.
    fn get(&self, key: &str) -> Result<Option<ObjectRead>, ObjectStoreError>;

    /// Stores a complete object at `key`.
    ///
    /// `size` is the exact number of bytes expected from `body`. Requiring an
    /// explicit size lets providers stream writes without first buffering the
    /// complete object merely to discover its content length.
    ///
    /// A successful write replaces the complete logical object previously
    /// stored at the same key.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectStoreError`] when the provider cannot service the
    /// request, the key is invalid, or the body cannot be consumed according
    /// to the declared size.
    fn put(
        &self,
        key: &str,
        size: u64,
        body: &mut dyn Read,
    ) -> Result<ObjectInfo, ObjectStoreError>;

    /// Returns portable object metadata without reading the object body.
    ///
    /// A missing key returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectStoreError`] when the provider cannot service the
    /// request or the key is invalid.
    fn head(&self, key: &str) -> Result<Option<ObjectInfo>, ObjectStoreError>;

    /// Removes the object at `key`.
    ///
    /// Returns whether an object existed before deletion. Repeating deletion
    /// of a missing key is safe.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectStoreError`] when the provider cannot service the
    /// request or the key is invalid.
    fn delete(&self, key: &str) -> Result<bool, ObjectStoreError>;

    /// Lists portable metadata for objects whose exact keys begin with
    /// `prefix`.
    ///
    /// Result ordering is intentionally unspecified by the baseline semantic
    /// contract. An empty prefix lists across the provider's bound logical
    /// store.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectStoreError`] when the provider cannot service the
    /// request or the prefix is invalid.
    fn list(&self, prefix: &str) -> Result<Vec<ObjectInfo>, ObjectStoreError>;
}

/// Provider used when no object-storage capability implementation is
/// configured.
///
/// Deployment resolution should normally detect this condition before
/// execution. Keeping an unavailable provider at runtime gives Presolve a
/// safe, explicit fallback instead of ambient host access.
#[derive(Debug, Default)]
pub struct UnavailableObjectStoreProvider;

impl ObjectStoreProvider for UnavailableObjectStoreProvider {
    fn get(&self, _key: &str) -> Result<Option<ObjectRead>, ObjectStoreError> {
        Err(ObjectStoreError::unavailable(
            "object-storage capability is not configured",
        ))
    }

    fn put(
        &self,
        _key: &str,
        _size: u64,
        _body: &mut dyn Read,
    ) -> Result<ObjectInfo, ObjectStoreError> {
        Err(ObjectStoreError::unavailable(
            "object-storage capability is not configured",
        ))
    }

    fn head(&self, _key: &str) -> Result<Option<ObjectInfo>, ObjectStoreError> {
        Err(ObjectStoreError::unavailable(
            "object-storage capability is not configured",
        ))
    }

    fn delete(&self, _key: &str) -> Result<bool, ObjectStoreError> {
        Err(ObjectStoreError::unavailable(
            "object-storage capability is not configured",
        ))
    }

    fn list(&self, _prefix: &str) -> Result<Vec<ObjectInfo>, ObjectStoreError> {
        Err(ObjectStoreError::unavailable(
            "object-storage capability is not configured",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn error_preserves_code_and_message() {
        let error = ObjectStoreError::invalid_key("empty key");

        assert_eq!(error.code(), ObjectStoreErrorCode::InvalidKey);
        assert_eq!(error.message(), "empty key");
    }

    #[test]
    fn object_info_preserves_portable_metadata() {
        let info = ObjectInfo::new("images/logo.png", 42);

        assert_eq!(info.key(), "images/logo.png");
        assert_eq!(info.size(), 42);
    }

    #[test]
    fn object_read_streams_body_without_erasing_metadata() {
        let info = ObjectInfo::new("example", 4);
        let body = Box::new(Cursor::new(b"data".to_vec()));
        let mut object = ObjectRead::new(info.clone(), body);
        let mut bytes = Vec::new();

        object
            .body()
            .read_to_end(&mut bytes)
            .expect("object body should be readable");

        assert_eq!(object.info(), &info);
        assert_eq!(bytes, b"data");
    }

    #[test]
    fn unavailable_provider_fails_explicitly() {
        let provider = UnavailableObjectStoreProvider;

        let error = provider
            .head("example")
            .expect_err("provider should be unavailable");

        assert_eq!(error.code(), ObjectStoreErrorCode::Unavailable);
    }
}
