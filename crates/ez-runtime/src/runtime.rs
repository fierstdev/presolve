use wasmtime::{
    Config, Engine, Store, Trap,
    component::{Component, Linker},
};

use crate::{
    CompiledApplication, RuntimeError, RuntimeLimits, bindings::Application, state::RuntimeState,
};

/// `EdgeZero` WebAssembly Component runtime.
///
/// A runtime owns the Wasmtime engine, host-side interface linker, and default
/// execution policy. Individual application invocations receive separate
/// stores and state.
pub struct Runtime {
    engine: Engine,
    linker: Linker<RuntimeState>,
    limits: RuntimeLimits,
}

impl Runtime {
    /// Creates an `EdgeZero` runtime with default resource limits.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasmtime cannot initialize or the required WASI
    /// interfaces cannot be registered.
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_limits(RuntimeLimits::default())
    }

    /// Creates an `EdgeZero` runtime with explicit resource limits.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasmtime cannot initialize or the required WASI
    /// interfaces cannot be registered.
    pub fn with_limits(limits: RuntimeLimits) -> Result<Self, RuntimeError> {
        let mut config = Config::new();

        config.wasm_component_model(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config).map_err(RuntimeError::Engine)?;

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(RuntimeError::WasiLinker)?;

        Ok(Self {
            engine,
            linker,
            limits,
        })
    }

    /// Returns the host-enforced invocation limits.
    #[must_use]
    pub const fn limits(&self) -> RuntimeLimits {
        self.limits
    }

    /// Compiles WebAssembly Component bytes for this runtime.
    ///
    /// Compilation validates the component and produces an executable
    /// Wasmtime component associated with this runtime's engine.
    ///
    /// # Errors
    ///
    /// Returns an error when `bytes` do not contain a valid component that
    /// Wasmtime can compile.
    pub fn compile(&self, bytes: &[u8]) -> Result<CompiledApplication, RuntimeError> {
        let component =
            Component::new(&self.engine, bytes).map_err(RuntimeError::ComponentCompile)?;

        Ok(CompiledApplication { component })
    }

    /// Executes the application's `run` export.
    ///
    /// A fresh Wasmtime store and host state are created for every invocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be configured, the application
    /// cannot be instantiated, the execution budget is exhausted, or the
    /// application otherwise traps.
    pub fn run(
        &self,
        application: &CompiledApplication,
        input: &str,
    ) -> Result<String, RuntimeError> {
        let mut store = Store::new(&self.engine, RuntimeState::new(self.limits));

        store.limiter(RuntimeState::limits_mut);

        store
            .set_fuel(self.limits.fuel())
            .map_err(RuntimeError::StoreConfiguration)?;

        let bindings = Application::instantiate(&mut store, application.component(), &self.linker)
            .map_err(RuntimeError::Instantiate)?;

        bindings
            .call_run(&mut store, input)
            .map_err(map_invocation_error)
    }

    /// Compiles and immediately executes component bytes.
    ///
    /// This convenience API is useful for development and tests. Production
    /// callers should generally compile once and reuse the resulting
    /// [`CompiledApplication`].
    ///
    /// # Errors
    ///
    /// Returns any error encountered during compilation, instantiation, store
    /// configuration, or invocation.
    pub fn run_bytes(&self, bytes: &[u8], input: &str) -> Result<String, RuntimeError> {
        let application = self.compile(bytes)?;
        self.run(&application, input)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new().expect("default EdgeZero runtime must initialize")
    }
}

fn map_invocation_error(error: wasmtime::Error) -> RuntimeError {
    if matches!(error.downcast_ref::<Trap>(), Some(Trap::OutOfFuel)) {
        RuntimeError::FuelExhausted(error)
    } else {
        RuntimeError::Invocation(error)
    }
}
