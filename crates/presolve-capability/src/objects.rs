use semver::Version;

use crate::{CapabilityContract, CapabilityContractError};

/// Canonical capability interface identity for object storage.
pub const OBJECT_STORE_INTERFACE: &str = "presolve:objects/store";

/// Reads a complete object by exact key.
pub const OBJECT_STORE_OPERATION_GET: &str = "get";

/// Stores a complete object at an exact key.
pub const OBJECT_STORE_OPERATION_PUT: &str = "put";

/// Removes an object at an exact key.
pub const OBJECT_STORE_OPERATION_DELETE: &str = "delete";

/// Reads object system metadata without returning the object body.
pub const OBJECT_STORE_OPERATION_HEAD: &str = "head";

/// Enumerates object keys by prefix.
///
/// Baseline list ordering is intentionally unspecified. Providers must not
/// imply lexical ordering or snapshot consistency unless a future semantic
/// feature explicitly requires it.
pub const OBJECT_STORE_OPERATION_LIST: &str = "list";

/// Object keys are opaque exact identifiers and are not filesystem paths.
pub const OBJECT_STORE_GUARANTEE_EXACT_KEY_IDENTITY: &str = "exact-key-identity";

/// Object bodies are arbitrary opaque bytes rather than text values.
pub const OBJECT_STORE_GUARANTEE_OPAQUE_BYTE_CONTENT: &str = "opaque-byte-content";

/// A successful `put` replaces the complete logical object at its key.
///
/// The baseline capability does not define partial-object mutation.
pub const OBJECT_STORE_GUARANTEE_PUT_REPLACES_OBJECT: &str = "put-replaces-object";

/// Successful writes are immediately observable through subsequent exact-key
/// reads using the same capability.
pub const OBJECT_STORE_GUARANTEE_READ_AFTER_WRITE: &str = "read-after-write";

/// Deleting an already-missing key is safe to repeat.
///
/// Providers may report whether an object previously existed, but absence alone
/// must not make repeated deletion unsafe.
pub const OBJECT_STORE_GUARANTEE_DELETE_IDEMPOTENT: &str = "delete-idempotent";

/// Provider supports reading an explicit contiguous byte range of an object.
pub const OBJECT_STORE_FEATURE_RANGE_READ: &str = "range-read";

/// Provider supports preconditioned object mutation.
///
/// This feature is intended to cover portable create-if-absent and
/// revision-matched replacement semantics without exposing provider-specific
/// `ETag` or generation APIs in the application contract.
pub const OBJECT_STORE_FEATURE_CONDITIONAL_WRITE: &str = "conditional-write";

/// Provider round-trips application-defined object metadata.
pub const OBJECT_STORE_FEATURE_USER_METADATA: &str = "user-metadata";

/// Returns the canonical object-storage capability contract.
///
/// This is the first richer capability contract used to exercise semantic
/// provider negotiation beyond interface-and-version matching.
///
/// # Errors
///
/// Returns an error only if Presolve's built-in static contract definition is
/// internally invalid.
pub fn object_store_contract() -> Result<CapabilityContract, CapabilityContractError> {
    CapabilityContract::new(
        OBJECT_STORE_INTERFACE,
        Version::new(0, 1, 0),
        [
            OBJECT_STORE_OPERATION_GET,
            OBJECT_STORE_OPERATION_PUT,
            OBJECT_STORE_OPERATION_DELETE,
            OBJECT_STORE_OPERATION_HEAD,
            OBJECT_STORE_OPERATION_LIST,
        ],
        [
            OBJECT_STORE_GUARANTEE_EXACT_KEY_IDENTITY,
            OBJECT_STORE_GUARANTEE_OPAQUE_BYTE_CONTENT,
            OBJECT_STORE_GUARANTEE_PUT_REPLACES_OBJECT,
            OBJECT_STORE_GUARANTEE_READ_AFTER_WRITE,
            OBJECT_STORE_GUARANTEE_DELETE_IDEMPOTENT,
        ],
        [
            OBJECT_STORE_FEATURE_RANGE_READ,
            OBJECT_STORE_FEATURE_CONDITIONAL_WRITE,
            OBJECT_STORE_FEATURE_USER_METADATA,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_object_store_contract_is_valid_and_canonical() {
        let contract = object_store_contract().expect("object-store contract should be valid");

        assert_eq!(contract.interface(), OBJECT_STORE_INTERFACE);
        assert_eq!(contract.version(), &Version::new(0, 1, 0));
        assert_eq!(
            contract.operations(),
            ["delete", "get", "head", "list", "put"]
        );
        assert_eq!(
            contract.guarantees(),
            [
                "delete-idempotent",
                "exact-key-identity",
                "opaque-byte-content",
                "put-replaces-object",
                "read-after-write"
            ]
        );
        assert_eq!(
            contract.optional_features(),
            ["conditional-write", "range-read", "user-metadata"]
        );
    }
}
