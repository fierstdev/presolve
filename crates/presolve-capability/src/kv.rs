use semver::Version;

use crate::{CapabilityContract, CapabilityContractError};

/// Canonical capability interface identity for key/value storage.
pub const KEY_VALUE_INTERFACE: &str = "presolve:kv/store";

/// Reads the value stored at an exact key.
pub const KEY_VALUE_OPERATION_GET: &str = "get";

/// Stores a complete value at an exact key.
pub const KEY_VALUE_OPERATION_SET: &str = "set";

/// Removes the value at an exact key.
pub const KEY_VALUE_OPERATION_DELETE: &str = "delete";

/// Keys are opaque exact identifiers and are not path-normalized.
pub const KEY_VALUE_GUARANTEE_EXACT_KEY_IDENTITY: &str = "exact-key-identity";

/// A missing key is semantically distinct from a present empty value.
pub const KEY_VALUE_GUARANTEE_MISSING_DISTINCT_FROM_EMPTY: &str = "missing-distinct-from-empty";

/// Setting an existing key replaces its complete previous value.
pub const KEY_VALUE_GUARANTEE_SET_REPLACES_VALUE: &str = "set-replaces-value";

/// Returns the canonical key/value capability contract.
///
/// # Errors
///
/// Returns an error only if Presolve's built-in static contract definition is
/// internally invalid.
pub fn key_value_contract() -> Result<CapabilityContract, CapabilityContractError> {
    CapabilityContract::new(
        KEY_VALUE_INTERFACE,
        Version::new(0, 1, 0),
        [
            KEY_VALUE_OPERATION_GET,
            KEY_VALUE_OPERATION_SET,
            KEY_VALUE_OPERATION_DELETE,
        ],
        [
            KEY_VALUE_GUARANTEE_EXACT_KEY_IDENTITY,
            KEY_VALUE_GUARANTEE_MISSING_DISTINCT_FROM_EMPTY,
            KEY_VALUE_GUARANTEE_SET_REPLACES_VALUE,
        ],
        std::iter::empty::<&str>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_key_value_contract_is_valid() {
        let contract = key_value_contract().expect("key/value contract should be valid");

        assert_eq!(contract.interface(), KEY_VALUE_INTERFACE);
        assert_eq!(contract.version(), &Version::new(0, 1, 0));
        assert_eq!(contract.operations(), ["delete", "get", "set"]);
        assert_eq!(
            contract.guarantees(),
            [
                "exact-key-identity",
                "missing-distinct-from-empty",
                "set-replaces-value"
            ]
        );
        assert_eq!(contract.optional_features(), &[] as &[String]);
    }
}
