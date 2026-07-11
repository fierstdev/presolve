use std::collections::BTreeMap;

use crate::application_semantic_model::{ApplicationSemanticModel, SemanticEntityKind};
use crate::component_graph::{SerializableValue, StateOperation};
use crate::semantic_id::SemanticId;
use crate::semantic_provenance::SourceProvenance;
use crate::validate_application_semantic_model;

/// An immutable transformation from canonical ASM input to a compiler product.
///
/// Implementations must treat the input ASM as read-only and return a newly
/// constructed output. This is the common pass boundary for analysis products
/// today and future transformed ASM products where applicable.
pub trait ImmutableAsmPass {
    type Output;

    fn transform(&self, model: &ApplicationSemanticModel) -> Self::Output;
}

/// Compatibility surface for existing analysis consumers.
pub trait AnalysisPass: ImmutableAsmPass {
    fn analyze(&self, model: &ApplicationSemanticModel) -> Self::Output {
        self.transform(model)
    }
}

impl<T> AnalysisPass for T where T: ImmutableAsmPass {}

#[derive(Debug, Default, Clone, Copy)]
pub struct DependencyAnalysisPass;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyAnalysis {
    pub dependencies: BTreeMap<SemanticId, Vec<SemanticId>>,
    pub dependents: BTreeMap<SemanticId, Vec<SemanticId>>,
}

impl ImmutableAsmPass for DependencyAnalysisPass {
    type Output = DependencyAnalysis;

    fn transform(&self, model: &ApplicationSemanticModel) -> DependencyAnalysis {
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

impl ImmutableAsmPass for ConstantEvaluationPass {
    type Output = ConstantEvaluation;
    fn transform(&self, model: &ApplicationSemanticModel) -> ConstantEvaluation {
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

impl ImmutableAsmPass for DeadSemanticAnalysisPass {
    type Output = DeadSemanticAnalysis;
    fn transform(&self, model: &ApplicationSemanticModel) -> DeadSemanticAnalysis {
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
                        model
                            .children_of(&method.id)
                            .iter()
                            .filter(|id| {
                                model.entity(id).is_some_and(|entity| {
                                    entity.kind() == SemanticEntityKind::Action
                                })
                            })
                            .map(|id| (*id).clone()),
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

impl ImmutableAsmPass for OptimizationPlanningPass {
    type Output = OptimizationPlan;
    fn transform(&self, model: &ApplicationSemanticModel) -> OptimizationPlan {
        let dead = DeadSemanticAnalysisPass.transform(model);
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
impl ImmutableAsmPass for ExplainabilityPass {
    type Output = ExplainabilityReport;
    fn transform(&self, model: &ApplicationSemanticModel) -> ExplainabilityReport {
        let dependencies = DependencyAnalysisPass.transform(model);
        let constants = ConstantEvaluationPass.transform(model);
        let optimizations = OptimizationPlanningPass.transform(model);
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

#[cfg(test)]
mod tests {
    use super::{
        AnalysisPass, ConstantEvaluationPass, DeadSemanticAnalysisPass, DependencyAnalysisPass,
        ExplainabilityPass, ImmutableAsmPass, OptimizationPlanningPass,
    };
    use crate::{build_application_semantic_model, SemanticOwner};

    #[test]
    fn finds_dead_actions_from_canonical_ownership() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  unused() {
    this.count++;
  }

  render() {
    return <div>Counter</div>;
  }
}
"#,
        );
        let mut asm = build_application_semantic_model(&parsed);
        let action_id = asm.components[0].actions[0].id.clone();
        asm.components[0].actions[0].owner = SemanticOwner::Application;

        let analysis = DeadSemanticAnalysisPass.analyze(&asm);
        assert_eq!(analysis.unreferenced_actions, vec![action_id]);
    }

    #[test]
    fn transforms_asm_immutably_with_compatibility_analysis_results() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={() => this.increment()}>{this.count}</button>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let original = asm.clone();

        assert_eq!(
            DependencyAnalysisPass.transform(&asm),
            DependencyAnalysisPass.analyze(&asm)
        );
        assert_eq!(
            ConstantEvaluationPass.transform(&asm),
            ConstantEvaluationPass.analyze(&asm)
        );
        assert_eq!(
            DeadSemanticAnalysisPass.transform(&asm),
            DeadSemanticAnalysisPass.analyze(&asm)
        );
        assert_eq!(
            OptimizationPlanningPass.transform(&asm),
            OptimizationPlanningPass.analyze(&asm)
        );
        assert_eq!(
            ExplainabilityPass.transform(&asm),
            ExplainabilityPass.analyze(&asm)
        );
        assert_eq!(asm, original);
    }
}
