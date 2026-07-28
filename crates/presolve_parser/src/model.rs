use std::collections::BTreeMap;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile {
    pub path: PathBuf,
    /// The complete source-faithful syntax product. Feature-specific parser
    /// facts below are derived views and must not become a second frontend.
    pub syntax: ParsedSourceAst,
    pub classes: Vec<ParsedClass>,
    pub type_aliases: Vec<ParsedTypeAlias>,
    /// Module-local declarations that bind a name in TypeScript's type
    /// namespace. The compiler uses this normalized fact to distinguish its
    /// built-in marker types from authored lookalikes.
    pub local_type_bindings: Vec<String>,
    /// Module-local declarations in the value namespace. I6 uses this
    /// normalized fact to reject authored functions that shadow compiler-owned
    /// validation intrinsics.
    pub local_value_bindings: Vec<String>,
    pub imports: Vec<ParsedImport>,
    pub exports: Vec<ParsedExport>,
    /// General-AST call sites. These facts intentionally carry no framework
    /// meaning; downstream authorities select and classify them.
    pub call_expressions: Vec<ParsedCallExpression>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSourceAst {
    pub source: String,
    pub estree_json: String,
    pub span: SourceSpan,
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
    /// The exact local binding span selected from the general source AST.
    pub local_span: SourceSpan,
}

/// A source-faithful call expression selected from the general OXC AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCallExpression {
    pub callee_span: SourceSpan,
    /// Structural member spans when the callee is a static member expression.
    /// These are syntax-only positions for an external semantic authority.
    pub member_object_span: Option<SourceSpan>,
    pub member_property_span: Option<SourceSpan>,
    pub span: SourceSpan,
    pub arguments: Vec<ParsedCallArgument>,
}

/// Argument fact retained without interpreting the called function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCallArgument {
    StringLiteral { value: String, span: SourceSpan },
    Other { span: SourceSpan },
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
    pub heritage: Option<ParsedClassHeritage>,
    pub decorators: Vec<ParsedDecorator>,
    pub properties: Vec<ParsedProperty>,
    pub methods: Vec<ParsedMethod>,
}

/// Source-faithful class heritage retained for component-inheritance diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedClassHeritage {
    pub base: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDecorator {
    pub name: String,
    pub is_invoked: bool,
    /// Source-normalized static-string values for every decorator argument.
    /// Existing compiler consumers continue to use `argument` for their
    /// one-argument contracts; multi-argument semantics must opt in explicitly.
    pub arguments: Vec<Option<String>>,
    pub argument: Option<String>,
    pub argument_count: usize,
    pub argument_spans: Vec<SourceSpan>,
    pub static_member_argument: Option<ParsedStaticMemberDesignator>,
    pub this_member_argument: Option<ParsedThisMemberDesignator>,
    /// Normalized syntax for the sole outer argument of `@validate(...)`.
    /// Semantic rule classification remains a core lowering responsibility.
    pub validation_rule_expression: Option<ParsedValidationRuleExpression>,
    pub span: SourceSpan,
}

/// Parser-owned syntax facts for one authored validation rule expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedValidationRuleExpression {
    pub kind: ParsedValidationRuleExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedValidationRuleExpressionKind {
    Call {
        callee: Option<String>,
        arguments: Vec<ParsedValidationRuleArgument>,
    },
    Identifier(String),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedValidationRuleArgument {
    pub kind: ParsedValidationRuleArgumentKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedValidationRuleArgumentKind {
    StringLiteral(String),
    Constant(ParsedConstantExpression),
    ThisMember(ParsedThisMemberDesignator),
    Unsupported,
}

/// An exact direct `this.<identifier>` decorator argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedThisMemberDesignator {
    pub member: String,
    pub span: SourceSpan,
    pub this_span: SourceSpan,
    pub member_span: SourceSpan,
}

/// A source-faithful `ComponentSymbol.contextField` decorator argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStaticMemberDesignator {
    pub object: String,
    pub member: String,
    pub span: SourceSpan,
    pub object_span: SourceSpan,
    pub member_span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProperty {
    pub name: String,
    pub is_identifier_name: bool,
    pub decorators: Vec<ParsedDecorator>,
    /// A direct initializer call selected from the general source AST. Its
    /// callee has no framework meaning until a semantic authority resolves it.
    pub initializer_call: Option<ParsedInitializerCall>,
    /// A static object-argument shape selected from a direct initializer call.
    /// It has no Form meaning until TypeScript authority resolves the outer
    /// call to `defineForm`.
    pub form_definition_shape: Option<ParsedFormDefinitionShape>,
    pub initializer: Option<String>,
    pub initializer_literal: Option<ParsedSerializableValue>,
    pub initializer_expression: Option<ParsedComputedExpression>,
    pub initializer_constant_expression: Option<ParsedConstantExpression>,
    pub initializer_span: Option<SourceSpan>,
    pub state_initial_value: Option<ParsedSerializableValue>,
    pub state_initial_expression: Option<ParsedConstantExpression>,
    pub state_type_annotation: Option<ParsedTypeAnnotation>,
    pub type_annotation: Option<ParsedTypeAnnotation>,
    pub name_span: SourceSpan,
    pub is_static: bool,
    pub is_definite_assignment: bool,
    pub is_declare: bool,
    pub span: SourceSpan,
}

/// Source-faithful static facts from a possible `defineForm({...})` argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFormDefinitionShape {
    pub call_span: SourceSpan,
    pub definition_span: SourceSpan,
    pub serialization: Option<String>,
    pub serialization_span: Option<SourceSpan>,
    pub fields_span: Option<SourceSpan>,
    pub fields: Vec<ParsedFormFieldShape>,
    pub unsupported_fields: Vec<Vec<String>>,
    pub submit: Option<ParsedFormSubmitShape>,
    pub unknown_options: Vec<String>,
}

