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

/// Resource limits applied to one `EdgeZero` application invocation.
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
}

impl RuntimeLimits {
    /// Creates the default `EdgeZero` runtime limits.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fuel: DEFAULT_FUEL,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_instances: DEFAULT_MAX_INSTANCES,
            max_memories: DEFAULT_MAX_MEMORIES,
            max_tables: DEFAULT_MAX_TABLES,
            max_table_elements: DEFAULT_MAX_TABLE_ELEMENTS,
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
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self::new()
    }
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
    }

    #[test]
    fn limits_can_be_overridden() {
        let limits = RuntimeLimits::new()
            .with_fuel(123)
            .with_max_memory_bytes(8 * MIB)
            .with_max_instances(10)
            .with_max_memories(4)
            .with_max_tables(5)
            .with_max_table_elements(1_000);

        assert_eq!(limits.fuel(), 123);
        assert_eq!(limits.max_memory_bytes(), 8 * MIB);
        assert_eq!(limits.max_instances(), 10);
        assert_eq!(limits.max_memories(), 4);
        assert_eq!(limits.max_tables(), 5);
        assert_eq!(limits.max_table_elements(), 1_000);
    }
}
