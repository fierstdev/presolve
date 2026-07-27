//! V2 inspection of capability admission.
use crate::SemanticCapabilityRegistry;
use serde::Serialize;
pub const CAPABILITY_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityProjectionV1 {
    pub schema_version: u32,
    pub admitted: usize,
    pub deferred: usize,
}
#[must_use]
pub fn build_capability_projection_v1(
    registry: &SemanticCapabilityRegistry,
) -> CapabilityProjectionV1 {
    CapabilityProjectionV1 {
        schema_version: CAPABILITY_PROJECTION_SCHEMA_VERSION,
        admitted: registry
            .capabilities
            .iter()
            .filter(|c| matches!(c.status, crate::SemanticCapabilityStatus::Admitted))
            .count(),
        deferred: registry
            .capabilities
            .iter()
            .filter(|c| matches!(c.status, crate::SemanticCapabilityStatus::Deferred))
            .count(),
    }
}
