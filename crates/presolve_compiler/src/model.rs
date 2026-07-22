use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSummary {
    pub path: PathBuf,
    pub byte_len: usize,
    pub line_count: usize,
    pub char_count: usize,
    pub has_tsx_like_syntax: bool,
    pub component_decorators: Vec<DecoratorSummary>,
    pub route_decorators: Vec<DecoratorSummary>,
    pub class_declarations: Vec<ClassSummary>,
    pub render_methods: Vec<RenderMethodSummary>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratorSummary {
    pub name: String,
    pub argument: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassSummary {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMethodSummary {
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}
