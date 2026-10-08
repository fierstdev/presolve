use std::time::Duration;

use wasmtime::{StoreLimits, StoreLimitsBuilder};

/// Number of bytes in one mebibyte.
pub const MIB: usize = 1024 * 1024;

/// Default deterministic execution budget.
pub const DEFAULT_FUEL: u64 = 10_000_000;

/// Default maximum size of each WebAssembly linear memory.
pub const DEFAULT_MAX_MEMORY_BYTES: usize = 64 * MIB;

/// Default maximum number of core instances in one store.
pub const DEFAULT_MAX_INSTANCES: usize = 1_024;

/// Default maximum number of linear memories in one store.
pub const DEFAULT_MAX_MEMORIES: usize = 256;

/// Default maximum number of tables in one store.
pub const DEFAULT_MAX_TABLES: usize = 256;

/// Default maximum number of elements in each table.
pub const DEFAULT_MAX_TABLE_ELEMENTS: usize = 100_000;

/// Default maximum wall-clock duration for active WebAssembly execution.
pub const DEFAULT_EXECUTION_TIMEOUT: Duration = Duration::from_secs(2);

/// Frequency at which the shared Wasmtime engine epoch advances.
///
/// Epoch deadlines are coarse-grained rather than precise wall-clock timers.
/// An application may therefore execute somewhat beyond its requested timeout
/// before reaching an epoch check.
pub const EPOCH_TICK_INTERVAL: Duration = Duration::from_millis(10);

/// Resource limits applied to one `Presolve` application invocation.
///
/// These are host-enforced ceilings. Application contracts may request fewer
/// resources, but they cannot override host policy to obtain more resources
/// than the runtime permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeLimits {
    fuel: u64,
    max_memory_bytes: usize,
    max_instances: usize,
    max_memories: usize,
    max_tables: usize,
    max_table_elements: usize,
    execution_timeout: Duration,
}

impl RuntimeLimits {
    /// Creates the default `Presolve` runtime limits.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fuel: DEFAULT_FUEL,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_instances: DEFAULT_MAX_INSTANCES,
            max_memories: DEFAULT_MAX_MEMORIES,
            max_tables: DEFAULT_MAX_TABLES,
            max_table_elements: DEFAULT_MAX_TABLE_ELEMENTS,
            execution_timeout: DEFAULT_EXECUTION_TIMEOUT,
        }
    }

    /// Sets the deterministic execution fuel budget.
    #[must_use]
    pub const fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Sets the maximum size of each WebAssembly linear memory.
    #[must_use]
    pub const fn with_max_memory_bytes(mut self, max_memory_bytes: usize) -> Self {
        self.max_memory_bytes = max_memory_bytes;
        self
    }

    /// Sets the maximum number of core instances.
    #[must_use]
    pub const fn with_max_instances(mut self, max_instances: usize) -> Self {
        self.max_instances = max_instances;
        self
    }

    /// Sets the maximum number of linear memories.
    #[must_use]
    pub const fn with_max_memories(mut self, max_memories: usize) -> Self {
        self.max_memories = max_memories;
        self
    }

    /// Sets the maximum number of tables.
    #[must_use]
    pub const fn with_max_tables(mut self, max_tables: usize) -> Self {
        self.max_tables = max_tables;
        self
    }

    /// Sets the maximum number of elements in each table.
    #[must_use]
    pub const fn with_max_table_elements(mut self, max_table_elements: usize) -> Self {
        self.max_table_elements = max_table_elements;
        self
    }

    /// Sets the maximum wall-clock duration for active WebAssembly execution.
    ///
    /// This is a coarse deadline implemented through Wasmtime epoch
    /// interruption. It does not interrupt a synchronous host function while
    /// WebAssembly is blocked inside that function.
    #[must_use]
    pub const fn with_execution_timeout(mut self, execution_timeout: Duration) -> Self {
        self.execution_timeout = execution_timeout;
        self
    }

    /// Returns the deterministic execution fuel budget.
    #[must_use]
    pub const fn fuel(self) -> u64 {
        self.fuel
    }

    /// Returns the maximum size of each linear memory.
    #[must_use]
    pub const fn max_memory_bytes(self) -> usize {
        self.max_memory_bytes
    }

    /// Returns the maximum number of core instances.
    #[must_use]
    pub const fn max_instances(self) -> usize {
        self.max_instances
    }

    /// Returns the maximum number of linear memories.
    #[must_use]
    pub const fn max_memories(self) -> usize {
        self.max_memories
    }

    /// Returns the maximum number of tables.
    #[must_use]
    pub const fn max_tables(self) -> usize {
        self.max_tables
    }

    /// Returns the maximum number of elements in each table.
    #[must_use]
    pub const fn max_table_elements(self) -> usize {
        self.max_table_elements
    }

    /// Returns the maximum active WebAssembly execution duration.
    #[must_use]
    pub const fn execution_timeout(self) -> Duration {
        self.execution_timeout
    }

    pub(crate) fn store_limits(self) -> StoreLimits {
        StoreLimitsBuilder::new()
            .memory_size(self.max_memory_bytes)
            .instances(self.max_instances)
            .memories(self.max_memories)
            .tables(self.max_tables)
            .table_elements(self.max_table_elements)
            .trap_on_grow_failure(true)
            .build()
    }

    pub(crate) fn epoch_deadline_ticks(self) -> u64 {
        duration_to_ticks(self.execution_timeout, EPOCH_TICK_INTERVAL)
    }
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self::new()
    }
}

