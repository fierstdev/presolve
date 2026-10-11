use std::{collections::BTreeMap, sync::Arc};

use presolve_provider_sdk::{KeyValueProvider, ObjectRead, ObjectStoreProvider};
use wasmtime::{ResourceLimiter, StoreLimits, component::ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::RuntimeLimits;

pub(crate) struct PendingObjectWrite {
    pub(crate) key: String,
    pub(crate) size: u64,
    pub(crate) body: Vec<u8>,
}

/// Per-instance host state.
///
/// Every component invocation receives its own state. Shared providers are
/// referenced through thread-safe provider handles.
pub(crate) struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
    limits: StoreLimits,
    key_value_provider: Arc<dyn KeyValueProvider>,
    object_store_provider: Arc<dyn ObjectStoreProvider>,
    object_reads: BTreeMap<u64, ObjectRead>,
    object_writes: BTreeMap<u64, PendingObjectWrite>,
    max_object_buffer_bytes: usize,
    object_buffered_write_bytes: usize,
    max_object_sessions: usize,
    next_object_handle: u64,
}

impl RuntimeState {
    #[must_use]
    pub(crate) fn new(
        runtime_limits: RuntimeLimits,
        key_value_provider: Arc<dyn KeyValueProvider>,
        object_store_provider: Arc<dyn ObjectStoreProvider>,
    ) -> Self {
        let mut builder = WasiCtx::builder();

        Self {
            table: ResourceTable::new(),
            wasi: builder.build(),
            limits: runtime_limits.store_limits(),
            key_value_provider,
            object_store_provider,
            object_reads: BTreeMap::new(),
            object_writes: BTreeMap::new(),
            max_object_buffer_bytes: runtime_limits.max_object_buffer_bytes(),
            object_buffered_write_bytes: 0,
            max_object_sessions: runtime_limits.max_object_sessions(),
            next_object_handle: 1,
        }
    }

    pub(crate) fn limits_mut(&mut self) -> &mut dyn ResourceLimiter {
        &mut self.limits
    }

    pub(crate) fn key_value_provider(&self) -> &dyn KeyValueProvider {
        self.key_value_provider.as_ref()
    }

    pub(crate) fn object_store_provider(&self) -> &dyn ObjectStoreProvider {
        self.object_store_provider.as_ref()
    }

    pub(crate) fn allocate_object_handle(&mut self) -> Option<u64> {
        let handle = self.next_object_handle;
        self.next_object_handle = self.next_object_handle.checked_add(1)?;
        Some(handle)
    }

    pub(crate) fn insert_object_read(&mut self, handle: u64, read: ObjectRead) {
        self.object_reads.insert(handle, read);
    }

    pub(crate) fn object_read_mut(&mut self, handle: u64) -> Option<&mut ObjectRead> {
        self.object_reads.get_mut(&handle)
    }

    pub(crate) fn remove_object_read(&mut self, handle: u64) -> Option<ObjectRead> {
        self.object_reads.remove(&handle)
    }

    pub(crate) fn insert_object_write(&mut self, handle: u64, write: PendingObjectWrite) {
        self.object_writes.insert(handle, write);
    }

    pub(crate) fn object_write(&self, handle: u64) -> Option<&PendingObjectWrite> {
        self.object_writes.get(&handle)
    }

    pub(crate) fn can_open_object_session(&self) -> bool {
        self.object_reads
            .len()
            .checked_add(self.object_writes.len())
            .is_some_and(|open| open < self.max_object_sessions)
    }

    pub(crate) const fn max_object_buffer_bytes(&self) -> usize {
        self.max_object_buffer_bytes
    }

    pub(crate) fn reserve_object_buffer_bytes(&mut self, bytes: usize) -> bool {
        let Some(next) = self.object_buffered_write_bytes.checked_add(bytes) else {
            return false;
        };

        if next > self.max_object_buffer_bytes {
            return false;
        }

        self.object_buffered_write_bytes = next;
        true
    }

    pub(crate) fn release_object_buffer_bytes(&mut self, bytes: usize) {
        debug_assert!(bytes <= self.object_buffered_write_bytes);
        self.object_buffered_write_bytes = self.object_buffered_write_bytes.saturating_sub(bytes);
    }

    pub(crate) fn object_write_mut(&mut self, handle: u64) -> Option<&mut PendingObjectWrite> {
        self.object_writes.get_mut(&handle)
    }

    pub(crate) fn remove_object_write(&mut self, handle: u64) -> Option<PendingObjectWrite> {
        self.object_writes.remove(&handle)
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
