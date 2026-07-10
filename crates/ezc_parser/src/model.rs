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
    pub state_initial_value: Option<ParsedStateInitialValue>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedStateInitialValue {
    Number(String),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMethod {
    pub name: String,
    pub span: SourceSpan,
    pub jsx_roots: Vec<ParsedJsxElement>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedJsxChild {
    Text(String),
    Binding(String),
    Element(ParsedJsxElement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJsxElement {
    pub name: String,
    pub span: SourceSpan,
    pub attributes: Vec<String>,
    pub event_handlers: Vec<ParsedEventHandler>,
    pub children: Vec<ParsedJsxChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEventHandler {
    pub event: String,
    pub handler: String,
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
