use crate::application_semantic_model::ApplicationSemanticModel;
use crate::component_graph::render_event_handlers;
use crate::semantic_id::SemanticId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazyActionChunk {
    pub event: SemanticId,
    pub actions: Vec<SemanticId>,
}

#[must_use]
pub fn plan_lazy_action_chunks(model: &ApplicationSemanticModel) -> Vec<LazyActionChunk> {
    model
        .components
        .iter()
        .flat_map(|component| {
            component
                .render
                .as_ref()
                .into_iter()
                .flat_map(move |render| {
                    render_event_handlers(render).into_iter().map(move |event| {
                        let method = event
                            .handler
                            .strip_prefix("this.")
                            .unwrap_or(&event.handler);
                        LazyActionChunk {
                            event: event.id.clone(),
                            actions: component
                                .actions
                                .iter()
                                .filter(|action| action.method == method)
                                .map(|action| action.id.clone())
                                .collect(),
                        }
                    })
                })
        })
        .collect()
}