/// One statically named leaf call below a possible Form `fields` object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFormFieldShape {
    pub name: String,
    pub path: Vec<String>,
    pub declaration_span: SourceSpan,
    pub call_span: SourceSpan,
    pub callee_span: SourceSpan,
    pub argument_count: usize,
    pub initial_value: Option<ParsedSerializableValue>,
    pub initial_span: Option<SourceSpan>,
    pub validations: Vec<ParsedValidationRuleExpression>,
    pub unknown_options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFormSubmitShape {
    pub span: SourceSpan,
    pub is_async: bool,
    pub parameter_count: usize,
}

/// Source-faithful facts for a direct class-field initializer call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInitializerCall {
    pub callee_span: SourceSpan,
    pub span: SourceSpan,
    /// The parser records call arity without assigning framework meaning to
    /// the callee. Semantic lowering uses this to reject malformed intrinsics.
    pub argument_count: usize,
    /// An inline function argument selected from a general initializer call.
    /// This remains syntax only: a later semantic authority decides whether
    /// the surrounding call is a framework action or some unrelated helper.
    pub inline_handler: Option<ParsedInlineHandler>,
}

/// Parser-owned facts for an inline function supplied to a class-field call.
///
/// The body retains only state updates supported by the existing compiler
/// action semantics, plus exact spans for every other non-empty statement.
/// Consumers must reject unsupported bodies rather than guessing a lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInlineHandler {
    pub span: SourceSpan,
    pub body_span: SourceSpan,
    pub is_async: bool,
    pub is_expression_body: bool,
    /// Ordered, source-owned parameters of the inline function. These remain
    /// syntax facts; later semantic lowering decides whether a handler form
    /// may consume them.
    pub parameters: Vec<ParsedMethodParameter>,
    /// Serializable local declarations retained in source order. They have no
    /// framework meaning until a later, authority-backed action projection
    /// validates their use.
    pub local_variables: Vec<ParsedLocalVariable>,
    pub state_updates: Vec<ParsedStateUpdate>,
    pub unsupported_statement_spans: Vec<SourceSpan>,
    /// A restricted ordered-body view retained from a general inline function.
    /// It has no framework meaning until an authority-backed later consumer
    /// selects the surrounding initializer call.
    pub effect_body: Option<ParsedEffectBody>,
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

/// A supported expression retained from one `@computed()` getter body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedComputedExpression {
    pub kind: ParsedComputedExpressionKind,
    pub span: SourceSpan,
}

