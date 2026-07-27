//! V2 inspection projection of canonical runtime effects.
use crate::RuntimeEffectRegistry;
use serde::Serialize;
pub const EFFECT_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectProjectionRecordV1 {
    pub effect: String,
    pub execution_function: String,
    pub browser_only: bool,
    pub initial_trigger: bool,
    pub action_trigger_count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectProjectionV1 {
    pub schema_version: u32,
    pub records: Vec<EffectProjectionRecordV1>,
}
#[must_use]
pub fn build_effect_projection_v1(registry: &RuntimeEffectRegistry) -> EffectProjectionV1 {
    let mut records = registry
        .records
        .values()
        .map(|r| EffectProjectionRecordV1 {
            effect: r.effect.to_string(),
            execution_function: r.execution_function.to_string(),
            browser_only: matches!(r.execution_boundary, crate::ExecutionBoundary::Client),
            initial_trigger: r.initial_trigger.is_some(),
            action_trigger_count: r.action_batch_triggers.len(),
        })
        .collect::<Vec<_>>();
    records.sort_by(|a, b| a.effect.cmp(&b.effect));
    EffectProjectionV1 {
        schema_version: EFFECT_PROJECTION_SCHEMA_VERSION,
        records,
    }
}
