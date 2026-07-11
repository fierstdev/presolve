use crate::application_semantic_model::ApplicationSemanticModel;
use crate::component_graph::render_event_handlers;
use crate::semantic_id::SemanticId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePlan {
    pub components: Vec<ResumeComponentPlan>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeComponentPlan {
    pub component: SemanticId,
    pub state: Vec<SemanticId>,
    pub events: Vec<SemanticId>,
}

#[must_use]
pub fn build_resume_plan(model: &ApplicationSemanticModel) -> ResumePlan {
    ResumePlan {
        components: model
            .components
            .iter()
            .map(|component| ResumeComponentPlan {
                component: component.id.clone(),
                state: component
                    .state_fields
                    .iter()
                    .map(|field| field.id.clone())
                    .collect(),
                events: component.render.as_ref().map_or_else(Vec::new, |render| {
                    render_event_handlers(render)
                        .into_iter()
                        .map(|handler| handler.id.clone())
                        .collect()
                }),
            })
            .collect(),
    }
}
