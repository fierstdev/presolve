//! Core compiler data structures for the first EdgeZero learning slice.
//!
//! This crate deliberately does **not** parse TSX yet. It records a source summary,
//! spans, obvious declarations, and diagnostics. That gives the project a stable
//! place to learn compiler fundamentals before choosing a real parser backend.

pub mod component_graph;
pub mod explain;
pub mod model;
pub mod summarize;

pub use component_graph::{
    build_component_graph, ComponentDiagnostic, ComponentGraph, ComponentMethod, ComponentNode,
    RenderModel, StateField,
};
pub use explain::{explain_json, explain_text};
pub use model::{
    ClassSummary, DecoratorSummary, Diagnostic, RenderMethodSummary, Severity, SourceSummary, Span,
};
pub use summarize::summarize_source;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        assert_eq!(
            summary.component_decorators[0].argument.as_deref(),
            Some("x-counter")
        );
        assert_eq!(summary.class_declarations.len(), 1);
        assert_eq!(summary.class_declarations[0].name, "Counter");
        assert_eq!(summary.render_methods.len(), 1);
        assert!(summary.has_tsx_like_syntax);
    }

    #[test]
    fn emits_diagnostics_for_empty_source() {
        let summary = summarize_source("Empty.tsx", "");
        assert!(summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZ0001"));
    }

    #[test]
    fn fixture_0001_source_summary_explain_text_matches_expected() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/0001-source-summary");

        let input_path = fixture_root.join("input/Counter.tsx");
        let expected_path = fixture_root.join("expected/explain.txt");

        let source = std::fs::read_to_string(&input_path).expect("failed to read fixture input");
        let expected = std::fs::read_to_string(&expected_path)
            .expect("failed to read expected explain output");

        let summary = summarize_source("fixtures/0001-source-summary/input/Counter.tsx", &source);

        let actual = explain_text(&summary);

        assert_eq!(actual, expected);
    }

    #[test]
    fn fixture_0001_source_summary_explain_json_matches_expected() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/0001-source-summary");

        let input_path = fixture_root.join("input/Counter.tsx");
        let expected_path = fixture_root.join("expected/explain.json");

        let source = std::fs::read_to_string(&input_path).expect("failed to read fixture input");
        let expected = std::fs::read_to_string(&expected_path)
            .expect("failed to read expected JSON explain output");

        let summary = summarize_source("fixtures/0001-source-summary/input/Counter.tsx", &source);

        let actual = explain_json(&summary);

        let actual_json: serde_json::Value =
            serde_json::from_str(&actual).expect("actual explain JSON is invalid");
        let expected_json: serde_json::Value =
            serde_json::from_str(&expected).expect("expected explain JSON fixture is invalid");

        assert_eq!(actual_json, expected_json);
    }

    #[test]
    fn builds_component_graph_from_parsed_counter() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let graph = build_component_graph(&parsed);

        assert!(graph.diagnostics.is_empty());

        let component = graph.components.first().expect("expected component");

        assert_eq!(component.class_name, "Counter");
        assert_eq!(component.element_name.as_deref(), Some("x-counter"));
        assert_eq!(component.route_path.as_deref(), Some("/counter"));

        assert_eq!(component.state_fields.len(), 1);
        assert_eq!(component.state_fields[0].name, "count");

        let method_names = component
            .methods
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(method_names, vec!["increment", "render"]);

        let render = component.render.as_ref().expect("expected render model");

        assert_eq!(render.root_element.as_deref(), Some("button"));
        assert_eq!(render.attributes, vec!["onClick={...}"]);
        assert_eq!(render.bindings, vec!["this.count"]);
    }
}
