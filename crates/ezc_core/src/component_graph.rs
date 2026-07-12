use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::Serialize;

use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;
use crate::semantic_reference::{SemanticReference, SemanticReferenceKind};

use ezc_parser::{
    ParsedArithmeticExpression, ParsedArithmeticExpressionKind, ParsedArithmeticOperator,
    ParsedClass, ParsedComparisonOperator, ParsedConstantExpression, ParsedConstantExpressionKind,
    ParsedEventHandler, ParsedFile, ParsedJsxAttribute, ParsedJsxAttributeValue, ParsedJsxChild,
    ParsedJsxConditional, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode, ParsedLogicalOperator,
    ParsedSerializableValue, ParsedStateOperation, ParsedUnaryOperator, SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentGraph {
    pub components: Vec<ComponentNode>,
    pub diagnostics: Vec<ComponentDiagnostic>,
    pub references: Vec<SemanticReference>,
    pub provenance: BTreeMap<SemanticId, SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentNode {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub class_name: String,
    pub element_name: Option<String>,
    pub route_path: Option<String>,
    pub state_fields: Vec<StateField>,
    pub methods: Vec<ComponentMethod>,
    pub actions: Vec<ComponentAction>,
    pub render: Option<RenderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateField {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub initial_value: Option<SerializableValue>,
    pub initial_expression: Option<ConstantExpression>,
    pub declared_type: Option<DeclaredStateType>,
}

/// A compiler-owned numeric arithmetic expression lowered from a state initializer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithmeticExpression {
    pub kind: ArithmeticExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticExpressionKind {
    Number(String),
    Binary {
        operator: ArithmeticOperator,
        left: Box<ArithmeticExpression>,
        right: Box<ArithmeticExpression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticEvaluationError {
    InvalidNumber(String),
    DivisionByZero,
    NonFiniteResult,
}

/// A compiler-owned constant expression lowered from a state initializer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantExpression {
    pub kind: ConstantExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstantExpressionKind {
    Literal(SerializableValue),
    Boolean(bool),
    Arithmetic(ArithmeticExpression),
    Comparison {
        operator: ComparisonOperator,
        left: ArithmeticExpression,
        right: ArithmeticExpression,
    },
    Logical {
        operator: LogicalOperator,
        left: Box<ConstantExpression>,
        right: Box<ConstantExpression>,
    },
    NullishCoalescing {
        left: Box<ConstantExpression>,
        right: Box<ConstantExpression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<ConstantExpression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstantEvaluationError {
    Arithmetic(ArithmeticEvaluationError),
}

impl ArithmeticExpression {
    /// Evaluate a finite constant numeric expression using `EdgeZero` arithmetic semantics.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid numeric literal, division or remainder by zero,
    /// or a non-finite intermediate or final result.
    pub fn evaluate(&self) -> Result<SerializableValue, ArithmeticEvaluationError> {
        self.evaluate_number()
            .map(|value| SerializableValue::Number(value.to_string()))
    }

    fn evaluate_number(&self) -> Result<f64, ArithmeticEvaluationError> {
        let value = match &self.kind {
            ArithmeticExpressionKind::Number(value) => value
                .parse::<f64>()
                .map_err(|_| ArithmeticEvaluationError::InvalidNumber(value.clone()))?,
            ArithmeticExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate_number()?;
                let right = right.evaluate_number()?;
                match operator {
                    ArithmeticOperator::Add => left + right,
                    ArithmeticOperator::Subtract => left - right,
                    ArithmeticOperator::Multiply => left * right,
                    ArithmeticOperator::Divide => {
                        if right == 0.0 {
                            return Err(ArithmeticEvaluationError::DivisionByZero);
                        }
                        left / right
                    }
                    ArithmeticOperator::Remainder => {
                        if right == 0.0 {
                            return Err(ArithmeticEvaluationError::DivisionByZero);
                        }
                        left % right
                    }
                }
            }
        };

        value
            .is_finite()
            .then_some(value)
            .ok_or(ArithmeticEvaluationError::NonFiniteResult)
    }
}

impl ConstantExpression {
    /// Evaluate a constant state initializer using `EdgeZero` expression semantics.
    ///
    /// # Errors
    ///
    /// Returns an error when one of the expression's numeric arithmetic operands is invalid.
    pub fn evaluate(&self) -> Result<SerializableValue, ConstantEvaluationError> {
        match &self.kind {
            ConstantExpressionKind::Literal(value) => Ok(value.clone()),
            ConstantExpressionKind::Boolean(value) => Ok(SerializableValue::Boolean(*value)),
            ConstantExpressionKind::Arithmetic(expression) => expression
                .evaluate()
                .map_err(ConstantEvaluationError::Arithmetic),
            ConstantExpressionKind::Comparison {
                operator,
                left,
                right,
            } => {
                let left = left
                    .evaluate_number()
                    .map_err(ConstantEvaluationError::Arithmetic)?;
                let right = right
                    .evaluate_number()
                    .map_err(ConstantEvaluationError::Arithmetic)?;
                Ok(SerializableValue::Boolean(match operator {
                    ComparisonOperator::Equal => numbers_are_equal(left, right),
                    ComparisonOperator::NotEqual => !numbers_are_equal(left, right),
                    ComparisonOperator::LessThan => left < right,
                    ComparisonOperator::LessThanOrEqual => left <= right,
                    ComparisonOperator::GreaterThan => left > right,
                    ComparisonOperator::GreaterThanOrEqual => left >= right,
                }))
            }
            ConstantExpressionKind::Logical {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate_boolean()?;
                let value = match (operator, left) {
                    (LogicalOperator::And, false) => false,
                    (LogicalOperator::Or, true) => true,
                    (LogicalOperator::And | LogicalOperator::Or, _) => right.evaluate_boolean()?,
                };
                Ok(SerializableValue::Boolean(value))
            }
            ConstantExpressionKind::NullishCoalescing { left, right } => {
                let value = left.evaluate()?;
                if matches!(value, SerializableValue::Null) {
                    right.evaluate()
                } else {
                    Ok(value)
                }
            }
            ConstantExpressionKind::Unary { operator, operand } => match operator {
                UnaryOperator::Not => Ok(SerializableValue::Boolean(!operand.evaluate_boolean()?)),
                UnaryOperator::Plus | UnaryOperator::Minus => {
                    let SerializableValue::Number(value) = operand.evaluate()? else {
                        unreachable!("parser requires numeric unary operands");
                    };
                    let value = value.parse::<f64>().map_err(|_| {
                        ConstantEvaluationError::Arithmetic(
                            ArithmeticEvaluationError::InvalidNumber(value),
                        )
                    })?;
                    let value = if matches!(operator, UnaryOperator::Minus) {
                        -value
                    } else {
                        value
                    };
                    value
                        .is_finite()
                        .then_some(SerializableValue::Number(value.to_string()))
                        .ok_or(ConstantEvaluationError::Arithmetic(
                            ArithmeticEvaluationError::NonFiniteResult,
                        ))
                }
            },
        }
    }

    fn evaluate_boolean(&self) -> Result<bool, ConstantEvaluationError> {
        let SerializableValue::Boolean(value) = self.evaluate()? else {
            unreachable!("parser only permits boolean constants as logical operands");
        };
        Ok(value)
    }
}

fn numbers_are_equal(left: f64, right: f64) -> bool {
    // EdgeZero constant expressions deliberately use exact finite numeric equality.
    // No tolerance is compiler semantics because the language has no approximate
    // comparison operator or configurable precision model.
    #[allow(clippy::float_cmp)]
    {
        left == right
    }
}

impl fmt::Display for ArithmeticExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ArithmeticExpressionKind::Number(value) => value.fmt(formatter),
            ArithmeticExpressionKind::Binary {
                operator,
                left,
                right,
            } => write!(formatter, "({left} {operator} {right})"),
        }
    }
}

impl fmt::Display for ConstantExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ConstantExpressionKind::Literal(value) => format_constant_literal(value, formatter),
            ConstantExpressionKind::Boolean(value) => value.fmt(formatter),
            ConstantExpressionKind::Arithmetic(expression) => expression.fmt(formatter),
            ConstantExpressionKind::Comparison {
                operator,
                left,
                right,
            } => write!(formatter, "({left} {operator} {right})"),
            ConstantExpressionKind::Logical {
                operator,
                left,
                right,
            } => write!(formatter, "({left} {operator} {right})"),
            ConstantExpressionKind::NullishCoalescing { left, right } => {
                write!(formatter, "({left} ?? {right})")
            }
            ConstantExpressionKind::Unary { operator, operand } => {
                write!(formatter, "({operator}{operand})")
            }
        }
    }
}

