use std::{collections::BTreeMap, sync::Arc};

#[cfg(test)]
use presolve_provider_sdk::{UnavailableSqlDatabaseProvider, UnavailableWorkQueueProvider};
use presolve_provider_sdk::{
    KeyValueProvider, ObjectRead, ObjectStoreProvider, SqlDatabaseProvider, SqlQueryResult,
    SqlTransaction, WorkQueueDelivery, WorkQueueProvider,
};
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
    sql_database_provider: Arc<dyn SqlDatabaseProvider>,
    work_queue_provider: Arc<dyn WorkQueueProvider>,
    object_reads: BTreeMap<u64, ObjectRead>,
    object_writes: BTreeMap<u64, PendingObjectWrite>,
    max_object_buffer_bytes: usize,
    object_buffered_write_bytes: usize,
    max_object_sessions: usize,
    next_object_handle: u64,
    sql_transactions: BTreeMap<u32, Box<dyn SqlTransaction>>,
    sql_query_results: BTreeMap<u32, SqlQueryResult>,
    max_sql_sessions: usize,
    next_sql_handle: u32,
    queue_deliveries: BTreeMap<u32, Box<dyn WorkQueueDelivery>>,
    max_queue_deliveries: usize,
    next_queue_handle: u32,
}

impl RuntimeState {
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        runtime_limits: RuntimeLimits,
        key_value_provider: Arc<dyn KeyValueProvider>,
        object_store_provider: Arc<dyn ObjectStoreProvider>,
    ) -> Self {
        Self::new_with_capabilities(
            runtime_limits,
            key_value_provider,
            object_store_provider,
            Arc::new(UnavailableSqlDatabaseProvider),
            Arc::new(UnavailableWorkQueueProvider),
        )
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn new_with_sql(
        runtime_limits: RuntimeLimits,
        key_value_provider: Arc<dyn KeyValueProvider>,
        object_store_provider: Arc<dyn ObjectStoreProvider>,
        sql_database_provider: Arc<dyn SqlDatabaseProvider>,
    ) -> Self {
        Self::new_with_capabilities(
            runtime_limits,
            key_value_provider,
            object_store_provider,
            sql_database_provider,
            Arc::new(UnavailableWorkQueueProvider),
        )
    }

    #[must_use]
    pub(crate) fn new_with_capabilities(
        runtime_limits: RuntimeLimits,
        key_value_provider: Arc<dyn KeyValueProvider>,
        object_store_provider: Arc<dyn ObjectStoreProvider>,
        sql_database_provider: Arc<dyn SqlDatabaseProvider>,
        work_queue_provider: Arc<dyn WorkQueueProvider>,
    ) -> Self {
        let mut builder = WasiCtx::builder();

        Self {
            table: ResourceTable::new(),
            wasi: builder.build(),
            limits: runtime_limits.store_limits(),
            key_value_provider,
            object_store_provider,
            sql_database_provider,
            work_queue_provider,
            object_reads: BTreeMap::new(),
            object_writes: BTreeMap::new(),
            max_object_buffer_bytes: runtime_limits.max_object_buffer_bytes(),
            object_buffered_write_bytes: 0,
            max_object_sessions: runtime_limits.max_object_sessions(),
            next_object_handle: 1,
            sql_transactions: BTreeMap::new(),
            sql_query_results: BTreeMap::new(),
            max_sql_sessions: runtime_limits.max_sql_sessions(),
            next_sql_handle: 1,
            queue_deliveries: BTreeMap::new(),
            max_queue_deliveries: runtime_limits.max_queue_deliveries(),
            next_queue_handle: 1,
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

    pub(crate) fn sql_database_provider(&self) -> &dyn SqlDatabaseProvider {
        self.sql_database_provider.as_ref()
    }

    pub(crate) fn work_queue_provider(&self) -> &dyn WorkQueueProvider {
        self.work_queue_provider.as_ref()
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

    pub(crate) fn can_open_sql_session(&self) -> bool {
        self.sql_transactions
            .len()
            .checked_add(self.sql_query_results.len())
            .is_some_and(|open| open < self.max_sql_sessions)
    }

    pub(crate) fn allocate_sql_handle(&mut self) -> Option<u32> {
        let handle = self.next_sql_handle;
        self.next_sql_handle = self.next_sql_handle.checked_add(1)?;
        Some(handle)
    }

    pub(crate) fn insert_sql_transaction(
        &mut self,
        handle: u32,
        transaction: Box<dyn SqlTransaction>,
    ) {
        self.sql_transactions.insert(handle, transaction);
    }

    pub(crate) fn sql_transaction_mut(
        &mut self,
        handle: u32,
    ) -> Option<&mut Box<dyn SqlTransaction>> {
        self.sql_transactions.get_mut(&handle)
    }

    pub(crate) fn remove_sql_transaction(
        &mut self,
        handle: u32,
    ) -> Option<Box<dyn SqlTransaction>> {
        self.sql_transactions.remove(&handle)
    }

    pub(crate) fn insert_sql_query_result(&mut self, handle: u32, result: SqlQueryResult) {
        self.sql_query_results.insert(handle, result);
    }

    pub(crate) fn sql_query_result_mut(&mut self, handle: u32) -> Option<&mut SqlQueryResult> {
        self.sql_query_results.get_mut(&handle)
    }

    pub(crate) fn remove_sql_query_result(&mut self, handle: u32) -> Option<SqlQueryResult> {
        self.sql_query_results.remove(&handle)
    }

    pub(crate) fn can_open_queue_delivery(&self) -> bool {
        self.queue_deliveries.len() < self.max_queue_deliveries
    }

    pub(crate) fn allocate_queue_handle(&mut self) -> Option<u32> {
        let handle = self.next_queue_handle;
        self.next_queue_handle = self.next_queue_handle.checked_add(1)?;
        Some(handle)
    }

    pub(crate) fn insert_queue_delivery(
        &mut self,
        handle: u32,
        delivery: Box<dyn WorkQueueDelivery>,
    ) {
        self.queue_deliveries.insert(handle, delivery);
    }

    pub(crate) fn queue_delivery(&self, handle: u32) -> Option<&dyn WorkQueueDelivery> {
        self.queue_deliveries.get(&handle).map(Box::as_ref)
    }

    pub(crate) fn remove_queue_delivery(
        &mut self,
        handle: u32,
    ) -> Option<Box<dyn WorkQueueDelivery>> {
        self.queue_deliveries.remove(&handle)
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
