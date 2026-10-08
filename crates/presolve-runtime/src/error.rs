use thiserror::Error;

/// Error produced while compiling or executing a `Presolve` application.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// The Wasmtime engine could not be initialized.
    #[error("failed to initialize Presolve runtime: {0}")]
    Engine(#[source] wasmtime::Error),

    /// The runtime epoch ticker could not be started.
    #[error("failed to start Presolve epoch ticker: {0}")]
    EpochTicker(#[source] std::io::Error),

    /// Standard WASI host interfaces could not be registered.
    #[error("failed to configure WASI host interfaces: {0}")]
    WasiLinker(#[source] wasmtime::Error),

    /// An `Presolve` capability interface could not be registered.
    #[error("failed to configure Presolve capability interfaces: {0}")]
    CapabilityLinker(#[source] wasmtime::Error),

    /// The supplied bytes are not a valid executable component.
    #[error("failed to compile WebAssembly Component: {0}")]
    ComponentCompile(#[source] wasmtime::Error),

    /// Per-invocation host state could not be configured.
    #[error("failed to configure Presolve application store: {0}")]
    StoreConfiguration(#[source] wasmtime::Error),

    /// The component exhausted its deterministic execution budget.
    #[error("Presolve application exhausted its execution fuel budget")]
    FuelExhausted(#[source] wasmtime::Error),

    /// The component exceeded its wall-clock execution deadline.
    #[error("Presolve application exceeded its execution deadline")]
    DeadlineExceeded(#[source] wasmtime::Error),

    /// The component could not be instantiated.
    #[error("failed to instantiate Presolve application: {0}")]
    Instantiate(#[source] wasmtime::Error),

    /// The component trapped or otherwise failed during invocation.
    #[error("Presolve application invocation failed: {0}")]
    Invocation(#[source] wasmtime::Error),
}