fn format_constant_literal(
    value: &SerializableValue,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match value {
        SerializableValue::Null => formatter.write_str("null"),
        SerializableValue::Number(value) => formatter.write_str(value),
        SerializableValue::String(value) => write!(formatter, "{value:?}"),
        SerializableValue::Boolean(value) => write!(formatter, "{value}"),
        SerializableValue::Array(_) | SerializableValue::Object(_) => {
            unreachable!("nullish constant literals are primitive values")
        }
    }
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operator = match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
        };
        formatter.write_str(operator)
    }
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operator = match self {
            Self::Equal => "===",
            Self::NotEqual => "!==",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        };
        formatter.write_str(operator)
    }
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::And => "&&",
            Self::Or => "||",
        })
    }
}
impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Not => "!",
            Self::Plus => "+",
            Self::Minus => "-",
        })
    }
}

impl fmt::Display for ArithmeticEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumber(value) => {
                write!(formatter, "unsupported numeric literal `{value}`")
            }
            Self::DivisionByZero => formatter.write_str("division or remainder by zero"),
            Self::NonFiniteResult => formatter.write_str("non-finite result"),
        }
    }
}

impl fmt::Display for ConstantEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arithmetic(error) => error.fmt(formatter),
        }
    }
}

/// Explicit state type metadata carried from the parser into canonical compiler data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredStateType {
    pub text: String,
    pub provenance: SourceProvenance,
    pub kind: Option<DeclaredStateTypeKind>,
}

/// The primitive state type forms currently recognized without type inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredStateTypeKind {
    String,
    Number,
    Boolean,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum SerializableValue {
    Null,
    Number(String),
    String(String),
    Boolean(bool),
    Array(Vec<SerializableValue>),
    Object(BTreeMap<String, SerializableValue>),
}

