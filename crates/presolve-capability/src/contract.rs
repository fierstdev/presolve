use std::collections::BTreeSet;

use semver::Version;
use serde::Serialize;
use thiserror::Error;

/// Canonical semantic definition of one `Presolve` capability.
///
/// The contract records:
///
/// - the stable capability interface identity,
/// - the semantic contract version,
/// - operations every conforming implementation exposes,
/// - guarantees every conforming implementation must uphold, and
/// - optional semantic features providers may advertise during negotiation.
///
/// Binding transports such as WIT, native Rust traits, HTTP, or platform
/// bindings are intentionally absent from this representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityContract {
    interface: String,
    version: Version,
    operations: Vec<String>,
    guarantees: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    optional_features: Vec<String>,
}

impl CapabilityContract {
    /// Creates and canonicalizes a capability contract.
    ///
    /// Semantic member identifiers use lowercase kebab-case. Input ordering is
    /// not semantically significant; operations, guarantees, and optional
    /// features are stored in lexical order.
    ///
    /// # Errors
    ///
    /// Returns an error when the interface or semantic member identifiers are
    /// invalid, no operations are declared, or a semantic member is duplicated.
    pub fn new<I, O, G, F>(
        interface: I,
        version: Version,
        operations: O,
        guarantees: G,
        optional_features: F,
    ) -> Result<Self, CapabilityContractError>
    where
        I: Into<String>,
        O: IntoIterator,
        O::Item: Into<String>,
        G: IntoIterator,
        G::Item: Into<String>,
        F: IntoIterator,
        F::Item: Into<String>,
    {
        let interface = interface.into();

        if !valid_capability_interface(&interface) {
            return Err(CapabilityContractError::InvalidInterface(interface));
        }

        let operations = canonical_operations(operations)?;
        let guarantees = canonical_guarantees(guarantees)?;
        let optional_features = canonical_optional_features(optional_features)?;

        if operations.is_empty() {
            return Err(CapabilityContractError::MissingOperations);
        }

        Ok(Self {
            interface,
            version,
            operations,
            guarantees,
            optional_features,
        })
    }

    /// Returns the stable capability interface identity.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the semantic capability contract version.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the canonical operation identifiers.
    #[must_use]
    pub fn operations(&self) -> &[String] {
        &self.operations
    }

    /// Returns semantic guarantees required of every conforming provider.
    #[must_use]
    pub fn guarantees(&self) -> &[String] {
        &self.guarantees
    }

    /// Returns optional semantic features a provider may advertise.
    #[must_use]
    pub fn optional_features(&self) -> &[String] {
        &self.optional_features
    }

    /// Serializes the canonical semantic contract as deterministic JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Failure while constructing a canonical capability contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CapabilityContractError {
    /// The interface does not follow the canonical capability identifier shape.
    #[error("invalid capability interface `{0}`")]
    InvalidInterface(String),

    /// A capability without operations cannot be invoked.
    #[error("capability contract must declare at least one operation")]
    MissingOperations,

    /// An operation identifier is malformed.
    #[error("invalid capability operation identifier `{0}`")]
    InvalidOperation(String),

    /// The same operation was declared more than once.
    #[error("capability operation `{0}` is declared more than once")]
    DuplicateOperation(String),

    /// A mandatory semantic guarantee identifier is malformed.
    #[error("invalid capability guarantee identifier `{0}`")]
    InvalidGuarantee(String),

    /// The same guarantee was declared more than once.
    #[error("capability guarantee `{0}` is declared more than once")]
    DuplicateGuarantee(String),

    /// An optional semantic feature identifier is malformed.
    #[error("invalid capability optional feature identifier `{0}`")]
    InvalidOptionalFeature(String),

    /// The same optional feature was declared more than once.
    #[error("capability optional feature `{0}` is declared more than once")]
    DuplicateOptionalFeature(String),
}

fn canonical_operations<O>(values: O) -> Result<Vec<String>, CapabilityContractError>
where
    O: IntoIterator,
    O::Item: Into<String>,
{
    canonical_members(
        values,
        CapabilityContractError::InvalidOperation,
        CapabilityContractError::DuplicateOperation,
    )
}

