use crate::application_semantic_model::ApplicationSemanticModel;
use crate::resume_diagnostics::{validate_resume_instances, ResumeDiagnostic};
use crate::resume_instance::{build_serializable_instances, SerializableInstance};
use crate::resume_manifest::{build_resume_manifest, ResumeManifest};
use crate::resume_plan::build_resume_plan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBootPlan {
    pub manifest: ResumeManifest,
    pub instances: Vec<SerializableInstance>,
    pub diagnostics: Vec<ResumeDiagnostic>,
    pub zero_replay: bool,
}

#[must_use]
pub fn build_resume_boot_plan(model: &ApplicationSemanticModel) -> ResumeBootPlan {
    let plan = build_resume_plan(model);
    let instances = build_serializable_instances(model);
    let diagnostics = validate_resume_instances(&plan, &instances);
    ResumeBootPlan {
        manifest: build_resume_manifest(&plan),
        instances,
        zero_replay: diagnostics.is_empty(),
        diagnostics,
    }
}