fn duration_to_ticks(duration: Duration, tick_interval: Duration) -> u64 {
    if duration.is_zero() {
        return 0;
    }

    let ticks = duration.as_nanos().div_ceil(tick_interval.as_nanos());

    u64::try_from(ticks).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_bounded() {
        let limits = RuntimeLimits::default();

        assert_eq!(limits.fuel(), DEFAULT_FUEL);
        assert_eq!(limits.max_memory_bytes(), DEFAULT_MAX_MEMORY_BYTES);
        assert_eq!(limits.max_instances(), DEFAULT_MAX_INSTANCES);
        assert_eq!(limits.max_memories(), DEFAULT_MAX_MEMORIES);
        assert_eq!(limits.max_tables(), DEFAULT_MAX_TABLES);
        assert_eq!(limits.max_table_elements(), DEFAULT_MAX_TABLE_ELEMENTS);
        assert_eq!(limits.execution_timeout(), DEFAULT_EXECUTION_TIMEOUT);
    }

    #[test]
    fn limits_can_be_overridden() {
        let limits = RuntimeLimits::new()
            .with_fuel(123)
            .with_max_memory_bytes(8 * MIB)
            .with_max_instances(10)
            .with_max_memories(4)
            .with_max_tables(5)
            .with_max_table_elements(1_000)
            .with_execution_timeout(Duration::from_millis(250));

        assert_eq!(limits.fuel(), 123);
        assert_eq!(limits.max_memory_bytes(), 8 * MIB);
        assert_eq!(limits.max_instances(), 10);
        assert_eq!(limits.max_memories(), 4);
        assert_eq!(limits.max_tables(), 5);
        assert_eq!(limits.max_table_elements(), 1_000);
        assert_eq!(limits.execution_timeout(), Duration::from_millis(250));
    }

    #[test]
    fn timeout_is_rounded_up_to_epoch_ticks() {
        let limits = RuntimeLimits::new().with_execution_timeout(Duration::from_millis(25));

        assert_eq!(limits.epoch_deadline_ticks(), 3);
    }

    #[test]
    fn zero_timeout_expires_immediately() {
        let limits = RuntimeLimits::new().with_execution_timeout(Duration::ZERO);

        assert_eq!(limits.epoch_deadline_ticks(), 0);
    }
}
