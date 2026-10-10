use std::{sync::Arc, thread, time::Duration};

use presolve_provider_sdk::{
    KeyValueProvider, ObjectStoreProvider, UnavailableKeyValueProvider,
    UnavailableObjectStoreProvider,
};
use wasmtime::{
    Config, Engine, Store, Trap,
    component::{Component, HasSelf, Linker},
};

use crate::{
    CompiledApplication, EPOCH_TICK_INTERVAL, RuntimeError, RuntimeLimits, bindings::Application,
    state::RuntimeState,
};

/// Builder for a `Presolve` runtime.
///
/// Providers are configured on the runtime rather than on application
/// components. This allows the same component artifact to execute against
/// different environment-specific implementations.
pub struct RuntimeBuilder {
    limits: RuntimeLimits,
    key_value_provider: Arc<dyn KeyValueProvider>,
    object_store_provider: Arc<dyn ObjectStoreProvider>,
}

impl RuntimeBuilder {
    /// Creates a runtime builder with default limits and unavailable optional
    /// capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self {
            limits: RuntimeLimits::default(),
            key_value_provider: Arc::new(UnavailableKeyValueProvider),
            object_store_provider: Arc::new(UnavailableObjectStoreProvider),
        }
    }

    /// Sets the runtime resource limits.
    #[must_use]
    pub fn limits(mut self, limits: RuntimeLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Supplies the provider for `presolve:kv/store`.
    #[must_use]
    pub fn key_value_provider(mut self, provider: Arc<dyn KeyValueProvider>) -> Self {
        self.key_value_provider = provider;
        self
    }

    /// Supplies the provider for `presolve:objects/store`.
    ///
    /// The provider is materialized into runtime configuration here. Guest
    /// Component Model bindings are added separately.
    #[must_use]
    pub fn object_store_provider(mut self, provider: Arc<dyn ObjectStoreProvider>) -> Self {
        self.object_store_provider = provider;
        self
    }

    /// Builds the runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying Wasmtime runtime, linker, capability
    /// interfaces, or epoch ticker cannot be initialized.
    pub fn build(self) -> Result<Runtime, RuntimeError> {
        Runtime::build(
            self.limits,
            self.key_value_provider,
            self.object_store_provider,
        )
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// `Presolve` WebAssembly Component runtime.
///
/// A runtime owns the Wasmtime engine, host-side interface linker, configured
/// capability providers, and default execution policy. Individual application
/// invocations receive separate stores and state.
pub struct Runtime {
    engine: Engine,
    linker: Linker<RuntimeState>,
    limits: RuntimeLimits,
    key_value_provider: Arc<dyn KeyValueProvider>,
    // Materialized now; consumed by the object-storage host binding in 13.9.
    _object_store_provider: Arc<dyn ObjectStoreProvider>,
}

impl Runtime {
    /// Creates an `Presolve` runtime with default resource limits and no
    /// configured optional capability providers.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasmtime cannot initialize, the epoch ticker cannot
    /// start, or required host interfaces cannot be registered.
    pub fn new() -> Result<Self, RuntimeError> {
        Self::builder().build()
    }

    /// Creates a configurable runtime builder.
    #[must_use]
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    /// Creates an `Presolve` runtime with explicit resource limits.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasmtime cannot initialize, the epoch ticker cannot
    /// start, or required host interfaces cannot be registered.
    pub fn with_limits(limits: RuntimeLimits) -> Result<Self, RuntimeError> {
        Self::builder().limits(limits).build()
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
    /// Fuel, memory limits, and epoch deadlines are therefore isolated between
    /// invocations. Capability provider implementations are shared through
    /// thread-safe provider handles.
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
        let mut store = Store::new(
            &self.engine,
            RuntimeState::new(self.limits, Arc::clone(&self.key_value_provider)),
        );

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

    fn build(
        limits: RuntimeLimits,
        key_value_provider: Arc<dyn KeyValueProvider>,
        object_store_provider: Arc<dyn ObjectStoreProvider>,
    ) -> Result<Self, RuntimeError> {
        let mut config = Config::new();

        config.wasm_component_model(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);

        let engine = Engine::new(&config).map_err(RuntimeError::Engine)?;

        start_epoch_ticker(&engine)?;

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(RuntimeError::WasiLinker)?;

        crate::bindings::kv::presolve::kv::store::add_to_linker::<_, HasSelf<_>>(
            &mut linker,
            |state| state,
        )
        .map_err(RuntimeError::CapabilityLinker)?;

        Ok(Self {
            engine,
            linker,
            limits,
            key_value_provider,
            _object_store_provider: object_store_provider,
        })
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new().expect("default Presolve runtime must initialize")
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
        .name("presolve-epoch".to_owned())
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
