#![forbid(unsafe_code)]

//! Provider interfaces implemented by `Presolve` capability backends.
//!
//! Capability semantics are defined independently of their binding transport.
//! Host-side infrastructure implements provider traits from this crate, while
//! WIT remains one binding target for WebAssembly Component workloads.

mod kv;
mod objects;

pub use kv::{
    KEY_VALUE_INTERFACE, KeyValueError, KeyValueErrorCode, KeyValueProvider,
    UnavailableKeyValueProvider,
};
pub use objects::{
    OBJECT_STORE_INTERFACE, ObjectInfo, ObjectRead, ObjectStoreError, ObjectStoreErrorCode,
    ObjectStoreProvider, UnavailableObjectStoreProvider,
};
