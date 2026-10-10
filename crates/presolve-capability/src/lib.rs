#![forbid(unsafe_code)]

//! Canonical capability semantics shared by `Presolve`.
//!
//! A capability contract describes what an application-facing capability means
//! independently of how that capability is transported or implemented.
//! Contracts can later drive resolver negotiation, provider conformance,
//! generated Rust bindings, WIT interfaces, TypeScript bindings, and
//! documentation without making any one transport canonical.

mod contract;
mod kv;
mod objects;

pub use contract::{CapabilityContract, CapabilityContractError};
pub use kv::{
    KEY_VALUE_GUARANTEE_EXACT_KEY_IDENTITY, KEY_VALUE_GUARANTEE_MISSING_DISTINCT_FROM_EMPTY,
    KEY_VALUE_GUARANTEE_SET_REPLACES_VALUE, KEY_VALUE_INTERFACE, KEY_VALUE_OPERATION_DELETE,
    KEY_VALUE_OPERATION_GET, KEY_VALUE_OPERATION_SET, key_value_contract,
};
pub use objects::{
    OBJECT_STORE_FEATURE_CONDITIONAL_WRITE, OBJECT_STORE_FEATURE_RANGE_READ,
    OBJECT_STORE_FEATURE_USER_METADATA, OBJECT_STORE_GUARANTEE_DELETE_IDEMPOTENT,
    OBJECT_STORE_GUARANTEE_EXACT_KEY_IDENTITY, OBJECT_STORE_GUARANTEE_OPAQUE_BYTE_CONTENT,
    OBJECT_STORE_GUARANTEE_PUT_REPLACES_OBJECT, OBJECT_STORE_GUARANTEE_READ_AFTER_WRITE,
    OBJECT_STORE_INTERFACE, OBJECT_STORE_OPERATION_DELETE, OBJECT_STORE_OPERATION_GET,
    OBJECT_STORE_OPERATION_HEAD, OBJECT_STORE_OPERATION_LIST, OBJECT_STORE_OPERATION_PUT,
    object_store_contract,
};
