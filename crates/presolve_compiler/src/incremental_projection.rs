//! V2 inspection of canonical incremental compilation plans.
use crate::platform::IncrementalCompilationPlanV1;
use serde::Serialize;
pub const INCREMENTAL_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalProjectionV1 {
    pub schema_version: u32,
    pub directly_invalidated: usize,
    pub closure: usize,
    pub reusable: usize,
    pub recompute: usize,
    pub fallback_reasons: usize,
}
#[must_use]
pub fn build_incremental_projection_v1(
    plan: &IncrementalCompilationPlanV1,
) -> IncrementalProjectionV1 {
    IncrementalProjectionV1 {
        schema_version: INCREMENTAL_PROJECTION_SCHEMA_VERSION,
        directly_invalidated: plan.directly_invalidated.len(),
        closure: plan.invalidation_closure.len(),
        reusable: plan.reusable_product_identities.len(),
        recompute: plan.recompute_work_units.len(),
        fallback_reasons: plan.fallback_reasons.len(),
    }
}
