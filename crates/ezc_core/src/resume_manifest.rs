use serde::Serialize;

use crate::resume_plan::ResumePlan;

pub const RESUME_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResumeManifest {
    pub schema_version: u32,
    pub components: Vec<crate::resume_plan::ResumeComponentPlan>,
}

#[must_use]
pub fn build_resume_manifest(plan: &ResumePlan) -> ResumeManifest {
    ResumeManifest {
        schema_version: RESUME_MANIFEST_SCHEMA_VERSION,
        components: plan.components.clone(),
    }
}

#[must_use]
///
/// # Panics
///
/// Panics when the compiler-owned resume manifest cannot serialize.
pub fn resume_manifest_json(manifest: &ResumeManifest) -> String {
    serde_json::to_string_pretty(manifest).expect("resume manifest should serialize")
}
