//! V2 inspection of canonical validation dependency plans.
use crate::ValidationDependencyPlans;
use serde::Serialize;
pub const VALIDATION_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationProjectionV1 {
    pub schema_version: u32,
    pub dependency_count: usize,
    pub blocked_count: usize,
}
#[must_use]
pub fn build_validation_projection_v1(plans: &ValidationDependencyPlans) -> ValidationProjectionV1 {
    ValidationProjectionV1 {
        schema_version: VALIDATION_PROJECTION_SCHEMA_VERSION,
        dependency_count: plans.dependencies.len(),
        blocked_count: plans.blocked.len(),
    }
}
