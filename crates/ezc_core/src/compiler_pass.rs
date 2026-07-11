use std::collections::BTreeMap;

use crate::application_semantic_model::ApplicationSemanticModel;
use crate::component_graph::{SerializableValue, StateOperation};
use crate::semantic_id::SemanticId;
use crate::semantic_provenance::SourceProvenance;
use crate::validate_application_semantic_model;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct DeadSemanticAnalysisPass;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadSemanticAnalysis {
    pub unreferenced_methods: Vec<SemanticId>,
    pub unreferenced_actions: Vec<SemanticId>,
}

impl AnalysisPass for DeadSemanticAnalysisPass {
    type Output = DeadSemanticAnalysis;
    fn analyze(&self, model: &ApplicationSemanticModel) -> DeadSemanticAnalysis {
        let live = model
            .references
            .iter()
            .map(|reference| reference.target.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let mut methods = Vec::new();
        let mut actions = Vec::new();
        for component in &model.components {
            for method in &component.methods {
                if method.name != "render" && !live.contains(&method.id) {
                    methods.push(method.id.clone());
                    actions.extend(
                        component
                            .actions
                            .iter()
                            .filter(|action| action.owner.entity_id() == Some(&method.id))
                            .map(|action| action.id.clone()),
                    );
                }
            }
        }
        DeadSemanticAnalysis {
            unreferenced_methods: methods,
            unreferenced_actions: actions,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OptimizationPlanningPass;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationPlan {
    pub recommendations: Vec<OptimizationRecommendation>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationRecommendation {
    pub id: SemanticId,
    pub provenance: SourceProvenance,
}

impl AnalysisPass for OptimizationPlanningPass {
    type Output = OptimizationPlan;
    fn analyze(&self, model: &ApplicationSemanticModel) -> OptimizationPlan {
        let dead = DeadSemanticAnalysisPass.analyze(model);
        let recommendations = dead
            .unreferenced_methods
            .into_iter()
            .chain(dead.unreferenced_actions)
            .filter_map(|id| {
                model
                    .provenance(&id)
                    .cloned()
                    .map(|provenance| OptimizationRecommendation { id, provenance })
            })
            .collect();
        OptimizationPlan { recommendations }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ExplainabilityPass;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainabilityReport {
    pub lines: Vec<String>,
}
impl AnalysisPass for ExplainabilityPass {
    type Output = ExplainabilityReport;
    fn analyze(&self, model: &ApplicationSemanticModel) -> ExplainabilityReport {
        let dependencies = DependencyAnalysisPass.analyze(model);
        let constants = ConstantEvaluationPass.analyze(model);
        let optimizations = OptimizationPlanningPass.analyze(model);
        let validation = validate_application_semantic_model(model);
        ExplainabilityReport {
            lines: vec![
                format!("components={}", model.components.len()),
                format!("dependencies={}", dependencies.dependencies.len()),
                format!("constants={}", constants.values.len()),
                format!(
                    "optimization_recommendations={}",
                    optimizations.recommendations.len()
                ),
                format!("validation_diagnostics={}", validation.len()),
            ],
        }
    }
}
