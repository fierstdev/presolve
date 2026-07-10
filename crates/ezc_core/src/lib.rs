//! Core compiler data structures for the first EdgeZero learning slice.
//!
//! This crate deliberately does **not** parse TSX yet. It records a source summary,
//! spans, obvious declarations, and diagnostics. That gives the project a stable
//! place to learn compiler fundamentals before choosing a real parser backend.

pub mod component_graph;
pub mod explain;
pub mod html_codegen;
pub mod model;
pub mod page_codegen;
pub mod runtime_codegen;
pub mod summarize;
pub mod template_graph;
pub mod template_manifest;

pub use component_graph::{
    build_component_graph, ComponentDiagnostic, ComponentGraph, ComponentMethod, ComponentNode,
    RenderChild, RenderModel, StateField,
};
pub use explain::{explain_json, explain_text};
pub use html_codegen::generate_static_html;
pub use model::{
    ClassSummary, DecoratorSummary, Diagnostic, RenderMethodSummary, Severity, SourceSummary, Span,
};
pub use page_codegen::generate_standalone_page;
pub use runtime_codegen::generate_runtime_stub;
pub use summarize::summarize_source;
pub use template_graph::{
    build_template_graph, AttributeValue, ElementNode, TemplateAttribute, TemplateChild,
    TemplateGraph, TemplateNode, TemplateNodeId,
};
pub use template_manifest::{
    build_template_manifest, template_manifest_json, ManifestComponent, ManifestEvent,
    ManifestNode, ManifestTemplate, TemplateManifest,
};

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
        assert_eq!(
            component.state_fields[0].initial_value.as_deref(),
            Some("0")
        );

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
        assert_eq!(render.event_handler_refs, vec!["this.increment"]);
        assert_eq!(
            render.children,
            vec![
                RenderChild::Text("Count:".to_string()),
                RenderChild::Binding("this.count".to_string()),
            ]
        );
    }

    #[test]
    fn component_graph_reports_semantic_errors() {
        let source =
            include_str!("../../../fixtures/0003-semantic-errors/input/BrokenSemantics.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0003-semantic-errors/input/BrokenSemantics.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);

        let codes = graph
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();

        assert!(codes.contains(&"EZC1001"));
        assert!(codes.contains(&"EZC1003"));
        assert!(codes.contains(&"EZC1004"));
    }

    #[test]
    fn generates_static_html_from_template_graph() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<button data-ez-node=\"n0\" data-ez-event-handler=\"this.increment\" data-ez-bindings=\"this.count\">Count:<!-- ez-binding:n1:this.count -->0</button>\n"
        );
    }

    #[test]
    fn builds_template_graph_from_component_graph() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);

        assert_eq!(template_graph.templates.len(), 1);

        let template = &template_graph.templates[0];
        assert_eq!(template.component_name, "Counter");

        let root = template.root.as_ref().expect("expected template root");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.tag_name, "button");

        assert_eq!(root.attributes.len(), 2);
        assert_eq!(root.attributes[0].name, "data-ez-event-handler");
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::EventHandler("this.increment".to_string())
        );

        assert_eq!(root.attributes[1].name, "data-ez-bindings");
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::BindingList(vec!["this.count".to_string()])
        );

        assert_eq!(
            root.children,
            vec![
                TemplateChild::Text("Count:".to_string()),
                TemplateChild::Binding {
                    id: TemplateNodeId("n1".to_string()),
                    expression: "this.count".to_string(),
                    initial_value: Some("0".to_string()),
                },
            ]
        );
    }

    #[test]
    fn builds_template_manifest_for_nested_jsx() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&template_graph);

        assert_eq!(manifest.components.len(), 1);

        let component = &manifest.components[0];
        assert_eq!(component.name, "NestedCounter");

        assert_eq!(
            component.template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "section".to_string(),
                },
                ManifestNode::Element {
                    id: "n1".to_string(),
                    tag: "button".to_string(),
                },
                ManifestNode::Binding {
                    id: "n2".to_string(),
                    expression: "this.count".to_string(),
                    initial_value: Some("0".to_string()),
                }
            ]
        );

        assert_eq!(
            component.template.events,
            vec![ManifestEvent {
                node: "n1".to_string(),
                handler: "this.increment".to_string(),
            }]
        );
    }

    #[test]
    fn generates_standalone_page_with_embedded_manifest() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);
        let manifest = build_template_manifest(&template_graph);
        let page = generate_standalone_page("NestedCounter", &html, &manifest);

        assert!(page.starts_with("<!doctype html>\n"));
        assert!(page.contains("<title>NestedCounter</title>"));
        assert!(page.contains("<section data-ez-node=\"n0\">"));
        assert!(page.contains("id=\"ez-template-manifest\""));
        assert!(page.contains("\"name\": \"NestedCounter\""));
        assert!(page.contains("<script src=\"./runtime.js\" defer></script>"));
    }
}
