use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub classes: Vec<ParsedClass>,
    pub diagnostics: Vec<ParseDiagnostic>,
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
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedSerializableValue {
    Null,
    Number(String),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethod {
    pub name: String,
    pub span: SourceSpan,
    pub jsx_roots: Vec<ParsedJsxNode>,
    pub bindings: Vec<String>,
    pub state_updates: Vec<ParsedStateUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStateUpdate {
    pub field: String,
    pub operation: ParsedStateOperation,
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
