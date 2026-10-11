#![forbid(unsafe_code)]

//! Reusable conformance assertions for Presolve capability providers.
//!
//! Provider implementations should run these suites in their own tests.
//! The suites exercise portable baseline semantics only. Optional semantic
//! features require feature-specific interfaces and conformance suites before
//! providers should advertise them.

mod kv;
mod objects;

pub use kv::assert_key_value_conformance;
pub use objects::assert_object_store_conformance;
