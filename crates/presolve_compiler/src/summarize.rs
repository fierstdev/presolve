use std::path::Path;

use crate::model::{
    ClassSummary, DecoratorSummary, Diagnostic, RenderMethodSummary, Severity, SourceSummary, Span,
};

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
            code: "PS0001".to_string(),
            message: "source file is empty".to_string(),
            span: None,
        });
    }

    let component_decorators = find_string_decorators(source, "component");
    let route_decorators = find_string_decorators(source, "route");
    let class_declarations = find_class_declarations(source);
    let render_methods = find_render_methods(source);
    let has_tsx_like_syntax =
        source.contains('<') && source.contains('>') && source.contains("render");

    if component_decorators.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "PS0100".to_string(),
            message: "no @component(...) decorator found".to_string(),
            span: None,
        });
    }

    if class_declarations.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "PS0101".to_string(),
            message: "no class declaration found".to_string(),
            span: None,
        });
    }

    if render_methods.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "PS0102".to_string(),
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
            .map_or(argument_start, |relative_end| {
                argument_start + relative_end + 1
            });

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

    Span {
        start,
        end,
        line,
        column,
    }
}
