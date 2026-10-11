#![forbid(unsafe_code)]

//! Node-side provider realization configuration.
//!
//! The Environment Specification describes provider identity and semantic
//! capability supply without embedding SDK configuration, endpoints,
//! credentials, or secret values. This crate models the separate boundary a
//! Presolve node uses to turn a selected `ProviderId` into a concrete provider.

use std::{
    collections::BTreeMap,
    env::{self, VarError},
    fmt,
};

use presolve_core::ProviderId;
use thiserror::Error;

const MAX_ADAPTER_ID_LENGTH: usize = 128;
const MAX_BINDING_KEY_LENGTH: usize = 128;

/// Stable identifier for one installed provider adapter implementation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderAdapterId(String);

impl ProviderAdapterId {
    /// Parses and validates an adapter identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is not canonical lowercase ASCII.
    pub fn new(value: impl Into<String>) -> Result<Self, ProviderRealizationError> {
        let value = value.into();

        if !valid_adapter_id(&value) {
            return Err(ProviderRealizationError::InvalidAdapterId { value });
        }

        Ok(Self(value))
    }

    /// Returns the canonical adapter identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ProviderAdapterId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ProviderAdapterId")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for ProviderAdapterId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Reference to secret material resolved only at provider realization time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretReference {
    /// Resolve the secret from one process environment variable.
    EnvironmentVariable {
        /// Environment variable name.
        name: String,
    },
}

impl SecretReference {
    /// Creates an environment-variable secret reference.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid environment-variable name.
    pub fn environment_variable(name: impl Into<String>) -> Result<Self, ProviderRealizationError> {
        let name = name.into();

        if !valid_environment_variable_name(&name) {
            return Err(ProviderRealizationError::InvalidSecretReference { name });
        }

        Ok(Self::EnvironmentVariable { name })
    }

    /// Returns the referenced environment-variable name.
    #[must_use]
    pub fn environment_variable_name(&self) -> &str {
        match self {
            Self::EnvironmentVariable { name } => name,
        }
    }
}

/// Resolved secret material.
///
/// Debug formatting is always redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    /// Wraps resolved secret material.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Explicitly exposes secret material to an adapter.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([redacted])")
    }
}

/// Source capable of resolving secret references.
pub trait SecretResolver: Send + Sync {
    /// Resolves one secret reference.
    ///
    /// # Errors
    ///
    /// Returns a categorized resolution failure without secret material.
    fn resolve(&self, reference: &SecretReference) -> Result<SecretValue, SecretResolutionError>;
}

/// Resolves secret references from the Presolve node process environment.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentSecretResolver;

impl SecretResolver for EnvironmentSecretResolver {
    fn resolve(&self, reference: &SecretReference) -> Result<SecretValue, SecretResolutionError> {
        let name = reference.environment_variable_name();

        match env::var(name) {
            Ok(value) => Ok(SecretValue::new(value)),
            Err(VarError::NotPresent) => Err(SecretResolutionError::NotPresent {
                name: name.to_owned(),
            }),
            Err(VarError::NotUnicode(_)) => Err(SecretResolutionError::NotUnicode {
                name: name.to_owned(),
            }),
        }
    }
}

/// Node-side realization metadata for one stable environment provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRealizationProfile {
    provider_id: ProviderId,
    adapter: ProviderAdapterId,
    settings: BTreeMap<String, String>,
    secrets: BTreeMap<String, SecretReference>,
}

impl ProviderRealizationProfile {
    /// Creates an empty realization profile for one provider identity.
    #[must_use]
    pub fn new(provider_id: ProviderId, adapter: ProviderAdapterId) -> Self {
        Self {
            provider_id,
            adapter,
            settings: BTreeMap::new(),
            secrets: BTreeMap::new(),
        }
    }

