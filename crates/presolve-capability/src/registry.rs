use semver::Version;

use crate::{
    CapabilityContract, CapabilityContractError, KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE,
    key_value_contract, object_store_contract,
};

/// Returns whether `interface` is a built-in capability interface understood by
/// this Presolve release.
#[must_use]
pub fn is_known_capability_interface(interface: &str) -> bool {
    matches!(interface, KEY_VALUE_INTERFACE | OBJECT_STORE_INTERFACE)
}

/// Returns the canonical semantic contract for an exact built-in capability
/// interface/version pair.
///
/// `Ok(None)` means either the interface is not built into this Presolve
/// release or the interface is known but the exact version is not. Call
/// [`is_known_capability_interface`] when that distinction matters.
///
/// # Errors
///
/// Returns an error only if construction of a built-in canonical contract
/// violates its own declared invariants.
pub fn canonical_contract_for(
    interface: &str,
    version: &Version,
) -> Result<Option<CapabilityContract>, CapabilityContractError> {
    let contract = match interface {
        KEY_VALUE_INTERFACE => key_value_contract()?,
        OBJECT_STORE_INTERFACE => object_store_contract()?,
        _ => return Ok(None),
    };

    if contract.version() == version {
        Ok(Some(contract))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_interfaces_are_recognized() {
        assert!(is_known_capability_interface(KEY_VALUE_INTERFACE));
        assert!(is_known_capability_interface(OBJECT_STORE_INTERFACE));
        assert!(!is_known_capability_interface("example:custom/service"));
    }

    #[test]
    fn exact_builtin_version_returns_contract() {
        let contract = object_store_contract().expect("object contract should be valid");

        assert_eq!(
            canonical_contract_for(OBJECT_STORE_INTERFACE, contract.version())
                .expect("lookup should succeed"),
            Some(contract)
        );
    }

    #[test]
    fn unknown_builtin_version_has_no_contract() {
        assert_eq!(
            canonical_contract_for(OBJECT_STORE_INTERFACE, &Version::new(9, 0, 0))
                .expect("lookup should succeed"),
            None
        );
    }

    #[test]
    fn custom_interface_has_no_contract() {
        assert_eq!(
            canonical_contract_for("example:custom/service", &Version::new(1, 0, 0))
                .expect("lookup should succeed"),
            None
        );
    }
}
