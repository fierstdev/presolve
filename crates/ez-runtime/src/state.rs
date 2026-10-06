use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

/// Per-instance host state.
///
/// Every component invocation receives its own state. Shared runtime
/// configuration belongs on `Runtime`; application-specific state belongs
/// here.
pub(crate) struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl RuntimeState {
    #[must_use]
    pub(crate) fn new() -> Self {
        let mut builder = WasiCtx::builder();

        Self {
            table: ResourceTable::new(),
            wasi: builder.build(),
        }
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