    /// Adds non-secret adapter configuration.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid or duplicate setting key.
    pub fn with_setting(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ProviderRealizationError> {
        let key = key.into();
        validate_binding_key(&key)?;

        if self.settings.contains_key(&key) {
            return Err(ProviderRealizationError::DuplicateSetting { key });
        }

        self.settings.insert(key, value.into());
        Ok(self)
    }

    /// Adds one named secret reference.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid or duplicate secret binding name.
    pub fn with_secret(
        mut self,
        name: impl Into<String>,
        reference: SecretReference,
    ) -> Result<Self, ProviderRealizationError> {
        let name = name.into();
        validate_binding_key(&name)?;

        if self.secrets.contains_key(&name) {
            return Err(ProviderRealizationError::DuplicateSecretBinding { name });
        }

        self.secrets.insert(name, reference);
        Ok(self)
    }

    /// Returns the stable provider identity this profile realizes.
    #[must_use]
    pub const fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the installed adapter identifier.
    #[must_use]
    pub const fn adapter(&self) -> &ProviderAdapterId {
        &self.adapter
    }

    /// Returns non-secret adapter settings.
    #[must_use]
    pub const fn settings(&self) -> &BTreeMap<String, String> {
        &self.settings
    }

    /// Returns configured secret references.
    #[must_use]
    pub const fn secrets(&self) -> &BTreeMap<String, SecretReference> {
        &self.secrets
    }

    /// Returns one non-secret setting.
    #[must_use]
    pub fn setting(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(String::as_str)
    }

    /// Returns one secret reference by adapter-defined binding name.
    #[must_use]
    pub fn secret(&self, name: &str) -> Option<&SecretReference> {
        self.secrets.get(name)
    }
}

/// Provider realization profiles available to one Presolve node.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProviderRealizationCatalog {
    profiles: Vec<ProviderRealizationProfile>,
}

impl ProviderRealizationCatalog {
    /// Creates an empty catalog.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    /// Registers one provider realization profile.
    ///
    /// # Errors
    ///
    /// Returns an error when a profile already exists for the same `ProviderId`.
    pub fn register(
        &mut self,
        profile: ProviderRealizationProfile,
    ) -> Result<(), ProviderRealizationError> {
        let provider_id = *profile.provider_id();

        if self
            .profiles
            .iter()
            .any(|registered| *registered.provider_id() == provider_id)
        {
            return Err(ProviderRealizationError::DuplicateProviderProfile { provider_id });
        }

        self.profiles.push(profile);
        self.profiles
            .sort_by(|left, right| left.provider_id().cmp(right.provider_id()));

        Ok(())
    }

    /// Returns the profile for one stable provider identity.
    #[must_use]
    pub fn get(&self, provider_id: ProviderId) -> Option<&ProviderRealizationProfile> {
        self.profiles
            .iter()
            .find(|profile| *profile.provider_id() == provider_id)
    }

    /// Returns all profiles in deterministic provider-ID order.
    #[must_use]
    pub fn profiles(&self) -> &[ProviderRealizationProfile] {
        &self.profiles
    }
}

/// Invalid node-side realization metadata.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProviderRealizationError {
    /// Adapter identifier is malformed.
    #[error("invalid provider adapter identifier `{value}`")]
    InvalidAdapterId {
        /// Rejected identifier.
        value: String,
    },

    /// Setting or secret binding name is malformed.
    #[error("invalid provider realization binding key `{key}`")]
    InvalidBindingKey {
        /// Rejected key.
        key: String,
    },

    /// Secret reference is malformed.
    #[error("invalid environment-variable secret reference `{name}`")]
    InvalidSecretReference {
        /// Rejected environment-variable name.
        name: String,
    },

    /// The profile contains the same setting twice.
    #[error("provider realization setting `{key}` is declared more than once")]
    DuplicateSetting {
        /// Duplicate setting key.
        key: String,
    },

    /// The profile contains the same secret binding twice.
    #[error("provider realization secret binding `{name}` is declared more than once")]
    DuplicateSecretBinding {
        /// Duplicate secret binding name.
        name: String,
    },

    /// The node has multiple profiles for one stable provider ID.
    #[error("provider `{provider_id}` has more than one realization profile")]
    DuplicateProviderProfile {
        /// Duplicate provider identity.
        provider_id: ProviderId,
    },
}

/// Failure while resolving secret material.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SecretResolutionError {
    /// Referenced environment variable does not exist.
    #[error("secret environment variable `{name}` is not present")]
    NotPresent {
        /// Environment-variable name.
        name: String,
    },

    /// Referenced environment variable is not valid Unicode.
    #[error("secret environment variable `{name}` is not valid Unicode")]
    NotUnicode {
        /// Environment-variable name.
        name: String,
    },
}

fn valid_adapter_id(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_ADAPTER_ID_LENGTH {
        return false;
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }

    let mut last = first;

    for character in chars {
        if !character.is_ascii_lowercase()
            && !character.is_ascii_digit()
            && !matches!(character, '.' | '-' | '_' | ':')
        {
            return false;
        }

        last = character;
    }

    last.is_ascii_lowercase() || last.is_ascii_digit()
}

