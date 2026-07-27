//! V2 projection of canonical Context resolution and resume records.
use crate::{ContextResolution, ContextResolutionResult, ContextResumePlan};
use serde::Serialize;
pub const CONTEXT_RESUME_PROJECTION_SCHEMA_VERSION: u32 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextConsumerResolutionV1 {
    pub consumer: String,
    pub outcome: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextResumeProjectionRecordV1 {
    pub context: String,
    pub resume_slot: String,
    pub consumer_count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextResumeProjectionV1 {
    pub schema_version: u32,
    pub resolutions: Vec<ContextConsumerResolutionV1>,
    pub resume: Vec<ContextResumeProjectionRecordV1>,
}
#[must_use]
pub fn build_context_resume_projection_v1(
    resolutions: &std::collections::BTreeMap<crate::ConsumerId, ContextResolution>,
    plan: &ContextResumePlan,
) -> ContextResumeProjectionV1 {
    let mut results = resolutions
        .values()
        .map(|r| ContextConsumerResolutionV1 {
            consumer: r.consumer.as_str().into(),
            outcome: match &r.result {
                ContextResolutionResult::Provider { provider, .. } => {
                    format!("provider:{provider}")
                }
                ContextResolutionResult::ContextDefault { context, .. } => {
                    format!("default:{context}")
                }
                ContextResolutionResult::Unresolved => "unresolved".into(),
                ContextResolutionResult::Ambiguous { .. } => "ambiguous".into(),
                ContextResolutionResult::InvalidContextReference => "invalid".into(),
            },
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.consumer.cmp(&b.consumer));
    let resume = plan
        .records
        .iter()
        .map(|r| ContextResumeProjectionRecordV1 {
            context: r.context.as_str().into(),
            resume_slot: r.resume_slot.as_str().into(),
            consumer_count: r.consumers.len(),
        })
        .collect();
    ContextResumeProjectionV1 {
        schema_version: CONTEXT_RESUME_PROJECTION_SCHEMA_VERSION,
        resolutions: results,
        resume,
    }
}
