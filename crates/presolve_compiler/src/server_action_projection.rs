//! V2 inspection of canonical server-action plans.
use crate::RouteServerActionPlanV1;
use serde::Serialize;
pub const SERVER_ACTION_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerActionProjectionV1 {
    pub schema_version: u32,
    pub route_count: usize,
    pub action_count: usize,
}
#[must_use]
pub fn build_server_action_projection_v1(
    plan: &RouteServerActionPlanV1,
) -> ServerActionProjectionV1 {
    ServerActionProjectionV1 {
        schema_version: SERVER_ACTION_PROJECTION_SCHEMA_VERSION,
        route_count: plan.routes.len(),
        action_count: plan.routes.iter().map(|r| r.actions.len()).sum(),
    }
}
