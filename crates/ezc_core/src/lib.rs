//! Core compiler data structures for the first `EdgeZero` learning slice.
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
    build_component_graph, ComponentAction, ComponentDiagnostic, ComponentGraph, ComponentMethod,
    ComponentNode, RenderChild, RenderEventHandler, RenderModel, SerializableValue, StateField,
    StateOperation,
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
    build_template_manifest, template_manifest_json, ManifestAction, ManifestComponent,
    ManifestEvent, ManifestNode, ManifestOperation, ManifestTemplate, TemplateManifest,
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
            component.state_fields[0].initial_value,
            Some(SerializableValue::Number("0".to_string()))
        );

        let method_names = component
            .methods
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(method_names, vec!["increment", "render"]);
        assert_eq!(
            component.actions,
            vec![ComponentAction {
                method: "increment".to_string(),
                operation: StateOperation::AddAssign(SerializableValue::Number("1".to_string())),
                field: "count".to_string(),
            }]
        );

        let render = component.render.as_ref().expect("expected render model");

        assert_eq!(render.root_element.as_deref(), Some("button"));
        assert_eq!(render.attributes, vec!["onClick={...}"]);
        assert_eq!(render.bindings, vec!["this.count"]);
        assert_eq!(
            render.event_handlers,
            vec![RenderEventHandler {
                event: "click".to_string(),
                handler: "this.increment".to_string(),
            }]
        );
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
    fn component_graph_reports_unsupported_event_errors() {
        let source = r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onMouseover={() => this.increment()}>Count: {this.count}</button>;
  }
}
"#;

        let parsed = ezc_parser::parse_file("UnsupportedEvent.tsx", source);

        let graph = build_component_graph(&parsed);

        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZC1005"));
    }

    #[test]
    fn component_graph_reports_duplicate_event_errors() {
        let parsed = ezc_parser::ParsedFile {
            path: "DuplicateEvent.tsx".into(),
            diagnostics: Vec::new(),
            classes: vec![ezc_parser::ParsedClass {
                name: "DuplicateEvent".to_string(),
                span: test_span(),
                decorators: vec![ezc_parser::ParsedDecorator {
                    name: "component".to_string(),
                    argument: Some("x-duplicate-event".to_string()),
                    span: test_span(),
                }],
                properties: Vec::new(),
                methods: vec![ezc_parser::ParsedMethod {
                    name: "render".to_string(),
                    span: test_span(),
                    jsx_roots: vec![ezc_parser::ParsedJsxElement {
                        name: "button".to_string(),
                        span: test_span(),
                        attributes: Vec::new(),
                        event_handlers: vec![
                            ezc_parser::ParsedEventHandler {
                                event: "click".to_string(),
                                handler: "this.render".to_string(),
                            },
                            ezc_parser::ParsedEventHandler {
                                event: "click".to_string(),
                                handler: "this.render".to_string(),
                            },
                        ],
                        children: Vec::new(),
                    }],
                    bindings: Vec::new(),
                    state_updates: Vec::new(),
                }],
            }],
        };

        let graph = build_component_graph(&parsed);

        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZC1006"));
    }

    fn test_span() -> ezc_parser::SourceSpan {
        ezc_parser::SourceSpan {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }

    #[test]
    fn builds_increment_action_from_parsed_method_update() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                method: "increment".to_string(),
                operation: StateOperation::Increment,
                field: "count".to_string(),
            }]
        );
    }

    #[test]
    fn builds_decrement_action_from_parsed_method_update() {
        let source =
            include_str!("../../../fixtures/0009-decrement-counter/input/DecrementCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                method: "decrement".to_string(),
                operation: StateOperation::Decrement,
                field: "count".to_string(),
            }]
        );
    }

    #[test]
    fn builds_add_and_subtract_assign_actions_from_parsed_method_updates() {
        let source =
            include_str!("../../../fixtures/0010-add-subtract-assign/input/StepCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![
                ComponentAction {
                    method: "addTwo".to_string(),
                    operation: StateOperation::AddAssign(SerializableValue::Number(
                        "2".to_string()
                    )),
                    field: "count".to_string(),
                },
                ComponentAction {
                    method: "subtractThree".to_string(),
                    operation: StateOperation::SubtractAssign(SerializableValue::Number(
                        "3".to_string()
                    )),
                    field: "count".to_string(),
                }
            ]
        );
    }

    #[test]
    fn builds_direct_assignment_action_from_parsed_method_update() {
        let source =
            include_str!("../../../fixtures/0011-direct-assignment/input/ResetCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                method: "reset".to_string(),
                operation: StateOperation::Assign(SerializableValue::Number("0".to_string())),
                field: "count".to_string(),
            }]
        );
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
            "<button data-ez-node=\"n0\" data-ez-on-click=\"this.increment\" data-ez-bindings=\"this.count\">Count:<!-- ez-binding:n1:this.count -->0</button>\n"
        );
    }

    #[test]
    fn preserves_string_state_literals_in_template_outputs() {
        let source = include_str!("../../../fixtures/0006-string-state/input/StringGreeting.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0006-string-state/input/StringGreeting.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let component = component_graph
            .components
            .first()
            .expect("expected component");

        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::String("Austin & <Zero>".to_string()))
        );

        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<p data-ez-node=\"n0\" data-ez-bindings=\"this.name\">Name:<!-- ez-binding:n1:this.name -->Austin &amp; &lt;Zero&gt;</p>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n1".to_string(),
                    expression: "this.name".to_string(),
                    initial_value: Some(SerializableValue::String("Austin & <Zero>".to_string())),
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["initial_value"],
            serde_json::json!("Austin & <Zero>")
        );
    }

    #[test]
    fn preserves_boolean_state_literals_in_template_outputs() {
        let source = include_str!("../../../fixtures/0007-boolean-state/input/BooleanFlags.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0007-boolean-state/input/BooleanFlags.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let component = component_graph
            .components
            .first()
            .expect("expected component");

        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::Boolean(true))
        );
        assert_eq!(
            component.state_fields[1].initial_value,
            Some(SerializableValue::Boolean(false))
        );

        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<section data-ez-node=\"n0\"><p data-ez-node=\"n1\" data-ez-bindings=\"this.enabled\">Enabled:<!-- ez-binding:n2:this.enabled -->true</p><p data-ez-node=\"n3\" data-ez-bindings=\"this.disabled\">Disabled:<!-- ez-binding:n4:this.disabled -->false</p></section>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "section".to_string(),
                },
                ManifestNode::Element {
                    id: "n1".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n2".to_string(),
                    expression: "this.enabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(true)),
                },
                ManifestNode::Element {
                    id: "n3".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n4".to_string(),
                    expression: "this.disabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(false)),
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][2]["initial_value"],
            serde_json::json!(true)
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][4]["initial_value"],
            serde_json::json!(false)
        );
    }

    #[test]
    fn preserves_null_state_literals_in_template_outputs() {
        let source = include_str!("../../../fixtures/0008-null-state/input/NullSelection.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0008-null-state/input/NullSelection.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let component = component_graph
            .components
            .first()
            .expect("expected component");

        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::Null)
        );

        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<p data-ez-node=\"n0\" data-ez-bindings=\"this.selection\">Selection:<!-- ez-binding:n1:this.selection --></p>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n1".to_string(),
                    expression: "this.selection".to_string(),
                    initial_value: Some(SerializableValue::Null),
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["initial_value"],
            serde_json::Value::Null
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
        assert_eq!(root.attributes[0].name, "data-ez-on-click");
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::EventHandler {
                event: "click".to_string(),
                handler: "this.increment".to_string(),
            }
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
                    initial_value: Some(SerializableValue::Number("0".to_string())),
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
        let manifest = build_template_manifest(&component_graph, &template_graph);

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
                    initial_value: Some(SerializableValue::Number("0".to_string())),
                }
            ]
        );

        assert_eq!(
            component.template.events,
            vec![ManifestEvent {
                node: "n1".to_string(),
                event: "click".to_string(),
                handler: "this.increment".to_string(),
            }]
        );

        assert_eq!(
            component.actions,
            vec![ManifestAction {
                method: "increment".to_string(),
                operation: ManifestOperation::Increment,
                field: "count".to_string(),
                operand: None,
            }]
        );
    }

    #[test]
    fn builds_template_manifest_for_decrement_action() {
        let source =
            include_str!("../../../fixtures/0009-decrement-counter/input/DecrementCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![ManifestAction {
                method: "decrement".to_string(),
                operation: ManifestOperation::Decrement,
                field: "count".to_string(),
                operand: None,
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("decrement")
        );
    }

    #[test]
    fn builds_template_manifest_for_add_and_subtract_assign_actions() {
        let source =
            include_str!("../../../fixtures/0010-add-subtract-assign/input/StepCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![
                ManifestAction {
                    method: "addTwo".to_string(),
                    operation: ManifestOperation::AddAssign,
                    field: "count".to_string(),
                    operand: Some(SerializableValue::Number("2".to_string())),
                },
                ManifestAction {
                    method: "subtractThree".to_string(),
                    operation: ManifestOperation::SubtractAssign,
                    field: "count".to_string(),
                    operand: Some(SerializableValue::Number("3".to_string())),
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("add_assign")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operand"],
            serde_json::json!("2")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][1]["operation"],
            serde_json::json!("subtract_assign")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][1]["operand"],
            serde_json::json!("3")
        );
    }

    #[test]
    fn builds_template_manifest_for_direct_assignment_action() {
        let source =
            include_str!("../../../fixtures/0011-direct-assignment/input/ResetCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![ManifestAction {
                method: "reset".to_string(),
                operation: ManifestOperation::Assign,
                field: "count".to_string(),
                operand: Some(SerializableValue::Number("0".to_string())),
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("assign")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operand"],
            serde_json::json!("0")
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
        let manifest = build_template_manifest(&component_graph, &template_graph);
        let page = generate_standalone_page("NestedCounter", &html, &manifest);

        assert!(page.starts_with("<!doctype html>\n"));
        assert!(page.contains("<title>NestedCounter</title>"));
        assert!(page.contains("<section data-ez-node=\"n0\">"));
        assert!(page.contains("id=\"ez-template-manifest\""));
        assert!(page.contains("\"name\": \"NestedCounter\""));
        assert!(page.contains("<script src=\"./runtime.js\" defer></script>"));
    }
}
