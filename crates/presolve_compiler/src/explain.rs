use std::fmt::Write as _;

use crate::model::{
    ClassSummary, DecoratorSummary, Diagnostic, RenderMethodSummary, Severity, SourceSummary, Span,
};

#[must_use]
pub fn explain_text(summary: &SourceSummary) -> String {
    let mut output = String::new();

    let _ = writeln!(output, "File: {}", summary.path.display());
    let _ = writeln!(output, "Bytes: {}", summary.byte_len);
    let _ = writeln!(output, "Lines: {}", summary.line_count);
    let _ = writeln!(output, "Characters: {}", summary.char_count);
    let _ = writeln!(
        output,
        "TSX-like syntax: {}",
        yes_no(summary.has_tsx_like_syntax)
    );

    let _ = writeln!(output, "\nComponents:");
    if !summary.component_decorators.is_empty() {
        for item in &summary.component_decorators {
            let argument = item.argument.as_deref().unwrap_or("<missing>");
            let _ = writeln!(
                output,
                "  @{}({argument:?}) at {}:{}",
                item.name, item.span.line, item.span.column
            );
        }
    } else if !summary.component_classes.is_empty() {
        for class in &summary.component_classes {
            let _ = writeln!(
                output,
                "  class {} extends Component at {}:{}",
                class.name, class.span.line, class.span.column
            );
        }
    } else {
        let _ = writeln!(output, "  none");
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
            let _ = writeln!(
                output,
                "  render() at {}:{}",
                method.span.line, method.span.column
            );
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

#[must_use]
pub fn explain_json(summary: &SourceSummary) -> String {
    // Manual JSON keeps the first slice dependency-free. Replace this with serde
    // once the schema is stable enough to deserve a dependency.
    let mut output = String::new();
    let _ = writeln!(output, "{{");
    let _ = writeln!(
        output,
        "  \"path\": {},",
        json_string(&summary.path.display().to_string())
    );
    let _ = writeln!(output, "  \"byteLen\": {},", summary.byte_len);
    let _ = writeln!(output, "  \"lineCount\": {},", summary.line_count);
    let _ = writeln!(output, "  \"charCount\": {},", summary.char_count);
    let _ = writeln!(
        output,
        "  \"hasTsxLikeSyntax\": {},",
        summary.has_tsx_like_syntax
    );
    let _ = writeln!(
        output,
        "  \"componentClasses\": [{}],",
        classes_json(&summary.component_classes)
    );
    let _ = writeln!(
        output,
        "  \"componentDecorators\": [{}],",
        decorators_json(&summary.component_decorators)
    );
    let _ = writeln!(
        output,
        "  \"routeDecorators\": [{}],",
        decorators_json(&summary.route_decorators)
    );
    let _ = writeln!(
        output,
        "  \"classDeclarations\": [{}],",
        classes_json(&summary.class_declarations)
    );
    let _ = writeln!(
        output,
        "  \"renderMethods\": [{}],",
        render_methods_json(&summary.render_methods)
    );
    let _ = writeln!(
        output,
        "  \"diagnostics\": [{}]",
        diagnostics_json(&summary.diagnostics)
    );
    let _ = writeln!(output, "}}");
    output
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn decorators_json(items: &[DecoratorSummary]) -> String {
    items
        .iter()
        .map(|item| {
            format!(
                "{{\"name\":{},\"argument\":{},\"span\":{}}}",
                json_string(&item.name),
                item.argument
                    .as_ref()
                    .map_or("null".to_string(), |value| json_string(value)),
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
