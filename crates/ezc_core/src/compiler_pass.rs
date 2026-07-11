use std::collections::BTreeMap;

use crate::application_semantic_model::ApplicationSemanticModel;
use crate::component_graph::{SerializableValue, StateOperation};
use crate::semantic_id::SemanticId;

pub trait AnalysisPass {
    type Output;

    fn analyze(&self, model: &ApplicationSemanticModel) -> Self::Output;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DependencyAnalysisPass;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyAnalysis {
    pub dependencies: BTreeMap<SemanticId, Vec<SemanticId>>,
    pub dependents: BTreeMap<SemanticId, Vec<SemanticId>>,
}

impl AnalysisPass for DependencyAnalysisPass {
    type Output = DependencyAnalysis;

    fn analyze(&self, model: &ApplicationSemanticModel) -> DependencyAnalysis {
        let mut dependencies = BTreeMap::<SemanticId, Vec<SemanticId>>::new();
        let mut dependents = BTreeMap::<SemanticId, Vec<SemanticId>>::new();

        for reference in &model.references {
            dependencies
                .entry(reference.source.clone())
                .or_default()
                .push(reference.target.clone());
            dependents
                .entry(reference.target.clone())
                .or_default()
                .push(reference.source.clone());
        }

        DependencyAnalysis {
            dependencies,
            dependents,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConstantEvaluationPass;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantEvaluation {
    pub values: BTreeMap<SemanticId, SerializableValue>,
}

impl AnalysisPass for ConstantEvaluationPass {
    type Output = ConstantEvaluation;
    fn analyze(&self, model: &ApplicationSemanticModel) -> ConstantEvaluation {
        let mut values = BTreeMap::new();
        for component in &model.components {
            for field in &component.state_fields {
                if let Some(value) = &field.initial_value {
                    values.insert(field.id.clone(), value.clone());
                }
            }
            for action in &component.actions {
                let value = match &action.operation {
                    StateOperation::AddAssign(value)
                    | StateOperation::SubtractAssign(value)
                    | StateOperation::Assign(value) => Some(value),
                    _ => None,
                };
                if let Some(value) = value {
                    values.insert(action.id.clone(), value.clone());
                }
            }
        }
        ConstantEvaluation { values }
    }
}
