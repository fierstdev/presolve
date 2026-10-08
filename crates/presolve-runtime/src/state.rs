use std::sync::Arc;

use presolve_provider_sdk::KeyValueProvider;
use wasmtime::{ResourceLimiter, StoreLimits, component::ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::RuntimeLimits;

/// Per-instance host state.
///
/// Every component invocation receives its own state. Shared providers are
/// referenced through thread-safe provider handles.
pub(crate) struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
    limits: StoreLimits,
    key_value_provider: Arc<dyn KeyValueProvider>,
}

impl RuntimeState {
    #[must_use]
    pub(crate) fn new(
        runtime_limits: RuntimeLimits,
        key_value_provider: Arc<dyn KeyValueProvider>,
    ) -> Self {
        let mut builder = WasiCtx::builder();

        Self {
            table: ResourceTable::new(),
            wasi: builder.build(),
            limits: runtime_limits.store_limits(),
            key_value_provider,
        }
    }

    pub(crate) fn limits_mut(&mut self) -> &mut dyn ResourceLimiter {
        &mut self.limits
    }

    pub(crate) fn key_value_provider(&self) -> &dyn KeyValueProvider {
        self.key_value_provider.as_ref()
    }
}

impl WasiView for RuntimeState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}
