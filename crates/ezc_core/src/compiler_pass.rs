use std::collections::BTreeMap;

use crate::application_semantic_model::{ApplicationSemanticModel, SemanticEntityKind};
use crate::component_graph::{
    ComponentDiagnostic, ComponentGraph, ConstantExpressionKind, DeclaredStateTypeKind,
    SerializableValue, StateOperation,
};
use crate::semantic_id::SemanticId;
use crate::semantic_provenance::SourceProvenance;
use crate::validate_application_semantic_model;
use crate::{build_application_semantic_model_from_component_graph, build_template_graph};

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

/// Immutable constant-folding pass over compiler-lowered state expressions.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConstantFoldingPass;

impl ImmutableAsmPass for ConstantFoldingPass {
    type Output = ApplicationSemanticModel;

    fn transform(&self, model: &ApplicationSemanticModel) -> ApplicationSemanticModel {
        let mut folded = model.clone();

        for component in &mut folded.components {
            for field in &mut component.state_fields {
                let Some(expression) = field.initial_expression.as_ref() else {
                    continue;
                };

                match expression.evaluate() {
                    Ok(value) => {
                        field.initial_value = Some(value);
                        if let Some(diagnostic) =
                            folded_type_mismatch_diagnostic(field, &component.class_name)
                        {
                            push_diagnostic_once(&mut folded.diagnostics, diagnostic);
                        }
                    }
                    Err(error) => push_diagnostic_once(
                        &mut folded.diagnostics,
                        ComponentDiagnostic {
                            provenance: folded.provenance.get(&field.id).map(|provenance| {
                                SourceProvenance::new(&provenance.path, expression.span)
                            }),
                            code: constant_expression_diagnostic_code(&expression.kind).to_string(),
                            message: format!(
                                "state field `{}` has an invalid {} initializer: {error}",
                                field.name,
                                constant_expression_kind_name(&expression.kind)
                            ),
                        },
                    ),
                }
            }
        }

        let graph = ComponentGraph {
            components: folded.components.clone(),
            diagnostics: folded.diagnostics.clone(),
            references: folded.references.clone(),
            provenance: folded.provenance.clone(),
        };
        folded.templates = build_template_graph(&graph).templates;
        folded
    }
}

/// Fold a legacy component graph through the canonical ASM pass for backend consumers.
#[must_use]
pub fn fold_component_graph(component_graph: &ComponentGraph) -> ComponentGraph {
    let model = build_application_semantic_model_from_component_graph(component_graph);
    let folded = ConstantFoldingPass.transform(&model);

    ComponentGraph {
        components: folded.components,
        diagnostics: folded.diagnostics,
        references: component_graph.references.clone(),
        provenance: folded.provenance,
    }
}

impl ImmutableAsmPass for ConstantEvaluationPass {
    type Output = ConstantEvaluation;
    fn transform(&self, model: &ApplicationSemanticModel) -> ConstantEvaluation {
        let folded = ConstantFoldingPass.transform(model);
        let mut values = BTreeMap::new();
        for component in &folded.components {
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

fn constant_expression_diagnostic_code(kind: &ConstantExpressionKind) -> &'static str {
    match kind {
        ConstantExpressionKind::Arithmetic(_) => "EZC1022",
        ConstantExpressionKind::Comparison { .. } => "EZC1023",
        ConstantExpressionKind::Boolean(_) | ConstantExpressionKind::Logical { .. } => "EZC1024",
        ConstantExpressionKind::Literal(_) | ConstantExpressionKind::NullishCoalescing { .. } => {
            "EZC1025"
        }
        ConstantExpressionKind::Unary { .. } => "EZC1026",
    }
}

fn constant_expression_kind_name(kind: &ConstantExpressionKind) -> &'static str {
    match kind {
        ConstantExpressionKind::Arithmetic(_) => "arithmetic",
        ConstantExpressionKind::Comparison { .. } => "comparison",
        ConstantExpressionKind::Boolean(_) | ConstantExpressionKind::Logical { .. } => "logical",
        ConstantExpressionKind::Literal(_) | ConstantExpressionKind::NullishCoalescing { .. } => {
            "nullish-coalescing"
        }
        ConstantExpressionKind::Unary { .. } => "unary",
    }
}

fn folded_type_mismatch_diagnostic(
    field: &crate::component_graph::StateField,
    class_name: &str,
) -> Option<ComponentDiagnostic> {
    let declared_type = field.declared_type.as_ref()?;
    let declared_kind = declared_type.kind?;
    let value_kind = match field.initial_value.as_ref()? {
        SerializableValue::String(_) => DeclaredStateTypeKind::String,
        SerializableValue::Number(_) => DeclaredStateTypeKind::Number,
        SerializableValue::Boolean(_) => DeclaredStateTypeKind::Boolean,
        SerializableValue::Null => DeclaredStateTypeKind::Null,
        SerializableValue::Array(_) | SerializableValue::Object(_) => return None,
    };
    (declared_kind != value_kind).then(|| ComponentDiagnostic {
        provenance: Some(declared_type.provenance.clone()),
        code: "EZC1016".to_string(),
        message: format!(
            "state field `{}` in class `{class_name}` declares `{}` but initializes with `{}`",
            field.name,
            declared_state_type_name(declared_kind),
            declared_state_type_name(value_kind)
        ),
    })
}

fn push_diagnostic_once(
    diagnostics: &mut Vec<ComponentDiagnostic>,
    diagnostic: ComponentDiagnostic,
) {
    if !diagnostics.contains(&diagnostic) {
        diagnostics.push(diagnostic);
    }
}

fn declared_state_type_name(kind: DeclaredStateTypeKind) -> &'static str {
    match kind {
        DeclaredStateTypeKind::String => "string",
        DeclaredStateTypeKind::Number => "number",
        DeclaredStateTypeKind::Boolean => "boolean",
        DeclaredStateTypeKind::Null => "null",
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
        AnalysisPass, ConstantEvaluationPass, ConstantFoldingPass, DeadSemanticAnalysisPass,
        DependencyAnalysisPass, ExplainabilityPass, ImmutableAsmPass, OptimizationPlanningPass,
    };
    use crate::{build_application_semantic_model, SemanticOwner, SerializableValue};

    #[test]
    fn folds_lowered_constant_expressions_immutably_before_backend_consumption() {
        let parsed = ezc_parser::parse_file(
            "src/FoldedState.tsx",
            r#"
@component("x-folded-state")
class FoldedState extends Component {
  total: number = state((1 + 2) * 3);

  render() {
    return <output>{this.total}</output>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let original = asm.clone();
        let field_id = asm.components[0].state_fields[0].id.clone();

        assert_eq!(asm.components[0].state_fields[0].initial_value, None);
        assert!(asm.diagnostics.is_empty());

        let folded = ConstantFoldingPass.transform(&asm);
        assert_eq!(
            folded.components[0].state_fields[0].initial_value,
            Some(SerializableValue::Number("9".to_string()))
        );
        assert_eq!(
            ConstantEvaluationPass.transform(&asm).values[&field_id],
            SerializableValue::Number("9".to_string())
        );
        assert_eq!(
            crate::generate_static_html(&crate::TemplateGraph {
                templates: folded.templates.clone(),
            }),
            "<output data-ez-node=\"n0\" data-ez-bindings=\"this.total\"><!-- ez-binding:n1:this.total -->9</output>\n"
        );
        assert_eq!(ConstantFoldingPass.transform(&folded), folded);
        assert_eq!(asm, original);
    }

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
            ConstantFoldingPass.transform(&asm),
            ConstantFoldingPass.analyze(&asm)
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
