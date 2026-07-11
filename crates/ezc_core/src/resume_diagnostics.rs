use crate::resume_instance::SerializableInstance;
use crate::resume_plan::ResumePlan;
use crate::semantic_id::SemanticId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeDiagnostic {
    pub code: String,
    pub component: SemanticId,
    pub state: Option<SemanticId>,
}

#[must_use]
pub fn validate_resume_instances(
    plan: &ResumePlan,
    instances: &[SerializableInstance],
) -> Vec<ResumeDiagnostic> {
    let mut diagnostics = Vec::new();
    for component in &plan.components {
        let Some(instance) = instances
            .iter()
            .find(|instance| instance.component == component.component)
        else {
            diagnostics.push(ResumeDiagnostic {
                code: "EZRSM1001".to_string(),
                component: component.component.clone(),
                state: None,
            });
            continue;
        };
        for state in &component.state {
            if !instance.state.contains_key(state) {
                diagnostics.push(ResumeDiagnostic {
                    code: "EZRSM1002".to_string(),
                    component: component.component.clone(),
                    state: Some(state.clone()),
                });
            }
        }
    }
    diagnostics
}
