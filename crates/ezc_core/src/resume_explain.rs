use crate::application_semantic_model::ApplicationSemanticModel;
use crate::lazy_action_chunks::plan_lazy_action_chunks;
use crate::resume_boot::build_resume_boot_plan;

#[must_use]
pub fn explain_resume(model: &ApplicationSemanticModel) -> String {
    let boot = build_resume_boot_plan(model);
    let chunks = plan_lazy_action_chunks(model);
    format!(
        "components={}\ninstances={}\nchunks={}\nzero_replay={}\ndiagnostics={}\n",
        boot.manifest.components.len(),
        boot.instances.len(),
        chunks.len(),
        boot.zero_replay,
        boot.diagnostics.len()
    )
}
