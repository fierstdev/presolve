//! V2 inspection of canonical Resource declarations.
use crate::ResourceDeclaration;
use serde::Serialize;
pub const RESOURCE_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceProjectionV1 {
    pub schema_version: u32,
    pub resources: usize,
}
#[must_use]
pub fn build_resource_projection_v1(resources: &[ResourceDeclaration]) -> ResourceProjectionV1 {
    ResourceProjectionV1 {
        schema_version: RESOURCE_PROJECTION_SCHEMA_VERSION,
        resources: resources.len(),
    }
}
