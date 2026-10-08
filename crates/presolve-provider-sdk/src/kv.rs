use thiserror::Error;

/// Canonical WIT interface identifier for the key/value capability.
pub const KEY_VALUE_INTERFACE: &str = "presolve:kv/store";

/// Stable categories for key/value provider failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyValueErrorCode {
    /// The provider cannot currently service requests.
    Unavailable,

    /// The supplied key is not valid.
    InvalidKey,

    /// The provider encountered an internal failure.
    Internal,
}

/// Failure returned by a key/value provider.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct KeyValueError {
    code: KeyValueErrorCode,
    message: String,
}

impl KeyValueError {
    /// Creates an unavailable-provider error.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: KeyValueErrorCode::Unavailable,
            message: message.into(),
        }
    }

    /// Creates an invalid-key error.
    #[must_use]
    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self {
            code: KeyValueErrorCode::InvalidKey,
            message: message.into(),
        }
    }

    /// Creates an internal-provider error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: KeyValueErrorCode::Internal,
            message: message.into(),
        }
    }

    /// Returns the stable provider error category.
    #[must_use]
    pub const fn code(&self) -> KeyValueErrorCode {
        self.code
    }

    /// Returns the human-readable provider error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Host implementation of the `presolve:kv/store` capability.
///
/// Implementations may be in-memory, local, remote, managed, or backed by
/// infrastructure supplied by the customer's environment.
pub trait KeyValueProvider: Send + Sync {
    /// Fetches the value associated with `key`.
    ///
    /// # Errors
    ///
    /// Returns [`KeyValueError`] when the provider cannot service the request
    /// or the key is invalid.
    fn get(&self, key: &str) -> Result<Option<String>, KeyValueError>;

    /// Stores `value` at `key`.
    ///
    /// # Errors
    ///
    /// Returns [`KeyValueError`] when the provider cannot service the request
    /// or the key is invalid.
    fn set(&self, key: &str, value: &str) -> Result<(), KeyValueError>;

    /// Removes `key`, returning whether it previously existed.
    ///
    /// # Errors
    ///
    /// Returns [`KeyValueError`] when the provider cannot service the request
    /// or the key is invalid.
    fn delete(&self, key: &str) -> Result<bool, KeyValueError>;
}

/// Provider used when no key/value capability implementation is configured.
///
/// Deployment resolution should normally detect this condition before
/// execution. Keeping an unavailable provider at runtime gives us a safe,
/// explicit fallback instead of ambient host access.
#[derive(Debug, Default)]
pub struct UnavailableKeyValueProvider;

impl KeyValueProvider for UnavailableKeyValueProvider {
    fn get(&self, _key: &str) -> Result<Option<String>, KeyValueError> {
        Err(KeyValueError::unavailable(
            "key/value capability is not configured",
        ))
    }

    fn set(&self, _key: &str, _value: &str) -> Result<(), KeyValueError> {
        Err(KeyValueError::unavailable(
            "key/value capability is not configured",
        ))
    }

    fn delete(&self, _key: &str) -> Result<bool, KeyValueError> {
        Err(KeyValueError::unavailable(
            "key/value capability is not configured",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_preserves_code_and_message() {
        let error = KeyValueError::invalid_key("empty key");

        assert_eq!(error.code(), KeyValueErrorCode::InvalidKey);
        assert_eq!(error.message(), "empty key");
    }

    #[test]
    fn unavailable_provider_fails_explicitly() {
        let provider = UnavailableKeyValueProvider;

        let error = provider
            .get("example")
            .expect_err("provider should be unavailable");

        assert_eq!(error.code(), KeyValueErrorCode::Unavailable);
    }
}
