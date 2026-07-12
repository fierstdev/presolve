use std::collections::BTreeMap;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub classes: Vec<ParsedClass>,
    pub type_aliases: Vec<ParsedTypeAlias>,
    pub imports: Vec<ParsedImport>,
    pub exports: Vec<ParsedExport>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Authored type alias retained for canonical semantic type lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTypeAlias {
    pub name: String,
    pub type_text: String,
    pub span: SourceSpan,
    pub type_span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedImport {
    pub source: String,
    pub specifiers: Vec<ParsedImportSpecifier>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedImportSpecifier {
    pub imported: String,
    pub local: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExport {
    pub kind: ParsedExportKind,
    pub source: Option<String>,
    pub specifiers: Vec<ParsedExportSpecifier>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedExportKind {
    Named,
    Default,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExportSpecifier {
    pub local: Option<String>,
    pub exported: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedClass {
    pub name: String,
    pub span: SourceSpan,
    pub decorators: Vec<ParsedDecorator>,
    pub properties: Vec<ParsedProperty>,
    pub methods: Vec<ParsedMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDecorator {
    pub name: String,
    pub argument: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProperty {
    pub name: String,
    pub initializer: Option<String>,
    pub state_initial_value: Option<ParsedSerializableValue>,
    pub state_initial_expression: Option<ParsedConstantExpression>,
    pub state_type_annotation: Option<ParsedTypeAnnotation>,
    pub span: SourceSpan,
}

/// Authored TypeScript annotation retained for a state field without type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTypeAnnotation {
    pub text: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedSerializableValue {
    Null,
    Number(String),
    String(String),
    Boolean(bool),
    Array(Vec<ParsedSerializableValue>),
    Object(BTreeMap<String, ParsedSerializableValue>),
}

/// A compiler-owned numeric arithmetic expression accepted in `state(...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArithmeticExpression {
    pub kind: ParsedArithmeticExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedArithmeticExpressionKind {
    Number(String),
    Binary {
        operator: ParsedArithmeticOperator,
        left: Box<ParsedArithmeticExpression>,
        right: Box<ParsedArithmeticExpression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

/// A compiler-owned constant expression accepted in `state(...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConstantExpression {
    pub kind: ParsedConstantExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedConstantExpressionKind {
    Primitive(ParsedSerializableValue),
    Boolean(bool),
    Arithmetic(ParsedArithmeticExpression),
    Comparison {
        operator: ParsedComparisonOperator,
        left: ParsedArithmeticExpression,
        right: ParsedArithmeticExpression,
    },
    Logical {
        operator: ParsedLogicalOperator,
        left: Box<ParsedConstantExpression>,
        right: Box<ParsedConstantExpression>,
    },
    NullishCoalescing {
        left: Box<ParsedConstantExpression>,
        right: Box<ParsedConstantExpression>,
    },
    Unary {
        operator: ParsedUnaryOperator,
        operand: Box<ParsedConstantExpression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedLogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedUnaryOperator {
    Not,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethod {
    pub name: String,
    pub span: SourceSpan,
    pub decorators: Vec<ParsedDecorator>,
    pub is_getter: bool,
    pub jsx_roots: Vec<ParsedJsxNode>,
    pub bindings: Vec<String>,
    pub state_updates: Vec<ParsedStateUpdate>,
    pub local_variables: Vec<ParsedLocalVariable>,
    pub parameters: Vec<ParsedMethodParameter>,
    pub return_type_annotation: Option<ParsedTypeAnnotation>,
    pub return_values: Vec<ParsedSerializableValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethodParameter {
    pub name: String,
    pub span: SourceSpan,
    pub type_annotation: Option<ParsedTypeAnnotation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLocalVariable {
    pub name: String,
    pub value: ParsedSerializableValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStateUpdate {
    pub field: String,
    pub operation: ParsedStateOperation,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedStateOperation {
    Increment,
    Decrement,
    AddAssign(ParsedSerializableValue),
    SubtractAssign(ParsedSerializableValue),
    Assign(ParsedSerializableValue),
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedJsxChild {
    Text {
        value: String,
        span: SourceSpan,
    },
    Binding {
        expression: String,
        span: SourceSpan,
    },
    Element(ParsedJsxElement),
    Fragment(ParsedJsxFragment),
    Conditional(ParsedJsxConditional),
    List(ParsedJsxList),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedJsxNode {
    Element(ParsedJsxElement),
    Fragment(ParsedJsxFragment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJsxElement {
    pub name: String,
    pub span: SourceSpan,
    pub attributes: Vec<ParsedJsxAttribute>,
    pub event_handlers: Vec<ParsedEventHandler>,
    pub children: Vec<ParsedJsxChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJsxFragment {
    pub span: SourceSpan,
    pub children: Vec<ParsedJsxChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJsxConditional {
    pub condition: String,
    pub span: SourceSpan,
    pub when_true: ParsedJsxNode,
    pub when_false: Option<ParsedJsxNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJsxList {
    pub iterable: String,
    pub item_variable: String,
    pub index_variable: Option<String>,
    pub key_expression: String,
    pub span: SourceSpan,
    pub item_template: ParsedJsxNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJsxAttribute {
    pub name: String,
    pub value: ParsedJsxAttributeValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedJsxAttributeValue {
    Boolean,
    Static(String),
    Expression(Option<String>),
    Spread(Option<String>),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEventHandler {
    pub event: String,
    pub handler: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub message: String,
    pub severity: ParseSeverity,
    pub labels: Vec<ParseLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLabel {
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}
