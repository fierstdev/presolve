use thiserror::Error;

/// Error produced while compiling or executing an `EdgeZero` application.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// The Wasmtime engine could not be initialized.
    #[error("failed to initialize EdgeZero runtime: {0}")]
    Engine(#[source] wasmtime::Error),

    /// Standard WASI host interfaces could not be registered.
    #[error("failed to configure WASI host interfaces: {0}")]
    WasiLinker(#[source] wasmtime::Error),

    /// The supplied bytes are not a valid executable component.
    #[error("failed to compile WebAssembly Component: {0}")]
    ComponentCompile(#[source] wasmtime::Error),

    /// The component could not be instantiated.
    #[error("failed to instantiate EdgeZero application: {0}")]
    Instantiate(#[source] wasmtime::Error),

    /// The component trapped or otherwise failed during invocation.
    #[error("EdgeZero application invocation failed: {0}")]
    Invocation(#[source] wasmtime::Error),
}
