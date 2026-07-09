//! Core compiler data structures for the first EdgeZero learning slice.
//!
//! This crate deliberately does **not** parse TSX yet. It records a source summary,
//! spans, obvious declarations, and diagnostics. That gives the project a stable
//! place to learn compiler fundamentals before choosing a real parser backend.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

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

/// Produce a first-pass source summary.
///
/// This function is intentionally lightweight. It is not a parser. Its job is to
/// establish the compiler habit of preserving source positions and explaining
/// what the tool thinks it found.
pub fn summarize_source(path: impl AsRef<Path>, source: &str) -> SourceSummary {
    let path = path.as_ref().to_path_buf();
    let mut diagnostics = Vec::new();

    if source.trim().is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "EZ0001".to_string(),
            message: "source file is empty".to_string(),
            span: None,
        });
    }

    let component_decorators = find_string_decorators(source, "component");
    let route_decorators = find_string_decorators(source, "route");
    let class_declarations = find_class_declarations(source);
    let render_methods = find_render_methods(source);
    let has_tsx_like_syntax = source.contains("<") && source.contains(">") && source.contains("render");

    if component_decorators.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "EZ0100".to_string(),
            message: "no @component(...) decorator found".to_string(),
            span: None,
        });
    }

    if class_declarations.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "EZ0101".to_string(),
            message: "no class declaration found".to_string(),
            span: None,
        });
    }

    if render_methods.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "EZ0102".to_string(),
            message: "no render() method found".to_string(),
            span: None,
        });
    }

    SourceSummary {
        path,
        byte_len: source.len(),
        line_count: source.lines().count(),
        char_count: source.chars().count(),
        has_tsx_like_syntax,
        component_decorators,
        route_decorators,
        class_declarations,
        render_methods,
        diagnostics,
    }
}