/// Parsed computed getter expression forms accepted by the E2 lowering slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedComputedExpressionKind {
    Literal(ParsedSerializableValue),
    ThisMember(String),
    MemberAccess {
        object: Box<ParsedComputedExpression>,
        property: String,
        optional: bool,
    },
    /// A statically bounded property or array-element read.
    ///
    /// The parser admits only string and non-negative integer literal indices;
    /// dynamic JavaScript property lookup remains outside the semantic subset.
    IndexAccess {
        object: Box<ParsedComputedExpression>,
        index: Box<ParsedComputedExpression>,
    },
    Conditional {
        condition: Box<ParsedComputedExpression>,
        when_true: Box<ParsedComputedExpression>,
        when_false: Box<ParsedComputedExpression>,
    },
    Template {
        quasis: Vec<String>,
        expressions: Vec<ParsedComputedExpression>,
    },
    Call {
        callee: String,
        arguments: Vec<ParsedComputedExpression>,
    },
    Arithmetic {
        left: Box<ParsedComputedExpression>,
        right: Box<ParsedComputedExpression>,
        operator: ParsedArithmeticOperator,
    },
    Comparison {
        left: Box<ParsedComputedExpression>,
        right: Box<ParsedComputedExpression>,
        operator: ParsedComparisonOperator,
    },
    Logical {
        left: Box<ParsedComputedExpression>,
        right: Box<ParsedComputedExpression>,
        operator: ParsedLogicalOperator,
    },
    NullishCoalescing {
        left: Box<ParsedComputedExpression>,
        right: Box<ParsedComputedExpression>,
    },
    Unary {
        operand: Box<ParsedComputedExpression>,
        operator: ParsedUnaryOperator,
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
    pub is_setter: bool,
    pub is_async: bool,
    pub is_static: bool,
    pub jsx_roots: Vec<ParsedJsxNode>,
    pub bindings: Vec<String>,
    pub state_updates: Vec<ParsedStateUpdate>,
    pub local_variables: Vec<ParsedLocalVariable>,
    pub parameters: Vec<ParsedMethodParameter>,
    pub return_type_annotation: Option<ParsedTypeAnnotation>,
    pub return_values: Vec<ParsedSerializableValue>,
    pub computed_expression: Option<ParsedComputedExpression>,
    pub effect_body: Option<ParsedEffectBody>,
    pub calls: Vec<ParsedMethodCall>,
}

/// Ordered syntax retained from one `@effect()` method body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEffectBody {
    pub statements: Vec<ParsedEffectStatement>,
    pub cleanup: Option<ParsedEffectCleanup>,
}

/// A synchronous cleanup callback returned from a retained inline effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEffectCleanup {
    pub span: SourceSpan,
    pub is_async: bool,
    pub body: Box<ParsedEffectBody>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEffectStatement {
    pub kind: ParsedEffectStatementKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedEffectStatementKind {
    StaticMemberAssignment {
        target: ParsedEffectExpression,
        value: ParsedEffectExpression,
    },
    CapabilityCall {
        callee: ParsedEffectExpression,
        arguments: Vec<ParsedEffectExpression>,
    },
    EffectReturn {
        value: Option<ParsedEffectExpression>,
    },
    Empty,
    Unsupported(ParsedUnsupportedEffectStatementKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedUnsupportedEffectStatementKind {
    LocalDeclaration,
    Branch,
    Loop,
    NestedBlock,
    ExceptionHandling,
    AsyncOperation,
    CompoundAssignment,
    CleanupReturnCandidate,
    UnsupportedExpression,
}

/// Expression syntax accepted as an operand of a lowered effect statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEffectExpression {
    pub kind: ParsedEffectExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedEffectExpressionKind {
    Literal(ParsedSerializableValue),
    Identifier(String),
    ThisMember(String),
    MemberAccess {
        object: Box<ParsedEffectExpression>,
        property: String,
    },
    Arithmetic {
        left: Box<ParsedEffectExpression>,
        right: Box<ParsedEffectExpression>,
        operator: ParsedArithmeticOperator,
    },
    Comparison {
        left: Box<ParsedEffectExpression>,
        right: Box<ParsedEffectExpression>,
        operator: ParsedComparisonOperator,
    },
    Logical {
        left: Box<ParsedEffectExpression>,
        right: Box<ParsedEffectExpression>,
        operator: ParsedLogicalOperator,
    },
    NullishCoalescing {
        left: Box<ParsedEffectExpression>,
        right: Box<ParsedEffectExpression>,
    },
    Unary {
        operand: Box<ParsedEffectExpression>,
        operator: ParsedUnaryOperator,
    },
}

/// One directly authored method call retained for computed-purity analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethodCall {
    pub callee: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethodParameter {
    pub name: String,
    pub decorators: Vec<ParsedDecorator>,
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
    AssignParameter(String),
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
    pub name_span: SourceSpan,
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
    pub name_span: SourceSpan,
    pub value_span: Option<SourceSpan>,
    pub expression_span: Option<SourceSpan>,
    pub this_member: Option<ParsedThisMemberDesignator>,
    pub constant_value: Option<ParsedSerializableValue>,
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
    pub arguments: Vec<ParsedSerializableValue>,
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
