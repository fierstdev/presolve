use std::{thread, time::Duration};

use wasmtime::{
    Config, Engine, Store, Trap,
    component::{Component, Linker},
};

use crate::{
    CompiledApplication, EPOCH_TICK_INTERVAL, RuntimeError, RuntimeLimits, bindings::Application,
    state::RuntimeState,
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
    /// Returns an error if Wasmtime cannot initialize, the epoch ticker cannot
    /// start, or the required WASI interfaces cannot be registered.
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_limits(RuntimeLimits::default())
    }

    /// Creates an `EdgeZero` runtime with explicit resource limits.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasmtime cannot initialize, the epoch ticker cannot
    /// start, or the required WASI interfaces cannot be registered.
    pub fn with_limits(limits: RuntimeLimits) -> Result<Self, RuntimeError> {
        let mut config = Config::new();

        config.wasm_component_model(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);

        let engine = Engine::new(&config).map_err(RuntimeError::Engine)?;

        start_epoch_ticker(&engine)?;

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
    /// Fuel and epoch deadlines are therefore isolated between invocations.
    ///
    /// # Errors
    ///
    /// Returns an error if the store cannot be configured, the application
    /// cannot be instantiated, an execution limit is reached, or the
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

        store.epoch_deadline_trap();
        store.set_epoch_deadline(self.limits.epoch_deadline_ticks());

        let bindings = Application::instantiate(&mut store, application.component(), &self.linker)
            .map_err(|error| map_execution_error(error, ExecutionPhase::Instantiation))?;

        bindings
            .call_run(&mut store, input)
            .map_err(|error| map_execution_error(error, ExecutionPhase::Invocation))
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

#[derive(Debug, Clone, Copy)]
enum ExecutionPhase {
    Instantiation,
    Invocation,
}

fn map_execution_error(error: wasmtime::Error, phase: ExecutionPhase) -> RuntimeError {
    match error.downcast_ref::<Trap>() {
        Some(Trap::OutOfFuel) => RuntimeError::FuelExhausted(error),
        Some(Trap::Interrupt) => RuntimeError::DeadlineExceeded(error),
        _ => match phase {
            ExecutionPhase::Instantiation => RuntimeError::Instantiate(error),
            ExecutionPhase::Invocation => RuntimeError::Invocation(error),
        },
    }
}

fn start_epoch_ticker(engine: &Engine) -> Result<(), RuntimeError> {
    let weak_engine = engine.weak();

    thread::Builder::new()
        .name("edgezero-epoch".to_owned())
        .spawn(move || {
            epoch_ticker_loop(&weak_engine, EPOCH_TICK_INTERVAL);
        })
        .map(|_| ())
        .map_err(RuntimeError::EpochTicker)
}

fn epoch_ticker_loop(engine: &wasmtime::EngineWeak, interval: Duration) {
    loop {
        thread::sleep(interval);

        let Some(engine) = engine.upgrade() else {
            break;
        };

        engine.increment_epoch();
    }
}
