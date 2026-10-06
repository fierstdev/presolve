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
mod runtime;
mod state;

pub use application::CompiledApplication;
pub use error::RuntimeError;
pub use runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_initializes() {
        Runtime::new().expect("runtime should initialize");
    }

    #[test]
    fn invalid_component_is_rejected() {
        let runtime = Runtime::new().expect("runtime should initialize");

        let result = runtime.compile(b"not a WebAssembly Component");

        assert!(matches!(result, Err(RuntimeError::ComponentCompile(_))));
    }
}