impl SerializableValue {
    #[must_use]
    pub fn render_text(&self) -> String {
        match self {
            Self::Number(value) | Self::String(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
            Self::Null | Self::Array(_) | Self::Object(_) => String::new(),
        }
    }

    #[must_use]
    pub fn member_path_value(&self, path: &str) -> Option<&Self> {
        if path.is_empty() {
            return None;
        }

        path.split('.').try_fold(self, |value, member| match value {
            Self::Object(values) if !member.is_empty() => values.get(member),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMethod {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub local_variables: Vec<MethodLocalVariable>,
    pub parameters: Vec<MethodParameter>,
}

/// A compiler-owned declaration of a supported method parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodParameter {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodLocalVariable {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub value: SerializableValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAction {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub method: String,
    pub operation: StateOperation,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateOperation {
    Increment,
    Decrement,
    AddAssign(SerializableValue),
    SubtractAssign(SerializableValue),
    Assign(SerializableValue),
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderChild {
    Text {
        value: String,
        span: SourceSpan,
    },
    Binding {
        expression: String,
        span: SourceSpan,
    },
    Element(RenderElement),
    Fragment(RenderFragment),
    Conditional(RenderConditional),
    List(RenderList),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderElement {
    pub tag_name: String,
    pub span: SourceSpan,
    pub attributes: Vec<RenderAttribute>,
    pub event_handlers: Vec<RenderEventHandler>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFragment {
    pub span: SourceSpan,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderConditional {
    pub condition: String,
    pub span: SourceSpan,
    pub when_true: Vec<RenderChild>,
    pub when_false: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderList {
    pub iterable: String,
    pub item_variable: String,
    pub index_variable: Option<String>,
    pub key_expression: String,
    pub span: SourceSpan,
    pub item_template: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAttribute {
    pub name: String,
    pub value: RenderAttributeValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderAttributeValue {
    Boolean,
    Static(String),
    Expression(Option<String>),
    Spread(Option<String>),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderEventHandler {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub event: String,
    pub handler: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderModel {
    pub root_element: Option<String>,
    pub root_span: Option<SourceSpan>,
    pub root_fragment: Option<RenderFragment>,
    pub attributes: Vec<RenderAttribute>,
    pub bindings: Vec<String>,
    pub event_handlers: Vec<RenderEventHandler>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDiagnostic {
    pub code: String,
    pub message: String,
    pub provenance: Option<SourceProvenance>,
}

#[must_use]
pub fn build_component_graph(parsed: &ParsedFile) -> ComponentGraph {
    build_component_graph_with_identity(parsed, false)
}

/// Build compiler semantics with a source-module-qualified identity root.
///
/// The canonical application model and compiler frontend use this path. The
/// legacy graph builder remains available for backend compatibility while
/// runtime artifact contracts are migrated independently.
#[must_use]
pub fn build_component_graph_for_module(parsed: &ParsedFile) -> ComponentGraph {
    build_component_graph_with_identity(parsed, true)
}

fn build_component_graph_with_identity(
    parsed: &ParsedFile,
    module_qualified_identity: bool,
) -> ComponentGraph {
    let mut components = Vec::new();
    let mut diagnostics = Vec::new();
    let mut references = Vec::new();
    let mut provenance = BTreeMap::new();

    for class in &parsed.classes {
        let element_name = decorator_argument(class, "component");
        let id = if module_qualified_identity {
            SemanticId::component_in_module(&parsed.path, element_name.as_deref(), &class.name)
        } else {
            SemanticId::component(element_name.as_deref(), &class.name)
        };
        let component = build_component_node(class, &parsed.path, id, &mut diagnostics);
        let component_provenance = collect_component_provenance(class, &component, &parsed.path);
        references.extend(collect_semantic_references(
            &component,
            &component_provenance,
        ));
        provenance.extend(component_provenance);
        components.push(component);
    }

    if parsed.classes.is_empty() && parsed.diagnostics.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            provenance: None,
            code: "EZC1000".to_string(),
            message: "no component classes found".to_string(),
        });
    }

    ComponentGraph {
        components,
        diagnostics,
        references,
        provenance,
    }
}

fn build_component_node(
    class: &ParsedClass,
    path: &Path,
    id: SemanticId,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) -> ComponentNode {
    let element_name = decorator_argument(class, "component");
    let route_path = decorator_argument(class, "route");

    if element_name.is_none() {
        diagnostics.push(ComponentDiagnostic {
            provenance: None,
            code: "EZC1001".to_string(),
            message: format!("class `{}` is missing @component(...)", class.name),
        });
    }

    let state_fields = state_fields_from_class(class, path, &id);

    collect_declared_state_type_diagnostics(&state_fields, &class.name, diagnostics);
    collect_declared_state_action_type_diagnostics(class, &state_fields, path, diagnostics);
    collect_declared_state_toggle_type_diagnostics(class, &state_fields, path, diagnostics);
    collect_declared_state_numeric_action_type_diagnostics(class, &state_fields, path, diagnostics);
    collect_declared_state_compound_numeric_action_type_diagnostics(
        class,
        &state_fields,
        path,
        diagnostics,
    );

    let methods = class
        .methods
        .iter()
        .map(|method| ComponentMethod {
            id: id.method(&method.name),
            owner: SemanticOwner::entity(id.clone()),
            name: method.name.clone(),
            local_variables: method
                .local_variables
                .iter()
                .enumerate()
                .map(|(index, local)| MethodLocalVariable {
                    id: id.method(&method.name).local_variable(&local.name, index),
                    owner: SemanticOwner::entity(id.method(&method.name)),
                    name: local.name.clone(),
                    value: serializable_value_from_parsed(&local.value),
                    span: local.span,
                })
                .collect(),
            parameters: method
                .parameters
                .iter()
                .map(|parameter| MethodParameter {
                    name: parameter.name.clone(),
                    span: parameter.span,
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    let actions = class
        .methods
        .iter()
        .flat_map(|method| {
            method
                .state_updates
                .iter()
                .enumerate()
                .map(|(index, update)| ComponentAction {
                    id: id.action(&method.name, index),
                    owner: SemanticOwner::entity(id.method(&method.name)),
                    method: method.name.clone(),
                    operation: state_operation_from_parsed(&update.operation),
                    field: update.field.clone(),
                })
        })
        .collect::<Vec<_>>();

    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .map(|method| render_model_from_parsed_method(method, &id));

    if render.is_none() {
        diagnostics.push(ComponentDiagnostic {
            provenance: None,
            code: "EZC1002".to_string(),
            message: format!("class `{}` is missing render()", class.name),
        });
    }

    if let Some(render) = &render {
        collect_render_binding_diagnostics(class, render, diagnostics);
        collect_render_event_diagnostics(class, render, diagnostics);
        collect_duplicate_event_diagnostics(render, &class.name, diagnostics);
        collect_render_attribute_diagnostics(render, &state_fields, &class.name, diagnostics);
        collect_render_list_diagnostics(render, &state_fields, &class.name, diagnostics);
    }

    ComponentNode {
        id,
        owner: SemanticOwner::Application,
        class_name: class.name.clone(),
        element_name,
        route_path,
        state_fields,
        methods,
        actions,
        render,
    }
}

fn state_fields_from_class(class: &ParsedClass, path: &Path, id: &SemanticId) -> Vec<StateField> {
    class
        .properties
        .iter()
        .filter(|property| property.initializer.as_deref() == Some("state(...)"))
        .map(|property| {
            let initial_expression = property
                .state_initial_expression
                .as_ref()
                .map(constant_expression_from_parsed);
            let initial_value = property
                .state_initial_value
                .as_ref()
                .map(serializable_value_from_parsed);

            StateField {
                id: id.state_field(&property.name),
                owner: SemanticOwner::entity(id.clone()),
                name: property.name.clone(),
                initial_value,
                initial_expression,
                declared_type: property.state_type_annotation.as_ref().map(|annotation| {
                    DeclaredStateType {
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                        kind: declared_state_type_kind(&annotation.text),
                    }
                }),
            }
        })
        .collect()
}

fn arithmetic_expression_from_parsed(
    expression: &ParsedArithmeticExpression,
) -> ArithmeticExpression {
    let kind = match &expression.kind {
        ParsedArithmeticExpressionKind::Number(value) => {
            ArithmeticExpressionKind::Number(value.clone())
        }
        ParsedArithmeticExpressionKind::Binary {
            operator,
            left,
            right,
        } => ArithmeticExpressionKind::Binary {
            operator: arithmetic_operator_from_parsed(*operator),
            left: Box::new(arithmetic_expression_from_parsed(left)),
            right: Box::new(arithmetic_expression_from_parsed(right)),
        },
    };

    ArithmeticExpression {
        kind,
        span: expression.span,
    }
}

fn constant_expression_from_parsed(expression: &ParsedConstantExpression) -> ConstantExpression {
    let kind = match &expression.kind {
        ParsedConstantExpressionKind::Primitive(value) => {
            ConstantExpressionKind::Literal(serializable_value_from_parsed(value))
        }
        ParsedConstantExpressionKind::Boolean(value) => ConstantExpressionKind::Boolean(*value),
        ParsedConstantExpressionKind::Arithmetic(expression) => {
            ConstantExpressionKind::Arithmetic(arithmetic_expression_from_parsed(expression))
        }
        ParsedConstantExpressionKind::Comparison {
            operator,
            left,
            right,
        } => ConstantExpressionKind::Comparison {
            operator: comparison_operator_from_parsed(*operator),
            left: arithmetic_expression_from_parsed(left),
            right: arithmetic_expression_from_parsed(right),
        },
        ParsedConstantExpressionKind::Logical {
            operator,
            left,
            right,
        } => ConstantExpressionKind::Logical {
            operator: logical_operator_from_parsed(*operator),
            left: Box::new(constant_expression_from_parsed(left)),
            right: Box::new(constant_expression_from_parsed(right)),
        },
        ParsedConstantExpressionKind::NullishCoalescing { left, right } => {
            ConstantExpressionKind::NullishCoalescing {
                left: Box::new(constant_expression_from_parsed(left)),
                right: Box::new(constant_expression_from_parsed(right)),
            }
        }
        ParsedConstantExpressionKind::Unary { operator, operand } => {
            ConstantExpressionKind::Unary {
                operator: match operator {
                    ParsedUnaryOperator::Not => UnaryOperator::Not,
                    ParsedUnaryOperator::Plus => UnaryOperator::Plus,
                    ParsedUnaryOperator::Minus => UnaryOperator::Minus,
                },
                operand: Box::new(constant_expression_from_parsed(operand)),
            }
        }
    };

    ConstantExpression {
        kind,
        span: expression.span,
    }
}

fn arithmetic_operator_from_parsed(operator: ParsedArithmeticOperator) -> ArithmeticOperator {
    match operator {
        ParsedArithmeticOperator::Add => ArithmeticOperator::Add,
        ParsedArithmeticOperator::Subtract => ArithmeticOperator::Subtract,
        ParsedArithmeticOperator::Multiply => ArithmeticOperator::Multiply,
        ParsedArithmeticOperator::Divide => ArithmeticOperator::Divide,
        ParsedArithmeticOperator::Remainder => ArithmeticOperator::Remainder,
    }
}

fn comparison_operator_from_parsed(operator: ParsedComparisonOperator) -> ComparisonOperator {
    match operator {
        ParsedComparisonOperator::Equal => ComparisonOperator::Equal,
        ParsedComparisonOperator::NotEqual => ComparisonOperator::NotEqual,
        ParsedComparisonOperator::LessThan => ComparisonOperator::LessThan,
        ParsedComparisonOperator::LessThanOrEqual => ComparisonOperator::LessThanOrEqual,
        ParsedComparisonOperator::GreaterThan => ComparisonOperator::GreaterThan,
        ParsedComparisonOperator::GreaterThanOrEqual => ComparisonOperator::GreaterThanOrEqual,
    }
}

fn logical_operator_from_parsed(operator: ParsedLogicalOperator) -> LogicalOperator {
    match operator {
        ParsedLogicalOperator::And => LogicalOperator::And,
        ParsedLogicalOperator::Or => LogicalOperator::Or,
    }
}

fn declared_state_type_kind(text: &str) -> Option<DeclaredStateTypeKind> {
    match text {
        "string" => Some(DeclaredStateTypeKind::String),
        "number" => Some(DeclaredStateTypeKind::Number),
        "boolean" => Some(DeclaredStateTypeKind::Boolean),
        "null" => Some(DeclaredStateTypeKind::Null),
        _ => None,
    }
}

fn collect_declared_state_type_diagnostics(
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for field in state_fields {
        let Some(declared_type) = field.declared_type.as_ref() else {
            continue;
        };
        let Some(declared_type_kind) = declared_type.kind else {
            continue;
        };
        let Some(initial_value_kind) = field
            .initial_value
            .as_ref()
            .and_then(primitive_serializable_value_type_kind)
        else {
            continue;
        };

        if declared_type_kind != initial_value_kind {
            diagnostics.push(ComponentDiagnostic {
                provenance: Some(declared_type.provenance.clone()),
                code: "EZC1016".to_string(),
                message: format!(
                    "state field `{}` in class `{class_name}` declares `{}` but initializes with `{}`",
                    field.name,
                    declared_state_type_kind_name(declared_type_kind),
                    declared_state_type_kind_name(initial_value_kind),
                ),
            });
        }
    }
}

fn collect_declared_state_action_type_diagnostics(
    class: &ParsedClass,
    state_fields: &[StateField],
    path: &Path,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for method in &class.methods {
        for update in &method.state_updates {
            let ParsedStateOperation::Assign(value) = &update.operation else {
                continue;
            };
            let Some(field) = state_fields.iter().find(|field| field.name == update.field) else {
                continue;
            };
            let Some(declared_type) = field.declared_type.as_ref() else {
                continue;
            };
            let Some(declared_type_kind) = declared_type.kind else {
                continue;
            };
            let value = serializable_value_from_parsed(value);
            let Some(value_kind) = primitive_serializable_value_type_kind(&value) else {
                continue;
            };

            if declared_type_kind != value_kind {
                diagnostics.push(ComponentDiagnostic {
                    provenance: Some(SourceProvenance::new(path, update.span)),
                    code: "EZC1017".to_string(),
                    message: format!(
                        "state field `{}` in class `{}` declares `{}` but action `{}` assigns `{}`",
                        field.name,
                        class.name,
                        declared_state_type_kind_name(declared_type_kind),
                        method.name,
                        declared_state_type_kind_name(value_kind),
                    ),
                });
            }
        }
    }
}

fn collect_declared_state_toggle_type_diagnostics(
    class: &ParsedClass,
    state_fields: &[StateField],
    path: &Path,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for method in &class.methods {
        for update in &method.state_updates {
            if !matches!(update.operation, ParsedStateOperation::Toggle) {
                continue;
            }
            let Some(field) = state_fields.iter().find(|field| field.name == update.field) else {
                continue;
            };
            let Some(declared_type) = field.declared_type.as_ref() else {
                continue;
            };
            let Some(declared_type_kind) = declared_type.kind else {
                continue;
            };

            if declared_type_kind != DeclaredStateTypeKind::Boolean {
                diagnostics.push(ComponentDiagnostic {
                    provenance: Some(SourceProvenance::new(path, update.span)),
                    code: "EZC1018".to_string(),
                    message: format!(
                        "state field `{}` in class `{}` declares `{}` but action `{}` applies a boolean toggle",
                        field.name,
                        class.name,
                        declared_state_type_kind_name(declared_type_kind),
                        method.name,
                    ),
                });
            }
        }
    }
}

fn collect_declared_state_numeric_action_type_diagnostics(
    class: &ParsedClass,
    state_fields: &[StateField],
    path: &Path,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for method in &class.methods {
        for update in &method.state_updates {
            let operation = match update.operation {
                ParsedStateOperation::Increment => "increment",
                ParsedStateOperation::Decrement => "decrement",
                _ => continue,
            };
            let Some(field) = state_fields.iter().find(|field| field.name == update.field) else {
                continue;
            };
            let Some(declared_type) = field.declared_type.as_ref() else {
                continue;
            };
            let Some(declared_type_kind) = declared_type.kind else {
                continue;
            };

            if declared_type_kind != DeclaredStateTypeKind::Number {
                diagnostics.push(ComponentDiagnostic {
                    provenance: Some(SourceProvenance::new(path, update.span)),
                    code: "EZC1019".to_string(),
                    message: format!(
                        "state field `{}` in class `{}` declares `{}` but action `{}` applies numeric {}",
                        field.name,
                        class.name,
                        declared_state_type_kind_name(declared_type_kind),
                        method.name,
                        operation,
                    ),
                });
            }
        }
    }
}

fn collect_declared_state_compound_numeric_action_type_diagnostics(
    class: &ParsedClass,
    state_fields: &[StateField],
    path: &Path,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for method in &class.methods {
        for update in &method.state_updates {
            let (operation, operand) = match &update.operation {
                ParsedStateOperation::AddAssign(value) => ("add assignment", value),
                ParsedStateOperation::SubtractAssign(value) => ("subtract assignment", value),
                _ => continue,
            };
            let Some(field) = state_fields.iter().find(|field| field.name == update.field) else {
                continue;
            };
            let Some(declared_type) = field.declared_type.as_ref() else {
                continue;
            };
            let Some(declared_type_kind) = declared_type.kind else {
                continue;
            };

            if declared_type_kind != DeclaredStateTypeKind::Number {
                diagnostics.push(ComponentDiagnostic {
                    provenance: Some(SourceProvenance::new(path, update.span)),
                    code: "EZC1020".to_string(),
                    message: format!(
                        "state field `{}` in class `{}` declares `{}` but action `{}` applies numeric {}",
                        field.name,
                        class.name,
                        declared_state_type_kind_name(declared_type_kind),
                        method.name,
                        operation,
                    ),
                });
            }

            let operand = serializable_value_from_parsed(operand);
            if !matches!(operand, SerializableValue::Number(_)) {
                diagnostics.push(ComponentDiagnostic {
                    provenance: Some(SourceProvenance::new(path, update.span)),
                    code: "EZC1021".to_string(),
                    message: format!(
                        "action `{}` applies numeric {} to state field `{}` with `{}` operand",
                        method.name,
                        operation,
                        field.name,
                        serializable_value_type_name(&operand),
                    ),
                });
            }
        }
    }
}

fn primitive_serializable_value_type_kind(
    value: &SerializableValue,
) -> Option<DeclaredStateTypeKind> {
    match value {
        SerializableValue::String(_) => Some(DeclaredStateTypeKind::String),
        SerializableValue::Number(_) => Some(DeclaredStateTypeKind::Number),
        SerializableValue::Boolean(_) => Some(DeclaredStateTypeKind::Boolean),
        SerializableValue::Null => Some(DeclaredStateTypeKind::Null),
        SerializableValue::Array(_) | SerializableValue::Object(_) => None,
    }
}

fn serializable_value_type_name(value: &SerializableValue) -> &'static str {
    match value {
        SerializableValue::Null => "null",
        SerializableValue::Number(_) => "number",
        SerializableValue::String(_) => "string",
        SerializableValue::Boolean(_) => "boolean",
        SerializableValue::Array(_) => "array",
        SerializableValue::Object(_) => "object",
    }
}

fn declared_state_type_kind_name(kind: DeclaredStateTypeKind) -> &'static str {
    match kind {
        DeclaredStateTypeKind::String => "string",
        DeclaredStateTypeKind::Number => "number",
        DeclaredStateTypeKind::Boolean => "boolean",
        DeclaredStateTypeKind::Null => "null",
    }
}

fn render_model_from_parsed_method(
    method: &ezc_parser::ParsedMethod,
    component_id: &SemanticId,
) -> RenderModel {
    let root = method.jsx_roots.first();
    let root_element = root.and_then(parsed_root_element);
    let root_fragment = root.and_then(parsed_root_fragment);
    let mut event_ids = EventIdAllocator::default();

    RenderModel {
        root_element: root_element.map(|element| element.name.clone()),
        root_span: root_element.map(|element| element.span),
        root_fragment: root_fragment
            .map(|fragment| render_fragment_from_parsed(fragment, component_id, &mut event_ids)),
        attributes: root_element.map_or_else(Vec::new, |element| {
            element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect()
        }),
        event_handlers: root_element.map_or_else(Vec::new, |element| {
            element
                .event_handlers
                .iter()
                .map(|handler| {
                    render_event_handler_from_parsed(handler, component_id, &mut event_ids)
                })
                .collect()
        }),
        children: root_element.map_or_else(Vec::new, |element| {
            element
                .children
                .iter()
                .map(|child| render_child_from_parsed(child, component_id, &mut event_ids))
                .collect()
        }),
        bindings: method.bindings.clone(),
    }
}

fn parsed_root_element(root: &ParsedJsxNode) -> Option<&ezc_parser::ParsedJsxElement> {
    match root {
        ParsedJsxNode::Element(element) => Some(element),
        ParsedJsxNode::Fragment(_) => None,
    }
}

fn parsed_root_fragment(root: &ParsedJsxNode) -> Option<&ParsedJsxFragment> {
    match root {
        ParsedJsxNode::Element(_) => None,
        ParsedJsxNode::Fragment(fragment) => Some(fragment),
    }
}

fn collect_render_binding_diagnostics(
    class: &ParsedClass,
    render: &RenderModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let property_names = class
        .properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<Vec<_>>();

    for binding in &render.bindings {
        if let Some(name) = this_member_name(binding) {
            if !property_names.contains(&name) {
                diagnostics.push(ComponentDiagnostic {
                    provenance: None,
                    code: "EZC1003".to_string(),
                    message: format!(
                        "render binding `{binding}` references unknown field `{name}` in class `{}`",
                        class.name
                    ),
                });
            }
        }
    }
}

fn collect_render_event_diagnostics(
    class: &ParsedClass,
    render: &RenderModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let method_names = class
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect::<Vec<_>>();

    for event_handler in render_event_handlers(render) {
        if event_handler.event != "click" {
            diagnostics.push(ComponentDiagnostic {
                provenance: None,
                code: "EZC1005".to_string(),
                message: format!(
                    "event `{}` is not supported yet in class `{}`",
                    event_handler.event, class.name
                ),
            });
        }

        if let Some(name) = this_member_name(&event_handler.handler) {
            if !method_names.contains(&name) {
                diagnostics.push(ComponentDiagnostic {
                    provenance: None,
                    code: "EZC1004".to_string(),
                    message: format!(
                        "event handler `{}` references unknown method `{name}` in class `{}`",
                        event_handler.handler, class.name
                    ),
                });
            }
        }
    }
}

fn state_operation_from_parsed(operation: &ParsedStateOperation) -> StateOperation {
    match operation {
        ParsedStateOperation::Increment => StateOperation::Increment,
        ParsedStateOperation::Decrement => StateOperation::Decrement,
        ParsedStateOperation::AddAssign(value) => {
            StateOperation::AddAssign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::SubtractAssign(value) => {
            StateOperation::SubtractAssign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::Assign(value) => {
            StateOperation::Assign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::Toggle => StateOperation::Toggle,
    }
}

fn serializable_value_from_parsed(value: &ParsedSerializableValue) -> SerializableValue {
    match value {
        ParsedSerializableValue::Null => SerializableValue::Null,
        ParsedSerializableValue::Number(value) => SerializableValue::Number(value.clone()),
        ParsedSerializableValue::String(value) => SerializableValue::String(value.clone()),
        ParsedSerializableValue::Boolean(value) => SerializableValue::Boolean(*value),
        ParsedSerializableValue::Array(values) => {
            SerializableValue::Array(values.iter().map(serializable_value_from_parsed).collect())
        }
        ParsedSerializableValue::Object(values) => SerializableValue::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), serializable_value_from_parsed(value)))
                .collect(),
        ),
    }
}

fn decorator_argument(class: &ParsedClass, name: &str) -> Option<String> {
    class
        .decorators
        .iter()
        .find(|decorator| decorator.name == name)
        .and_then(|decorator| decorator.argument.clone())
}

fn render_child_from_parsed(
    child: &ParsedJsxChild,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderChild {
    match child {
        ParsedJsxChild::Text { value, span } => RenderChild::Text {
            value: value.clone(),
            span: *span,
        },
        ParsedJsxChild::Binding { expression, span } => RenderChild::Binding {
            expression: expression.clone(),
            span: *span,
        },
        ParsedJsxChild::Element(element) => RenderChild::Element(RenderElement {
            tag_name: element.name.clone(),
            span: element.span,
            attributes: element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect(),
            event_handlers: element
                .event_handlers
                .iter()
                .map(|handler| render_event_handler_from_parsed(handler, component_id, event_ids))
                .collect(),
            children: element
                .children
                .iter()
                .map(|child| render_child_from_parsed(child, component_id, event_ids))
                .collect::<Vec<_>>(),
        }),
        ParsedJsxChild::Fragment(fragment) => RenderChild::Fragment(render_fragment_from_parsed(
            fragment,
            component_id,
            event_ids,
        )),
        ParsedJsxChild::Conditional(conditional) => RenderChild::Conditional(
            render_conditional_from_parsed(conditional, component_id, event_ids),
        ),
        ParsedJsxChild::List(list) => {
            RenderChild::List(render_list_from_parsed(list, component_id, event_ids))
        }
    }
}

fn render_list_from_parsed(
    list: &ParsedJsxList,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderList {
    RenderList {
        iterable: list.iterable.clone(),
        item_variable: list.item_variable.clone(),
        index_variable: list.index_variable.clone(),
        key_expression: list.key_expression.clone(),
        span: list.span,
        item_template: render_children_from_parsed_node(
            &list.item_template,
            component_id,
            event_ids,
        ),
    }
}

fn render_conditional_from_parsed(
    conditional: &ParsedJsxConditional,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderConditional {
    RenderConditional {
        condition: conditional.condition.clone(),
        span: conditional.span,
        when_true: render_children_from_parsed_node(
            &conditional.when_true,
            component_id,
            event_ids,
        ),
        when_false: conditional
            .when_false
            .as_ref()
            .map(|node| render_children_from_parsed_node(node, component_id, event_ids))
            .unwrap_or_default(),
    }
}

fn render_children_from_parsed_node(
    node: &ParsedJsxNode,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> Vec<RenderChild> {
    match node {
        ParsedJsxNode::Element(element) => vec![RenderChild::Element(RenderElement {
            tag_name: element.name.clone(),
            span: element.span,
            attributes: element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect(),
            event_handlers: element
                .event_handlers
                .iter()
                .map(|handler| render_event_handler_from_parsed(handler, component_id, event_ids))
                .collect(),
            children: element
                .children
                .iter()
                .map(|child| render_child_from_parsed(child, component_id, event_ids))
                .collect::<Vec<_>>(),
        })],
        ParsedJsxNode::Fragment(fragment) => fragment
            .children
            .iter()
            .map(|child| render_child_from_parsed(child, component_id, event_ids))
            .collect(),
    }
}

fn render_fragment_from_parsed(
    fragment: &ParsedJsxFragment,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderFragment {
    RenderFragment {
        span: fragment.span,
        children: fragment
            .children
            .iter()
            .map(|child| render_child_from_parsed(child, component_id, event_ids))
            .collect(),
    }
}

fn render_attribute_from_parsed(attribute: &ParsedJsxAttribute) -> RenderAttribute {
    RenderAttribute {
        name: attribute.name.clone(),
        value: match &attribute.value {
            ParsedJsxAttributeValue::Boolean => RenderAttributeValue::Boolean,
            ParsedJsxAttributeValue::Static(value) => RenderAttributeValue::Static(value.clone()),
            ParsedJsxAttributeValue::Expression(expression) => {
                RenderAttributeValue::Expression(expression.clone())
            }
            ParsedJsxAttributeValue::Spread(expression) => {
                RenderAttributeValue::Spread(expression.clone())
            }
            ParsedJsxAttributeValue::Unsupported => RenderAttributeValue::Unsupported,
        },
        span: attribute.span,
    }
}

fn render_event_handler_from_parsed(
    event_handler: &ParsedEventHandler,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderEventHandler {
    RenderEventHandler {
        id: component_id.event_handler(&event_handler.event, event_ids.next()),
        owner: SemanticOwner::entity(component_id.template()),
        event: event_handler.event.clone(),
        handler: event_handler.handler.clone(),
        span: event_handler.span,
    }
}

#[derive(Debug, Default)]
struct EventIdAllocator {
    next: usize,
}

impl EventIdAllocator {
    fn next(&mut self) -> usize {
        let current = self.next;
        self.next += 1;
        current
    }
}

fn collect_component_provenance(
    class: &ParsedClass,
    component: &ComponentNode,
    path: &Path,
) -> BTreeMap<SemanticId, SourceProvenance> {
    let mut provenance = BTreeMap::new();
    provenance.insert(
        component.id.clone(),
        SourceProvenance::new(path, class.span),
    );

    for property in &class.properties {
        if property.initializer.as_deref() != Some("state(...)") {
            continue;
        }

        if let Some(field) = component
            .state_fields
            .iter()
            .find(|field| field.name == property.name)
        {
            provenance.insert(field.id.clone(), SourceProvenance::new(path, property.span));
        }
    }

    for method in &class.methods {
        if let Some(component_method) = component
            .methods
            .iter()
            .find(|component_method| component_method.name == method.name)
        {
            provenance.insert(
                component_method.id.clone(),
                SourceProvenance::new(path, method.span),
            );
        }

        if method.name == "render" {
            provenance.insert(
                component.id.template(),
                SourceProvenance::new(path, method.span),
            );
        }

        for (index, update) in method.state_updates.iter().enumerate() {
            provenance.insert(
                component.id.action(&method.name, index),
                SourceProvenance::new(path, update.span),
            );
        }

        if let Some(component_method) = component
            .methods
            .iter()
            .find(|component_method| component_method.name == method.name)
        {
            for local in &component_method.local_variables {
                provenance.insert(local.id.clone(), SourceProvenance::new(path, local.span));
            }
        }
    }

    if let Some(render) = &component.render {
        for handler in render_event_handlers(render) {
            provenance.insert(
                handler.id.clone(),
                SourceProvenance::new(path, handler.span),
            );
        }
    }

    provenance
}

fn collect_semantic_references(
    component: &ComponentNode,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> Vec<SemanticReference> {
    let mut references = component
        .actions
        .iter()
        .filter_map(|action| {
            component
                .state_fields
                .iter()
                .find(|field| field.name == action.field)
                .map(|field| SemanticReference {
                    kind: SemanticReferenceKind::ActionState,
                    source: action.id.clone(),
                    target: field.id.clone(),
                    provenance: provenance
                        .get(&action.id)
                        .expect("action semantic provenance should exist")
                        .clone(),
                })
        })
        .collect::<Vec<_>>();

    if let Some(render) = &component.render {
        references.extend(
            render_event_handlers(render)
                .into_iter()
                .filter_map(|handler| {
                    let method_name = this_member_name(&handler.handler)?;
                    component
                        .methods
                        .iter()
                        .find(|method| method.name == method_name)
                        .map(|method| SemanticReference {
                            kind: SemanticReferenceKind::EventMethod,
                            source: handler.id.clone(),
                            target: method.id.clone(),
                            provenance: provenance
                                .get(&handler.id)
                                .expect("event semantic provenance should exist")
                                .clone(),
                        })
                }),
        );
    }

    references
}

pub(crate) fn render_event_handlers(render: &RenderModel) -> Vec<&RenderEventHandler> {
    let mut event_handlers = render.event_handlers.iter().collect::<Vec<_>>();

    for child in &render.children {
        collect_child_event_handlers(child, &mut event_handlers);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_event_handlers(child, &mut event_handlers);
        }
    }

    event_handlers
}

fn collect_child_event_handlers<'a>(
    child: &'a RenderChild,
    event_handlers: &mut Vec<&'a RenderEventHandler>,
) {
    match child {
        RenderChild::Element(element) => {
            event_handlers.extend(element.event_handlers.iter());

            for child in &element.children {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_event_handlers(child, event_handlers);
            }
            for child in &conditional.when_false {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_duplicate_event_diagnostics(
    render: &RenderModel,
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    collect_duplicate_events_for_handlers(&render.event_handlers, class_name, diagnostics);

    for child in &render.children {
        collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
        }
    }
}

fn collect_render_attribute_diagnostics(
    render: &RenderModel,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    collect_attribute_diagnostics_for_attributes(
        &render.attributes,
        state_fields,
        class_name,
        diagnostics,
        None,
    );

    for child in &render.children {
        collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_render_list_diagnostics(
    render: &RenderModel,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for child in &render.children {
        collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_child_list_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            for child in &element.children {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            collect_list_diagnostics(list, state_fields, class_name, diagnostics);

            for child in &list.item_template {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_list_diagnostics(
    list: &RenderList,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    if list.key_expression.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            provenance: None,
            code: "EZC1011".to_string(),
            message: format!(
                "list over `{}` in class `{class_name}` is missing a `key={{...}}` attribute; stable keys are required for retained-node reconciliation",
                list.iterable
            ),
        });
        return;
    }

    if list.index_variable.as_deref() == Some(list.key_expression.as_str()) {
        diagnostics.push(ComponentDiagnostic {
            provenance: None,
            code: "EZC1012".to_string(),
            message: format!(
                "list key `{}` in class `{class_name}` uses the iteration index; index keys are unstable when items move",
                list.key_expression
            ),
        });
        return;
    }

    let member_path = list_member_key_path(list);
    if list.key_expression != list.item_variable && member_path.is_none() {
        diagnostics.push(ComponentDiagnostic {
            provenance: None,
            code: "EZC1013".to_string(),
            message: format!(
                "list key `{}` in class `{class_name}` is not supported yet; use the item variable `{}` or one of its object members",
                list.key_expression, list.item_variable
            ),
        });
        return;
    }

    let Some(field_name) = this_member_name(&list.iterable) else {
        return;
    };
    let Some(SerializableValue::Array(values)) = state_fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| field.initial_value.as_ref())
    else {
        return;
    };

    let mut keys = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let key_value = member_path.map_or(Some(value), |path| value.member_path_value(path));
        let Some(key) = key_value.and_then(list_key_from_static_value) else {
            diagnostics.push(ComponentDiagnostic {
                provenance: None,
                code: "EZC1015".to_string(),
                message: member_path.map_or_else(
                    || format!(
                        "list key `{}` resolves to a non-primitive initial item at index {index} in class `{class_name}`; keyed reconciliation requires primitive keys",
                        list.key_expression
                    ),
                    |_| format!(
                        "list key `{}` cannot resolve a primitive member value for initial item at index {index} in class `{class_name}`; every item must provide that member",
                        list.key_expression
                    ),
                ),
            });
            return;
        };

        if keys.contains(&key) {
            diagnostics.push(ComponentDiagnostic {
                provenance: None,
                code: "EZC1014".to_string(),
                message: format!(
                    "list key `{}` resolves to duplicate initial value `{key}` in class `{class_name}`; keyed reconciliation requires unique keys",
                    list.key_expression
                ),
            });
            return;
        }

        keys.push(key);
    }
}

fn list_member_key_path(list: &RenderList) -> Option<&str> {
    list.key_expression
        .strip_prefix(&list.item_variable)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .filter(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn list_key_from_static_value(value: &SerializableValue) -> Option<String> {
    match value {
        SerializableValue::Null => Some("null".to_string()),
        SerializableValue::Number(value) | SerializableValue::String(value) => Some(value.clone()),
        SerializableValue::Boolean(value) => Some(value.to_string()),
        SerializableValue::Array(_) | SerializableValue::Object(_) => None,
    }
}

fn collect_child_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_attribute_diagnostics_for_attributes(
                &element.attributes,
                state_fields,
                class_name,
                diagnostics,
                None,
            );

            for child in &element.children {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    &list.item_variable,
                    list.index_variable.as_deref(),
                );
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_attribute_diagnostics_for_attributes(
    attributes: &[RenderAttribute],
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
    list_scope: Option<(&str, Option<&str>)>,
) {
    let mut seen = Vec::<&str>::new();

    for attribute in attributes {
        if !attribute.name.starts_with("on") {
            if seen.contains(&attribute.name.as_str()) {
                diagnostics.push(ComponentDiagnostic {
                    provenance: None,
                    code: "EZC1007".to_string(),
                    message: format!(
                        "attribute `{}` is declared more than once on the same element in class `{}`",
                        attribute.name, class_name
                    ),
                });
            } else if attribute.name != "{...}" {
                seen.push(&attribute.name);
            }
        }

        match &attribute.value {
            RenderAttributeValue::Expression(_) if attribute.name == "key" => {}
            RenderAttributeValue::Expression(expression)
                if !is_event_attribute(&attribute.name) =>
            {
                if expression.as_deref().is_some_and(|expression| {
                    list_scope
                        .is_some_and(|scope| list_item_attribute_expression(expression, scope))
                }) {
                    continue;
                }

                match expression.as_deref().and_then(this_member_name) {
                    Some(field_name)
                        if state_fields.iter().any(|field| field.name == field_name) => {}
                    Some(field_name) => diagnostics.push(ComponentDiagnostic {
                        provenance: None,
                        code: "EZC1003".to_string(),
                        message: format!(
                            "attribute binding `{}` references unknown state field `{field_name}` in class `{}`",
                            attribute.name, class_name
                        ),
                    }),
                    None => diagnostics.push(ComponentDiagnostic {
                        provenance: None,
                        code: "EZC1008".to_string(),
                        message: format!(
                            "attribute `{}` uses an unsupported expression value in class `{}`",
                            attribute.name, class_name
                        ),
                    }),
                }
            }
            RenderAttributeValue::Spread(_) => {
                diagnostics.push(ComponentDiagnostic {
                    provenance: None,
                    code: "EZC1009".to_string(),
                    message: format!(
                        "JSX spread attributes are not supported yet in class `{class_name}`"
                    ),
                });
            }
            RenderAttributeValue::Unsupported if !is_event_attribute(&attribute.name) => {
                diagnostics.push(ComponentDiagnostic {
                    provenance: None,
                    code: "EZC1010".to_string(),
                    message: format!(
                        "attribute `{}` uses an unsupported JSX value in class `{}`",
                        attribute.name, class_name
                    ),
                });
            }
            _ => {}
        }
    }
}

fn collect_list_item_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
    item_variable: &str,
    index_variable: Option<&str>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_attribute_diagnostics_for_attributes(
                &element.attributes,
                state_fields,
                class_name,
                diagnostics,
                Some((item_variable, index_variable)),
            );

            for child in &element.children {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
            for child in &conditional.when_false {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    &list.item_variable,
                    list.index_variable.as_deref(),
                );
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn list_item_attribute_expression(expression: &str, scope: (&str, Option<&str>)) -> bool {
    expression == scope.0
        || scope.1 == Some(expression)
        || expression
            .strip_prefix(scope.0)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .is_some_and(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn is_event_attribute(name: &str) -> bool {
    name.strip_prefix("on")
        .and_then(|event| event.chars().next())
        .is_some_and(char::is_uppercase)
}

fn collect_duplicate_child_event_diagnostics(
    child: &RenderChild,
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_duplicate_events_for_handlers(&element.event_handlers, class_name, diagnostics);

            for child in &element.children {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_duplicate_events_for_handlers(
    event_handlers: &[RenderEventHandler],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let mut seen = Vec::<&str>::new();

    for event_handler in event_handlers {
        if seen.contains(&event_handler.event.as_str()) {
            diagnostics.push(ComponentDiagnostic {
                provenance: None,
                code: "EZC1006".to_string(),
                message: format!(
                    "event `{}` is declared more than once on the same element in class `{}`",
                    event_handler.event, class_name
                ),
            });
        } else {
            seen.push(&event_handler.event);
        }
    }
}

fn this_member_name(reference: &str) -> Option<&str> {
    reference.strip_prefix("this.")
}
