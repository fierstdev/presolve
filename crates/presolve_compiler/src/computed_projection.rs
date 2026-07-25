//! V2 inspection projection of canonical computed runtime records.
use crate::RuntimeComputedRegistry;
use serde::Serialize;
pub const COMPUTED_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComputedProjectionRecordV1 {
    pub computed: String,
    pub cache_slot: String,
    pub dirty_flag: String,
    pub dependencies: Vec<String>,
    pub memoized_per_instance: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComputedProjectionV1 {
    pub schema_version: u32,
    pub records: Vec<ComputedProjectionRecordV1>,
}
#[must_use]
pub fn build_computed_projection_v1(registry: &RuntimeComputedRegistry) -> ComputedProjectionV1 {
    let mut records = registry
        .records
        .values()
        .map(|record| ComputedProjectionRecordV1 {
            computed: record.computed.to_string(),
            cache_slot: record.cache_slot.as_str().to_owned(),
            dirty_flag: record.dirty_flag.id.as_str().to_owned(),
            dependencies: record
                .dependencies
                .iter()
                .map(ToString::to_string)
                .collect(),
            memoized_per_instance: true,
        })
        .collect::<Vec<_>>();
    records.sort_by(|a, b| a.computed.cmp(&b.computed));
    ComputedProjectionV1 {
        schema_version: COMPUTED_PROJECTION_SCHEMA_VERSION,
        records,
    }
}
