//! V2 inspection of canonical route-loader plans.
use crate::RouteLoaderPlanV2;
use serde::Serialize;
pub const ROUTE_LOADER_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderProjectionV2 {
    pub schema_version: u32,
    pub route_count: usize,
    pub loader_count: usize,
}
#[must_use]
pub fn build_route_loader_projection_v2(plan: &RouteLoaderPlanV2) -> RouteLoaderProjectionV2 {
    RouteLoaderProjectionV2 {
        schema_version: ROUTE_LOADER_PROJECTION_SCHEMA_VERSION,
        route_count: plan.routes.len(),
        loader_count: plan.routes.iter().map(|r| r.loaders.len()).sum(),
    }
}
