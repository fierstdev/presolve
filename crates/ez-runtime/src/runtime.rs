use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker},
};

use crate::{CompiledApplication, RuntimeError, bindings::Application, state::RuntimeState};

/// `EdgeZero` WebAssembly Component runtime.
///
/// A runtime owns the Wasmtime engine and the host-side interface linker.
/// Individual application invocations receive separate stores and state.
pub struct Runtime {
    engine: Engine,
    linker: Linker<RuntimeState>,
}

impl Runtime {
    /// Creates an `EdgeZero` runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasmtime cannot initialize or the required WASI
    /// interfaces cannot be registered.
    pub fn new() -> Result<Self, RuntimeError> {
        let mut config = Config::new();

        config.wasm_component_model(true);

        let engine = Engine::new(&config).map_err(RuntimeError::Engine)?;

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(RuntimeError::WasiLinker)?;

        Ok(Self { engine, linker })
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
    /// Returns an error if the application cannot be instantiated or its
    /// exported function traps or otherwise fails.
    pub fn run(
        &self,
        application: &CompiledApplication,
        input: &str,
    ) -> Result<String, RuntimeError> {
        let mut store = Store::new(&self.engine, RuntimeState::new());

        let bindings = Application::instantiate(&mut store, application.component(), &self.linker)
            .map_err(RuntimeError::Instantiate)?;

        bindings
            .call_run(&mut store, input)
            .map_err(RuntimeError::Invocation)
    }

    /// Compiles and immediately executes component bytes.
    ///
    /// This convenience API is useful for development and tests. Production
    /// callers should generally compile once and reuse the resulting
    /// [`CompiledApplication`].
    ///
    /// # Errors
    ///
    /// Returns any error encountered during compilation, instantiation, or
    /// invocation.
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
