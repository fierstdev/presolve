use crate::application_semantic_model::ApplicationSemanticModel;
use crate::component_graph::SerializableValue;
use crate::semantic_id::SemanticId;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializableInstance {
    pub component: SemanticId,
    pub state: BTreeMap<SemanticId, SerializableValue>,
}

#[must_use]
pub fn build_serializable_instances(model: &ApplicationSemanticModel) -> Vec<SerializableInstance> {
    model
        .components
        .iter()
        .map(|component| SerializableInstance {
            component: component.id.clone(),
            state: component
                .state_fields
                .iter()
                .filter_map(|field| {
                    field
                        .initial_value
                        .as_ref()
                        .map(|value| (field.id.clone(), value.clone()))
                })
                .collect(),
        })
        .collect()
}