pub fn explain_text(summary: &SourceSummary) -> String {
    let mut output = String::new();

    let _ = writeln!(output, "File: {}", summary.path.display());
    let _ = writeln!(output, "Bytes: {}", summary.byte_len);
    let _ = writeln!(output, "Lines: {}", summary.line_count);
    let _ = writeln!(output, "Characters: {}", summary.char_count);
    let _ = writeln!(output, "TSX-like syntax: {}", yes_no(summary.has_tsx_like_syntax));

    let _ = writeln!(output, "\nComponents:");
    if summary.component_decorators.is_empty() {
        let _ = writeln!(output, "  none");
    } else {
        for item in &summary.component_decorators {
            let argument = item.argument.as_deref().unwrap_or("<missing>");
            let _ = writeln!(
                output,
                "  @{}({argument:?}) at {}:{}",
                item.name, item.span.line, item.span.column
            );
        }
    }

    let _ = writeln!(output, "\nRoutes:");
    if summary.route_decorators.is_empty() {
        let _ = writeln!(output, "  none");
    } else {
        for item in &summary.route_decorators {
            let argument = item.argument.as_deref().unwrap_or("<missing>");
            let _ = writeln!(
                output,
                "  @{}({argument:?}) at {}:{}",
                item.name, item.span.line, item.span.column
            );
        }
    }

    let _ = writeln!(output, "\nClasses:");
    if summary.class_declarations.is_empty() {
        let _ = writeln!(output, "  none");
    } else {
        for class in &summary.class_declarations {
            let _ = writeln!(
                output,
                "  class {} at {}:{}",
                class.name, class.span.line, class.span.column
            );
        }
    }

    let _ = writeln!(output, "\nRender methods:");
    if summary.render_methods.is_empty() {
        let _ = writeln!(output, "  none");
    } else {
        for method in &summary.render_methods {
            let _ = writeln!(output, "  render() at {}:{}", method.span.line, method.span.column);
        }
    }

    let _ = writeln!(output, "\nDiagnostics:");
    if summary.diagnostics.is_empty() {
        let _ = writeln!(output, "  none");
    } else {
        for diagnostic in &summary.diagnostics {
            let _ = writeln!(
                output,
                "  {:?} {}: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            );
        }
    }

    output
}

pub fn explain_json(summary: &SourceSummary) -> String {
    // Manual JSON keeps the first slice dependency-free. Replace this with serde
    // once the schema is stable enough to deserve a dependency.
    let mut output = String::new();
    let _ = writeln!(output, "{{");
    let _ = writeln!(output, "  \"path\": {},", json_string(&summary.path.display().to_string()));
    let _ = writeln!(output, "  \"byteLen\": {},", summary.byte_len);
    let _ = writeln!(output, "  \"lineCount\": {},", summary.line_count);
    let _ = writeln!(output, "  \"charCount\": {},", summary.char_count);
    let _ = writeln!(output, "  \"hasTsxLikeSyntax\": {},", summary.has_tsx_like_syntax);
    let _ = writeln!(output, "  \"componentDecorators\": [{}],", decorators_json(&summary.component_decorators));
    let _ = writeln!(output, "  \"routeDecorators\": [{}],", decorators_json(&summary.route_decorators));
    let _ = writeln!(output, "  \"classDeclarations\": [{}],", classes_json(&summary.class_declarations));
    let _ = writeln!(output, "  \"renderMethods\": [{}],", render_methods_json(&summary.render_methods));
    let _ = writeln!(output, "  \"diagnostics\": [{}]", diagnostics_json(&summary.diagnostics));
    let _ = writeln!(output, "}}");
    output
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn find_string_decorators(source: &str, name: &str) -> Vec<DecoratorSummary> {
    let marker = format!("@{name}(");
    let mut items = Vec::new();
    let mut search_start = 0;

    while let Some(relative_index) = source[search_start..].find(&marker) {
        let start = search_start + relative_index;
        let argument_start = start + marker.len();
        let argument = extract_first_string_argument(&source[argument_start..]);
        let end = source[argument_start..]
            .find(')')
            .map_or(argument_start, |relative_end| argument_start + relative_end + 1);

        items.push(DecoratorSummary {
            name: name.to_string(),
            argument,
            span: span_at(source, start, end),
        });

        search_start = end.max(start + marker.len());
    }

    items
}

fn extract_first_string_argument(fragment: &str) -> Option<String> {
    let mut chars = fragment.char_indices();
    let (_, quote) = chars.find(|(_, ch)| *ch == '"' || *ch == '\'')?;
    let content_start = fragment.find(quote)? + quote.len_utf8();
    let rest = &fragment[content_start..];
    let content_end = rest.find(quote)?;
    Some(rest[..content_end].to_string())
}

fn find_class_declarations(source: &str) -> Vec<ClassSummary> {
    let mut classes = Vec::new();
    let mut search_start = 0;

    while let Some(relative_index) = source[search_start..].find("class ") {
        let start = search_start + relative_index;
        let name_start = start + "class ".len();
        let name_end = source[name_start..]
            .char_indices()
            .find(|(_, ch)| !is_identifier_char(*ch))
            .map_or(source.len(), |(offset, _)| name_start + offset);

        if name_end > name_start {
            classes.push(ClassSummary {
                name: source[name_start..name_end].to_string(),
                span: span_at(source, start, name_end),
            });
        }

        search_start = name_end.max(start + "class ".len());
    }

    classes
}

fn find_render_methods(source: &str) -> Vec<RenderMethodSummary> {
    let mut methods = Vec::new();
    let mut search_start = 0;

    while let Some(relative_index) = source[search_start..].find("render()") {
        let start = search_start + relative_index;
        let end = start + "render()".len();
        methods.push(RenderMethodSummary {
            span: span_at(source, start, end),
        });
        search_start = end;
    }

    methods
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn span_at(source: &str, start: usize, end: usize) -> Span {
    let prefix = &source[..start.min(source.len())];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len() + 1, |(_, tail)| tail.len() + 1);

    Span { start, end, line, column }
}

fn decorators_json(items: &[DecoratorSummary]) -> String {
    items
        .iter()
        .map(|item| {
            format!(
                "{{\"name\":{},\"argument\":{},\"span\":{}}}",
                json_string(&item.name),
                item.argument.as_ref().map_or("null".to_string(), |value| json_string(value)),
                span_json(item.span)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn classes_json(items: &[ClassSummary]) -> String {
    items
        .iter()
        .map(|item| {
            format!(
                "{{\"name\":{},\"span\":{}}}",
                json_string(&item.name),
                span_json(item.span)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn render_methods_json(items: &[RenderMethodSummary]) -> String {
    items
        .iter()
        .map(|item| format!("{{\"span\":{}}}", span_json(item.span)))
        .collect::<Vec<_>>()
        .join(",")
}

fn diagnostics_json(items: &[Diagnostic]) -> String {
    items
        .iter()
        .map(|item| {
            format!(
                "{{\"severity\":{},\"code\":{},\"message\":{},\"span\":{}}}",
                json_string(match item.severity {
                    Severity::Info => "info",
                    Severity::Warning => "warning",
                    Severity::Error => "error",
                }),
                json_string(&item.code),
                json_string(&item.message),
                item.span.map_or("null".to_string(), span_json)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn span_json(span: Span) -> String {
    format!(
        "{{\"start\":{},\"end\":{},\"line\":{},\"column\":{}}}",
        span.start, span.end, span.line, span.column
    )
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(output, "\\u{:04x}", ch as u32);
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_component_decorator_class_and_render_method() {
        let source = r#"
@component("x-counter")
class Counter extends Component {
  render() {
    return <button>Count</button>;
  }
}
"#;

        let summary = summarize_source("Counter.tsx", source);

        assert_eq!(summary.component_decorators.len(), 1);
        assert_eq!(summary.component_decorators[0].argument.as_deref(), Some("x-counter"));
        assert_eq!(summary.class_declarations.len(), 1);
        assert_eq!(summary.class_declarations[0].name, "Counter");
        assert_eq!(summary.render_methods.len(), 1);
        assert!(summary.has_tsx_like_syntax);
    }

    #[test]
    fn emits_diagnostics_for_empty_source() {
        let summary = summarize_source("Empty.tsx", "");
        assert!(summary.diagnostics.iter().any(|diagnostic| diagnostic.code == "EZ0001"));
    }
}