fn canonical_guarantees<G>(values: G) -> Result<Vec<String>, CapabilityContractError>
where
    G: IntoIterator,
    G::Item: Into<String>,
{
    canonical_members(
        values,
        CapabilityContractError::InvalidGuarantee,
        CapabilityContractError::DuplicateGuarantee,
    )
}

fn canonical_optional_features<F>(values: F) -> Result<Vec<String>, CapabilityContractError>
where
    F: IntoIterator,
    F::Item: Into<String>,
{
    canonical_members(
        values,
        CapabilityContractError::InvalidOptionalFeature,
        CapabilityContractError::DuplicateOptionalFeature,
    )
}

fn canonical_members<I>(
    values: I,
    invalid: fn(String) -> CapabilityContractError,
    duplicate: fn(String) -> CapabilityContractError,
) -> Result<Vec<String>, CapabilityContractError>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut canonical = BTreeSet::new();

    for value in values {
        let value = value.into();

        if !valid_kebab_segment(&value) {
            return Err(invalid(value));
        }

        if !canonical.insert(value.clone()) {
            return Err(duplicate(value));
        }
    }

    Ok(canonical.into_iter().collect())
}

fn valid_capability_interface(interface: &str) -> bool {
    let Some((namespace, remainder)) = interface.split_once(':') else {
        return false;
    };

    if remainder.contains(':') {
        return false;
    }

    let Some((package, name)) = remainder.split_once('/') else {
        return false;
    };

    if name.contains('/') {
        return false;
    }

    valid_kebab_segment(namespace) && valid_kebab_segment(package) && valid_kebab_segment(name)
}

fn valid_kebab_segment(segment: &str) -> bool {
    if segment.is_empty() || segment.len() > 63 {
        return false;
    }

    let bytes = segment.as_bytes();

    let Some(first) = bytes.first() else {
        return false;
    };

    let Some(last) = bytes.last() else {
        return false;
    };

    first.is_ascii_lowercase()
        && last.is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_member_order_does_not_change_canonical_contract() {
        let first = CapabilityContract::new(
            "presolve:test/store",
            Version::new(1, 2, 3),
            ["put", "get", "delete"],
            ["read-after-write", "exact-key-identity"],
            ["range-read", "conditional-write"],
        )
        .expect("first capability contract should be valid");

        let second = CapabilityContract::new(
            "presolve:test/store",
            Version::new(1, 2, 3),
            ["delete", "get", "put"],
            ["exact-key-identity", "read-after-write"],
            ["conditional-write", "range-read"],
        )
        .expect("second capability contract should be valid");

        assert_eq!(first, second);
        assert_eq!(
            first
                .canonical_bytes()
                .expect("first contract should serialize"),
            second
                .canonical_bytes()
                .expect("second contract should serialize")
        );
    }

    #[test]
    fn duplicate_semantic_member_is_rejected() {
        let error = CapabilityContract::new(
            "presolve:test/store",
            Version::new(1, 0, 0),
            ["get", "get"],
            std::iter::empty::<&str>(),
            std::iter::empty::<&str>(),
        )
        .expect_err("duplicate operation should fail");

        assert_eq!(
            error,
            CapabilityContractError::DuplicateOperation("get".to_owned())
        );
    }

    #[test]
    fn malformed_interface_is_rejected() {
        let error = CapabilityContract::new(
            "objects",
            Version::new(1, 0, 0),
            ["get"],
            std::iter::empty::<&str>(),
            std::iter::empty::<&str>(),
        )
        .expect_err("malformed interface should fail");

        assert_eq!(
            error,
            CapabilityContractError::InvalidInterface("objects".to_owned())
        );
    }

    #[test]
    fn capability_requires_at_least_one_operation() {
        let error = CapabilityContract::new(
            "presolve:test/store",
            Version::new(1, 0, 0),
            std::iter::empty::<&str>(),
            std::iter::empty::<&str>(),
            std::iter::empty::<&str>(),
        )
        .expect_err("operationless capability should fail");

        assert_eq!(error, CapabilityContractError::MissingOperations);
    }
}
