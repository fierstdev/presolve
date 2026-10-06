#![forbid(unsafe_code)]

//! Portable application execution for `EdgeZero`.
//!
//! `edgezero-runtime` embeds Wasmtime and executes applications implementing
//! the versioned `EdgeZero` Component Model world.
//!
//! Applications interact with the host through explicitly defined Component
//! Model interfaces rather than ambient operating-system access.

mod application;
mod bindings;
mod error;
mod limits;
mod runtime;
mod state;

pub use application::CompiledApplication;
pub use error::RuntimeError;
pub use limits::{
    DEFAULT_FUEL, DEFAULT_MAX_INSTANCES, DEFAULT_MAX_MEMORIES, DEFAULT_MAX_MEMORY_BYTES,
    DEFAULT_MAX_TABLE_ELEMENTS, DEFAULT_MAX_TABLES, MIB, RuntimeLimits,
};
pub use runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_initializes() {
        Runtime::new().expect("runtime should initialize");
    }

    #[test]
    fn runtime_uses_default_limits() {
        let runtime = Runtime::new().expect("runtime should initialize");

        assert_eq!(runtime.limits(), RuntimeLimits::default());
    }

    #[test]
    fn runtime_accepts_custom_limits() {
        let limits = RuntimeLimits::new()
            .with_fuel(500_000)
            .with_max_memory_bytes(16 * MIB);

        let runtime = Runtime::with_limits(limits).expect("runtime should initialize");

        assert_eq!(runtime.limits(), limits);
    }

    #[test]
    fn invalid_component_is_rejected() {
        let runtime = Runtime::new().expect("runtime should initialize");

        let result = runtime.compile(b"not a WebAssembly Component");

        assert!(matches!(result, Err(RuntimeError::ComponentCompile(_))));
    }
}
