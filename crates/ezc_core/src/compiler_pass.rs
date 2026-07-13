use std::collections::BTreeMap;

use crate::application_semantic_model::{ApplicationSemanticModel, SemanticEntityKind};
use crate::component_graph::{
    ComponentDiagnostic, ComponentGraph, SerializableValue, StateOperation,
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
                let Some(result) = folded.expression_graph.evaluate(&field.id) else {
                    continue;
                };

                let root = folded
                    .expression_graph
                    .root_for(&field.id)
                    .expect("expression evaluation should have a graph root");
                let expression = folded
                    .expression_graph
                    .nodes
                    .get(root)
                    .expect("expression graph root should be a node");

                match result {
                    Ok(value) => {
                        field.initial_value = Some(value);
                    }
                    Err(error) => push_diagnostic_once(
                        &mut folded.diagnostics,
                        ComponentDiagnostic {
                            provenance: Some(expression.provenance.clone()),
                            code: constant_expression_diagnostic_code_from_node(&expression.kind)
                                .as_str()
                                .to_string(),
                            message: format!(
                                "state field `{}` has an invalid {} initializer: {error}",
                                field.name,
                                constant_expression_kind_name_from_node(&expression.kind)
                            ),
                        },
                    ),
                }
            }
        }

        for component in &folded.components {
            for field in &component.state_fields {
                if let Some(diagnostic) =
                    unknown_declared_type_diagnostic(&folded, field, component)
                {
                    push_diagnostic_once(&mut folded.diagnostics, diagnostic);
                }
                if let Some(diagnostic) =
                    folded_type_mismatch_diagnostic(&folded, field, &component.class_name)
                {
                    push_diagnostic_once(&mut folded.diagnostics, diagnostic);
                }
            }
            for action in &component.actions {
                if let Some(diagnostic) =
                    action_assignment_mismatch_diagnostic(&folded, component, action)
                {
                    push_diagnostic_once(&mut folded.diagnostics, diagnostic);
                }
                for diagnostic in compound_mutation_type_diagnostics(&folded, component, action) {
                    push_diagnostic_once(&mut folded.diagnostics, diagnostic);
                }
            }
        }

        for entity in &folded.template_entities {
            if let Some(diagnostic) = template_binding_type_diagnostic(&folded, entity) {
                push_diagnostic_once(&mut folded.diagnostics, diagnostic);
            }
            if let Some(diagnostic) = attribute_binding_type_diagnostic(&folded, entity) {
                push_diagnostic_once(&mut folded.diagnostics, diagnostic);
            }
            if let Some(diagnostic) = conditional_type_diagnostic(&folded, entity) {
                push_diagnostic_once(&mut folded.diagnostics, diagnostic);
            }
            if let Some(diagnostic) = list_iterable_type_diagnostic(&folded, entity) {
                push_diagnostic_once(&mut folded.diagnostics, diagnostic);
            }
            if let Some(diagnostic) = member_access_type_diagnostic(&folded, entity) {
                push_diagnostic_once(&mut folded.diagnostics, diagnostic);
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

fn constant_expression_diagnostic_code_from_node(
    kind: &crate::ExpressionNodeKind,
) -> crate::TypeDiagnosticCode {
    match kind {
        crate::ExpressionNodeKind::Arithmetic { .. } => {
            crate::TypeDiagnosticCode::InvalidArithmeticOperator
        }
        crate::ExpressionNodeKind::Comparison { .. } => {
            crate::TypeDiagnosticCode::InvalidComparisonOperator
        }
        crate::ExpressionNodeKind::Boolean(_) | crate::ExpressionNodeKind::Logical { .. } => {
            crate::TypeDiagnosticCode::InvalidLogicalOperator
        }
        crate::ExpressionNodeKind::Literal(_)
        | crate::ExpressionNodeKind::NullishCoalescing { .. } => {
            crate::TypeDiagnosticCode::InvalidNullishOperator
        }
        crate::ExpressionNodeKind::Identifier(_)
        | crate::ExpressionNodeKind::ThisMember { .. }
        | crate::ExpressionNodeKind::MemberAccess { .. } => {
            unreachable!("constant folding only evaluates state initializer expressions")
        }
        crate::ExpressionNodeKind::Unary { .. } => crate::TypeDiagnosticCode::InvalidUnaryOperator,
    }
}

fn constant_expression_kind_name_from_node(kind: &crate::ExpressionNodeKind) -> &'static str {
    match kind {
        crate::ExpressionNodeKind::Arithmetic { .. } => "arithmetic",
        crate::ExpressionNodeKind::Comparison { .. } => "comparison",
        crate::ExpressionNodeKind::Boolean(_) | crate::ExpressionNodeKind::Logical { .. } => {
            "logical"
        }
        crate::ExpressionNodeKind::Literal(_)
        | crate::ExpressionNodeKind::NullishCoalescing { .. } => "nullish-coalescing",
        crate::ExpressionNodeKind::Identifier(_)
        | crate::ExpressionNodeKind::ThisMember { .. }
        | crate::ExpressionNodeKind::MemberAccess { .. } => {
            unreachable!("constant folding only evaluates state initializer expressions")
        }
        crate::ExpressionNodeKind::Unary { .. } => "unary",
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

fn folded_type_mismatch_diagnostic(
    model: &ApplicationSemanticModel,
    field: &crate::component_graph::StateField,
    class_name: &str,
) -> Option<ComponentDiagnostic> {
    let declared_type = field.declared_type.as_ref()?;
    let target = model.semantic_types.assignments.get(&field.id)?;
    let source = crate::state_initializer_value_type(field.initial_value.as_ref()?);
    (!crate::is_assignable(&source, &target.semantic_type)).then(|| ComponentDiagnostic {
        provenance: Some(declared_type.provenance.clone()),
        code: crate::TypeDiagnosticCode::IncompatibleStateInitializer
            .as_str()
            .to_string(),
        message: format!(
            "state field `{}` in class `{class_name}` declares `{}` but initializes with `{}`",
            field.name,
            declared_type.text,
            state_initializer_type_name(&source)
        ),
    })
}

fn unknown_declared_type_diagnostic(
    model: &ApplicationSemanticModel,
    field: &crate::component_graph::StateField,
    component: &crate::component_graph::ComponentNode,
) -> Option<ComponentDiagnostic> {
    let declared_type = field.declared_type.as_ref()?;
    (!model.semantic_types.assignments.contains_key(&field.id)).then(|| ComponentDiagnostic {
        provenance: Some(declared_type.provenance.clone()),
        code: crate::TypeDiagnosticCode::UnknownType.as_str().to_string(),
        message: format!(
            "state field `{}` in class `{}` declares unresolved type `{}`",
            field.name, component.class_name, declared_type.text
        ),
    })
}

fn action_assignment_mismatch_diagnostic(
    model: &ApplicationSemanticModel,
    component: &crate::component_graph::ComponentNode,
    action: &crate::component_graph::ComponentAction,
) -> Option<ComponentDiagnostic> {
    let StateOperation::Assign(value) = &action.operation else {
        return None;
    };
    let field = component
        .state_fields
        .iter()
        .find(|field| field.name == action.field)?;
    let declared_type = field.declared_type.as_ref()?;
    let target = model.semantic_types.assignments.get(&field.id)?;
    let source = crate::state_initializer_value_type(value);
    (!crate::is_assignable(&source, &target.semantic_type)).then(|| ComponentDiagnostic {
        provenance: model.provenance.get(&action.id).cloned(),
        code: crate::TypeDiagnosticCode::IncompatibleAssignment
            .as_str()
            .to_string(),
        message: format!(
            "state field `{}` in class `{}` declares `{}` but action `{}` assigns `{}`",
            field.name,
            component.class_name,
            declared_type.text,
            action.method,
            state_initializer_type_name(&source)
        ),
    })
}

fn compound_mutation_type_diagnostics(
    model: &ApplicationSemanticModel,
    component: &crate::component_graph::ComponentNode,
    action: &crate::component_graph::ComponentAction,
) -> Vec<ComponentDiagnostic> {
    let Some(field) = component
        .state_fields
        .iter()
        .find(|field| field.name == action.field)
    else {
        return Vec::new();
    };
    let Some(declared_type) = field.declared_type.as_ref() else {
        return Vec::new();
    };
    let Some(target) = model.semantic_types.assignments.get(&field.id) else {
        return Vec::new();
    };
    let provenance = model.provenance.get(&action.id).cloned();
    let number_compatible =
        crate::is_assignable(&crate::SemanticType::Number, &target.semantic_type);
    let boolean_compatible =
        crate::is_assignable(&crate::SemanticType::Boolean, &target.semantic_type);

    match &action.operation {
        StateOperation::Toggle if !boolean_compatible => vec![ComponentDiagnostic {
            provenance,
            code: crate::TypeDiagnosticCode::InvalidToggleTarget
                .as_str()
                .to_string(),
            message: format!(
                "state field `{}` in class `{}` declares `{}` but action `{}` applies a boolean toggle",
                field.name, component.class_name, declared_type.text, action.method
            ),
        }],
        StateOperation::Increment | StateOperation::Decrement if !number_compatible => {
            let operation = if matches!(action.operation, StateOperation::Increment) {
                "increment"
            } else {
                "decrement"
            };
            vec![ComponentDiagnostic {
                provenance,
                code: crate::TypeDiagnosticCode::InvalidNumericMutationTarget
                    .as_str()
                    .to_string(),
                message: format!(
                    "state field `{}` in class `{}` declares `{}` but action `{}` applies numeric {}",
                    field.name, component.class_name, declared_type.text, action.method, operation
                ),
            }]
        }
        StateOperation::AddAssign(value) | StateOperation::SubtractAssign(value) => {
            let operation = if matches!(action.operation, StateOperation::AddAssign(_)) {
                "add assignment"
            } else {
                "subtract assignment"
            };
            let mut diagnostics = Vec::new();
            if !number_compatible {
                diagnostics.push(ComponentDiagnostic {
                    provenance: provenance.clone(),
                    code: crate::TypeDiagnosticCode::InvalidCompoundMutationTarget
                        .as_str()
                        .to_string(),
                    message: format!(
                        "state field `{}` in class `{}` declares `{}` but action `{}` applies numeric {}",
                        field.name, component.class_name, declared_type.text, action.method, operation
                    ),
                });
            }
            let source = crate::state_initializer_value_type(value);
            if !crate::is_assignable(&source, &crate::SemanticType::Number) {
                diagnostics.push(ComponentDiagnostic {
                    provenance,
                    code: crate::TypeDiagnosticCode::InvalidCompoundMutationOperand
                        .as_str()
                        .to_string(),
                    message: format!(
                        "action `{}` applies numeric {} to state field `{}` with `{}` operand",
                        action.method, operation, field.name, state_initializer_type_name(&source)
                    ),
                });
            }
            diagnostics
        }
        _ => Vec::new(),
    }
}

fn template_binding_type_diagnostic(
    model: &ApplicationSemanticModel,
    entity: &crate::TemplateSemanticEntity,
) -> Option<ComponentDiagnostic> {
    if entity.kind != crate::TemplateSemanticKind::Binding {
        return None;
    }
    let assignment = model.semantic_types.assignments.get(&entity.id)?;
    (!is_text_renderable(&assignment.semantic_type)).then(|| ComponentDiagnostic {
        provenance: Some(entity.provenance.clone()),
        code: crate::TypeDiagnosticCode::NonRenderableValue
            .as_str()
            .to_string(),
        message: format!(
            "template text binding `{}` cannot render a {} value directly",
            entity.expression.as_deref().unwrap_or("<unknown>"),
            state_initializer_type_name(&assignment.semantic_type)
        ),
    })
}

fn attribute_binding_type_diagnostic(
    model: &ApplicationSemanticModel,
    entity: &crate::TemplateSemanticEntity,
) -> Option<ComponentDiagnostic> {
    if entity.kind != crate::TemplateSemanticKind::AttributeBinding {
        return None;
    }
    let name = entity.attribute_name.as_deref()?;
    let contract = crate::dom_binding_contract(name)?;
    let assignment = model.semantic_types.assignments.get(&entity.id)?;
    (!crate::is_assignable(&assignment.semantic_type, &contract.semantic_type)).then(|| {
        ComponentDiagnostic {
            provenance: Some(entity.provenance.clone()),
            code: crate::TypeDiagnosticCode::InvalidBinding
                .as_str()
                .to_string(),
            message: format!(
                "{} binding `{}` requires {}, but expression `{}` has {}",
                match contract.kind {
                    crate::DomBindingKind::Attribute => "attribute",
                    crate::DomBindingKind::Property => "property",
                },
                name,
                state_initializer_type_name(&contract.semantic_type),
                entity.expression.as_deref().unwrap_or("<unknown>"),
                state_initializer_type_name(&assignment.semantic_type)
            ),
        }
    })
}

fn conditional_type_diagnostic(
    model: &ApplicationSemanticModel,
    entity: &crate::TemplateSemanticEntity,
) -> Option<ComponentDiagnostic> {
    if entity.kind != crate::TemplateSemanticKind::Conditional {
        return None;
    }
    let assignment = model.semantic_types.assignments.get(&entity.id)?;
    (!is_boolean_condition(&assignment.semantic_type)).then(|| ComponentDiagnostic {
        provenance: Some(entity.provenance.clone()),
        code: crate::TypeDiagnosticCode::InvalidCondition
            .as_str()
            .to_string(),
        message: format!(
            "conditional expression `{}` requires boolean, but has {}",
            entity.expression.as_deref().unwrap_or("<unknown>"),
            state_initializer_type_name(&assignment.semantic_type)
        ),
    })
}

fn list_iterable_type_diagnostic(
    model: &ApplicationSemanticModel,
    entity: &crate::TemplateSemanticEntity,
) -> Option<ComponentDiagnostic> {
    if entity.kind != crate::TemplateSemanticKind::List {
        return None;
    }
    let assignment = model.semantic_types.assignments.get(&entity.id)?;
    let iterable = matches!(
        assignment.semantic_type,
        crate::SemanticType::Array(_)
            | crate::SemanticType::Tuple(_)
            | crate::SemanticType::Unknown
    );
    (!iterable).then(|| ComponentDiagnostic {
        provenance: Some(entity.provenance.clone()),
        code: crate::TypeDiagnosticCode::NonIterableList
            .as_str()
            .to_string(),
        message: format!(
            "list iterable `{}` requires an array-like value, but has {}",
            entity.expression.as_deref().unwrap_or("<unknown>"),
            state_initializer_type_name(&assignment.semantic_type)
        ),
    })
}

fn member_access_type_diagnostic(
    model: &ApplicationSemanticModel,
    entity: &crate::TemplateSemanticEntity,
) -> Option<ComponentDiagnostic> {
    let access = model.semantic_types.member_accesses.get(&entity.id)?;
    access.semantic_type.is_none().then(|| ComponentDiagnostic {
        provenance: Some(entity.provenance.clone()),
        code: crate::TypeDiagnosticCode::MissingMember
            .as_str()
            .to_string(),
        message: format!(
            "member access `{}` does not resolve against its canonical object type",
            access.expression
        ),
    })
}

fn is_boolean_condition(semantic_type: &crate::SemanticType) -> bool {
    match semantic_type {
        crate::SemanticType::Unknown
        | crate::SemanticType::Boolean
        | crate::SemanticType::BooleanLiteral(_) => true,
        crate::SemanticType::Union(members) => members.iter().all(is_boolean_condition),
        _ => false,
    }
}

fn is_text_renderable(semantic_type: &crate::SemanticType) -> bool {
    match semantic_type {
        crate::SemanticType::Unknown
        | crate::SemanticType::Null
        | crate::SemanticType::Boolean
        | crate::SemanticType::Number
        | crate::SemanticType::String
        | crate::SemanticType::BooleanLiteral(_)
        | crate::SemanticType::NumberLiteral(_)
        | crate::SemanticType::StringLiteral(_) => true,
        crate::SemanticType::Union(members) => members.iter().all(is_text_renderable),
        crate::SemanticType::Never
        | crate::SemanticType::Array(_)
        | crate::SemanticType::Tuple(_)
        | crate::SemanticType::Object(_)
        | crate::SemanticType::Resource(_) => false,
    }
}

fn push_diagnostic_once(
    diagnostics: &mut Vec<ComponentDiagnostic>,
    diagnostic: ComponentDiagnostic,
) {
    if !diagnostics.contains(&diagnostic) {
        diagnostics.push(diagnostic);
    }
}

fn state_initializer_type_name(semantic_type: &crate::SemanticType) -> &'static str {
    match semantic_type {
        crate::SemanticType::Unknown => "unknown",
        crate::SemanticType::Never => "never",
        crate::SemanticType::Null => "null",
        crate::SemanticType::Boolean | crate::SemanticType::BooleanLiteral(_) => "boolean",
        crate::SemanticType::Number | crate::SemanticType::NumberLiteral(_) => "number",
        crate::SemanticType::String | crate::SemanticType::StringLiteral(_) => "string",
        crate::SemanticType::Array(_) => "array",
        crate::SemanticType::Tuple(_) => "tuple",
        crate::SemanticType::Object(_) => "object",
        crate::SemanticType::Union(_) => "union",
        crate::SemanticType::Resource(_) => "resource",
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
    fn rejects_non_renderable_direct_text_bindings_with_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/NonRenderableBinding.tsx",
            r#"
@component("x-non-renderable")
class NonRenderableBinding extends Component {
  user = state({ id: "1" });
  label = state("EdgeZero");

  render() {
    return <p>{this.user}{this.label}</p>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let binding = asm
            .template_entities
            .iter()
            .find(|entity| entity.expression.as_deref() == Some("this.user"))
            .expect("user text binding");

        assert_eq!(
            asm.semantic_types.assignments[&binding.id].semantic_type,
            crate::SemanticType::Object(crate::ObjectType {
                properties: std::collections::BTreeMap::from([(
                    "id".to_string(),
                    crate::SemanticType::String,
                )]),
            })
        );

        let folded = ConstantFoldingPass.transform(&asm);
        assert!(folded.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "EZC1027"
                && diagnostic.message.contains("this.user")
                && diagnostic.message.contains("object")
        }));
        assert!(!folded
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("this.label")));
    }

    #[test]
    fn validates_typed_dom_attribute_and_property_bindings() {
        let parsed = ezc_parser::parse_file(
            "src/TypedAttributes.tsx",
            r#"
@component("x-typed-attributes")
class TypedAttributes extends Component {
  enabled = state(true);
  label = state("EdgeZero");
  count = state(1);

  render() {
    return <><button disabled={this.label} value={this.count}>Save</button><a href={this.enabled}>Link</a></>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let value_binding = asm
            .template_entities
            .iter()
            .find(|entity| entity.attribute_name.as_deref() == Some("value"))
            .expect("value binding");
        assert_eq!(
            asm.semantic_types.assignments[&value_binding.id].semantic_type,
            crate::SemanticType::Number
        );

        let folded = ConstantFoldingPass.transform(&asm);
        let diagnostics = folded
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "EZC1028")
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("property binding `disabled`")
                && diagnostic.message.contains("string")
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("attribute binding `href`")
                && diagnostic.message.contains("boolean")
        }));
    }

    #[test]
    fn requires_boolean_canonical_template_conditions() {
        let parsed = ezc_parser::parse_file(
            "src/TypedConditions.tsx",
            r#"
@component("x-typed-conditions")
class TypedConditions extends Component {
  enabled = state(true);
  count = state(1);

  render() {
    return <>{this.enabled ? <p>On</p> : <p>Off</p>}{this.count ? <p>Count</p> : <p>Empty</p>}</>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let count_condition = asm
            .template_entities
            .iter()
            .find(|entity| {
                entity.kind == crate::TemplateSemanticKind::Conditional
                    && entity.expression.as_deref() == Some("this.count")
            })
            .expect("count conditional");
        assert_eq!(
            asm.semantic_types.assignments[&count_condition.id].semantic_type,
            crate::SemanticType::Number
        );

        let folded = ConstantFoldingPass.transform(&asm);
        let diagnostics = folded
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "EZC1029")
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("this.count"));
    }

    #[test]
    fn types_list_iterables_and_rejects_non_arrays() {
        let parsed = ezc_parser::parse_file(
            "src/TypedLists.tsx",
            r#"
@component("x-typed-lists")
class TypedLists extends Component {
  items = state([{ id: "a" }]);
  count = state(1);

  render() {
    return <><ul>{this.items.map((item, index) => <li key={item.id}>{index}</li>)}</ul><div>{this.count.map((item) => <span>{item}</span>)}</div></>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let item_list = asm
            .template_entities
            .iter()
            .find(|entity| entity.expression.as_deref() == Some("this.items"))
            .expect("items list");
        let scope = asm
            .semantic_types
            .list_scopes
            .get(&item_list.id)
            .expect("typed item list scope");
        assert_eq!(scope.item_name, "item");
        assert_eq!(scope.index_type, Some(crate::SemanticType::Number));
        assert!(matches!(scope.item_type, crate::SemanticType::Object(_)));

        let folded = ConstantFoldingPass.transform(&asm);
        assert!(folded.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "EZC1030" && diagnostic.message.contains("this.count")
        }));
    }

    #[test]
    fn resolves_typed_list_item_member_accesses() {
        let parsed = ezc_parser::parse_file(
            "src/TypedMembers.tsx",
            r#"
@component("x-typed-members")
class TypedMembers extends Component {
  todos = state([{ details: { region: "west" } }]);

  render() {
    return <ul>{this.todos.map((todo) => <li key={todo.details.region}>{todo.details.region}{todo.missing}</li>)}</ul>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let region = asm
            .template_entities
            .iter()
            .find(|entity| entity.expression.as_deref() == Some("todo.details.region"))
            .expect("region member binding");
        assert_eq!(
            asm.semantic_types.assignments[&region.id].semantic_type,
            crate::SemanticType::String
        );
        assert_eq!(
            asm.semantic_types.member_accesses[&region.id].semantic_type,
            Some(crate::SemanticType::String)
        );

        let folded = ConstantFoldingPass.transform(&asm);
        assert!(folded.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "EZC1031" && diagnostic.message.contains("todo.missing")
        }));
    }

    #[test]
    fn validates_state_initializers_with_canonical_semantic_assignability() {
        let parsed = ezc_parser::parse_file(
            "src/StateCompatibility.tsx",
            r#"
@component("x-state-compatibility")
class StateCompatibility extends Component {
  names: string[] = state([]);
  filter: "all" | "active" = state("all");
  user: { id: string } | null = state(null);
  invalid: string[] = state([1]);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let folded = ConstantFoldingPass.transform(&asm);
        let diagnostics = folded
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "EZC1016")
            .collect::<Vec<_>>();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("invalid"));
        assert!(diagnostics[0].message.contains("declares `string[]`"));
        assert!(diagnostics[0].message.contains("initializes with `tuple`"));
    }

    #[test]
    fn reports_unresolved_declared_state_types_with_a_stable_type_diagnostic() {
        let parsed = ezc_parser::parse_file(
            "src/UnknownStateType.tsx",
            r#"
@component("x-unknown-state-type")
class UnknownStateType extends Component {
  value: MissingType = state(null);
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let folded = ConstantFoldingPass.transform(&asm);
        assert!(folded.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::TypeDiagnosticCode::UnknownType.as_str()
                && diagnostic.message.contains("MissingType")
        }));
    }

    #[test]
    fn uses_canonical_expression_provenance_for_folding_diagnostics() {
        let parsed = ezc_parser::parse_file(
            "src/InvalidExpression.tsx",
            r#"
@component("x-invalid-expression")
class InvalidExpression extends Component {
  total = state(10 / 0);
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let field = &asm.components[0].state_fields[0];
        let root = asm
            .expression_graph
            .root_for(&field.id)
            .expect("expression root");
        let expected_provenance = asm
            .expression_graph
            .nodes
            .get(root)
            .expect("expression root node")
            .provenance
            .clone();

        let folded = ConstantFoldingPass.transform(&asm);
        let diagnostic = folded
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "EZC1022")
            .expect("constant-expression diagnostic");

        assert_eq!(diagnostic.provenance.as_ref(), Some(&expected_provenance));
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