fn validate_binding_key(key: &str) -> Result<(), ProviderRealizationError> {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return Err(ProviderRealizationError::InvalidBindingKey {
            key: key.to_owned(),
        });
    };

    if key.len() > MAX_BINDING_KEY_LENGTH
        || !first.is_ascii_lowercase()
        || !chars.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '_' | '-')
        })
    {
        return Err(ProviderRealizationError::InvalidBindingKey {
            key: key.to_owned(),
        });
    }

    Ok(())
}

fn valid_environment_variable_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_identity_is_canonical_and_vendor_neutral() {
        let adapter =
            ProviderAdapterId::new("cloudflare:kv").expect("adapter identifier should be valid");

        assert_eq!(adapter.as_str(), "cloudflare:kv");
        assert!(ProviderAdapterId::new("Cloudflare KV").is_err());
        assert!(ProviderAdapterId::new("cloudflare:").is_err());
    }

    #[test]
    fn profile_separates_settings_from_secret_references() {
        let provider_id = ProviderId::new();
        let profile = ProviderRealizationProfile::new(
            provider_id,
            ProviderAdapterId::new("cloudflare:kv").expect("adapter should be valid"),
        )
        .with_setting("account_id", "example-account")
        .expect("setting should be valid")
        .with_setting("namespace_id", "example-namespace")
        .expect("setting should be valid")
        .with_secret(
            "api_token",
            SecretReference::environment_variable("PRESOLVE_TEST_API_TOKEN")
                .expect("secret reference should be valid"),
        )
        .expect("secret binding should be valid");

        assert_eq!(profile.provider_id(), &provider_id);
        assert_eq!(profile.setting("account_id"), Some("example-account"));
        assert_eq!(
            profile
                .secret("api_token")
                .expect("secret binding should exist")
                .environment_variable_name(),
            "PRESOLVE_TEST_API_TOKEN"
        );
    }

    #[test]
    fn secret_values_are_redacted_by_debug_formatting() {
        let secret = SecretValue::new("super-secret");

        assert_eq!(secret.expose_secret(), "super-secret");
        assert_eq!(format!("{secret:?}"), "SecretValue([redacted])");
        assert!(!format!("{secret:?}").contains("super-secret"));
    }

    #[test]
    fn duplicate_profile_identity_is_rejected() {
        let provider_id = ProviderId::new();
        let adapter = ProviderAdapterId::new("example:provider").expect("adapter should be valid");
        let mut catalog = ProviderRealizationCatalog::new();

        catalog
            .register(ProviderRealizationProfile::new(
                provider_id,
                adapter.clone(),
            ))
            .expect("first profile should register");

        let error = catalog
            .register(ProviderRealizationProfile::new(provider_id, adapter))
            .expect_err("duplicate provider identity should fail");

        assert_eq!(
            error,
            ProviderRealizationError::DuplicateProviderProfile { provider_id }
        );
    }

    #[test]
    fn duplicate_binding_names_are_rejected() {
        let provider_id = ProviderId::new();
        let adapter = ProviderAdapterId::new("example:provider").expect("adapter should be valid");

        let setting_error = ProviderRealizationProfile::new(provider_id, adapter.clone())
            .with_setting("endpoint", "one")
            .expect("first setting should work")
            .with_setting("endpoint", "two")
            .expect_err("duplicate setting should fail");

        assert!(matches!(
            setting_error,
            ProviderRealizationError::DuplicateSetting { key } if key == "endpoint"
        ));

        let reference =
            SecretReference::environment_variable("TOKEN").expect("reference should be valid");
        let secret_error = ProviderRealizationProfile::new(provider_id, adapter)
            .with_secret("token", reference.clone())
            .expect("first secret should work")
            .with_secret("token", reference)
            .expect_err("duplicate secret binding should fail");

        assert!(matches!(
            secret_error,
            ProviderRealizationError::DuplicateSecretBinding { name } if name == "token"
        ));
    }

    #[test]
    fn catalog_lookup_uses_stable_provider_identity() {
        let selected = ProviderId::new();
        let other = ProviderId::new();
        let adapter = ProviderAdapterId::new("example:provider").expect("adapter should be valid");
        let mut catalog = ProviderRealizationCatalog::new();

        catalog
            .register(ProviderRealizationProfile::new(other, adapter.clone()))
            .expect("other profile should register");
        catalog
            .register(ProviderRealizationProfile::new(selected, adapter))
            .expect("selected profile should register");

        assert_eq!(
            catalog
                .get(selected)
                .expect("selected provider should be found")
                .provider_id(),
            &selected
        );
    }
}
