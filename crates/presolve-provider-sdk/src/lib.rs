#![forbid(unsafe_code)]

//! Provider interfaces implemented by `Presolve` capability backends.
//!
//! Application components depend on versioned WIT capabilities. Host-side
//! infrastructure implements the corresponding provider traits from this
//! crate.

mod kv;

pub use kv::{KeyValueError, KeyValueErrorCode, KeyValueProvider, UnavailableKeyValueProvider};
