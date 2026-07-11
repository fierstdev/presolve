//! Core compiler data structures for the first `EdgeZero` learning slice.
//!
//! This crate deliberately does **not** parse TSX yet. It records a source summary,
//! spans, obvious declarations, and diagnostics. That gives the project a stable
//! place to learn compiler fundamentals before choosing a real parser backend.

pub mod application_semantic_model;
pub mod asm_validation;
pub mod binding_table;
pub mod compilation_unit;
pub mod compiler_pass;
pub mod component_graph;
pub mod explain;
pub mod html_codegen;
pub mod layout_graph;
pub mod lazy_action_chunks;
pub mod model;
pub mod module_graph;
pub mod page_codegen;
pub mod resume_boot;
pub mod resume_diagnostics;
pub mod resume_explain;
pub mod resume_instance;
pub mod resume_manifest;
pub mod resume_plan;
pub mod route_graph;
pub mod runtime_codegen;
pub mod semantic_id;
pub mod semantic_provenance;
pub mod semantic_reference;
pub mod summarize;
pub mod symbol_table;
pub mod template_graph;
pub mod template_manifest;
pub mod template_semantics;

pub use application_semantic_model::{
    build_application_semantic_model, build_application_semantic_model_for_unit,
    ApplicationSemanticModel, SemanticEntity,
};
pub use asm_validation::{validate_application_semantic_model, AsmValidationDiagnostic};
pub use binding_table::{
    build_binding_table, BindingDiagnostic, BindingTable, ExportBinding, ImportBinding,
    ImportBindingTarget, ModuleBindingTable,
};
pub use compilation_unit::CompilationUnit;
pub use compiler_pass::{
    AnalysisPass, ConstantEvaluation, ConstantEvaluationPass, DependencyAnalysis,
    DependencyAnalysisPass,
};
pub use component_graph::{
    build_component_graph, build_component_graph_for_module, ComponentAction, ComponentDiagnostic,
    ComponentGraph, ComponentMethod, ComponentNode, DeclaredStateType, DeclaredStateTypeKind,
    RenderAttribute, RenderAttributeValue, RenderChild, RenderEventHandler, RenderFragment,
    RenderList, RenderModel, SerializableValue, StateField, StateOperation,
};
pub use explain::{explain_json, explain_text};
pub use html_codegen::generate_static_html;
pub use model::{
    ClassSummary, DecoratorSummary, Diagnostic, RenderMethodSummary, Severity, SourceSummary, Span,
};
pub use module_graph::{
    build_module_graph, ModuleEdge, ModuleEdgeKind, ModuleGraph, ModuleNode, ModuleTarget,
};
pub use page_codegen::generate_standalone_page;
pub use resume_manifest::{
    build_resume_manifest, resume_manifest_json, ResumeManifest, RESUME_MANIFEST_SCHEMA_VERSION,
};
pub use resume_plan::{build_resume_plan, ResumeComponentPlan, ResumePlan};
pub use runtime_codegen::generate_runtime_stub;
pub use semantic_id::{SemanticId, SemanticOwner};
pub use semantic_provenance::SourceProvenance;
pub use semantic_reference::{SemanticReference, SemanticReferenceKind};
pub use summarize::summarize_source;
pub use symbol_table::{
    build_symbol_table, ModuleSymbol, ModuleSymbolTable, SymbolDiagnostic, SymbolKind, SymbolTable,
};
pub use template_graph::{
    build_template_graph, AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode,
    TemplateAttribute, TemplateChild, TemplateGraph, TemplateNode, TemplateNodeId,
};
pub use template_manifest::{
    build_template_manifest, template_manifest_json, ManifestAction, ManifestBindingTarget,
    ManifestComponent, ManifestEvent, ManifestNode, ManifestOperation, ManifestTemplate,
    TemplateManifest, TEMPLATE_MANIFEST_SCHEMA_VERSION,
};
pub use template_semantics::{
    build_template_semantic_entities, TemplateSemanticEntity, TemplateSemanticKind,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, path::Path};

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
    #[allow(clippy::too_many_lines)]
    fn builds_component_graph_from_parsed_counter() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let graph = build_component_graph(&parsed);

        assert!(graph.diagnostics.is_empty());

        let component = graph.components.first().expect("expected component");

        assert_eq!(component.class_name, "Counter");
        assert_eq!(component.id.as_str(), "component:x-counter");
        assert_eq!(component.owner, SemanticOwner::Application);
        assert_eq!(component.element_name.as_deref(), Some("x-counter"));
        assert_eq!(component.route_path.as_deref(), Some("/counter"));

        assert_eq!(component.state_fields.len(), 1);
        assert_eq!(component.state_fields[0].name, "count");
        assert_eq!(
            component.state_fields[0].id.as_str(),
            "component:x-counter/state:count"
        );
        assert_eq!(
            component.state_fields[0].owner,
            SemanticOwner::entity(component.id.clone())
        );
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
            component.methods[0].id.as_str(),
            "component:x-counter/method:increment"
        );
        assert_eq!(
            component.methods[0].owner,
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-counter"), "Counter").action("increment", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-counter"), "Counter").method("increment"),
                ),
                method: "increment".to_string(),
                operation: StateOperation::AddAssign(SerializableValue::Number("1".to_string())),
                field: "count".to_string(),
            }]
        );

        let render = component.render.as_ref().expect("expected render model");

        assert_eq!(render.root_element.as_deref(), Some("button"));
        assert_eq!(render.attributes.len(), 1);
        assert_eq!(render.attributes[0].name, "onClick");
        assert!(matches!(
            render.attributes[0].value,
            RenderAttributeValue::Expression(_)
        ));
        assert_eq!(render.bindings, vec!["this.count"]);
        assert_eq!(render.root_span.expect("expected root span").line, 12);
        assert_eq!(render.root_span.expect("expected root span").column, 7);
        assert_eq!(render.event_handlers.len(), 1);
        assert_eq!(
            render.event_handlers[0].id.as_str(),
            "component:x-counter/event:click:0"
        );
        assert_eq!(
            render.event_handlers[0].owner,
            SemanticOwner::entity(component.id.template())
        );
        assert_eq!(render.event_handlers[0].event, "click");
        assert_eq!(render.event_handlers[0].handler, "this.increment");
        assert_eq!(render.event_handlers[0].span.line, 12);
        assert_eq!(render.event_handlers[0].span.column, 15);
        assert_eq!(render.children.len(), 2);

        let RenderChild::Text { value, span } = &render.children[0] else {
            panic!("expected text child");
        };
        assert_eq!(value, "Count:");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 9);

        let RenderChild::Binding { expression, span } = &render.children[1] else {
            panic!("expected binding child");
        };
        assert_eq!(expression, "this.count");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 16);

        assert_eq!(graph.references.len(), 2);
        assert_eq!(graph.references[0].kind, SemanticReferenceKind::ActionState);
        assert_eq!(
            graph.references[0].source,
            SemanticId::component(Some("x-counter"), "Counter").action("increment", 0)
        );
        assert_eq!(
            graph.references[0].target,
            SemanticId::component(Some("x-counter"), "Counter").state_field("count")
        );
        assert_eq!(
            graph.references[0].provenance.path,
            Path::new("fixtures/0001-source-summary/input/Counter.tsx")
        );
        assert_eq!(graph.references[0].provenance.span.line, 7);

        assert_eq!(graph.references[1].kind, SemanticReferenceKind::EventMethod);
        assert_eq!(
            graph.references[1].source,
            SemanticId::component(Some("x-counter"), "Counter").event_handler("click", 0)
        );
        assert_eq!(
            graph.references[1].target,
            SemanticId::component(Some("x-counter"), "Counter").method("increment")
        );
        assert_eq!(
            graph.references[1].provenance.path,
            Path::new("fixtures/0001-source-summary/input/Counter.tsx")
        );
        assert_eq!(graph.references[1].provenance.span.line, 12);

        assert_eq!(graph.provenance[&component.id].span.line, 1);
        assert_eq!(graph.provenance[&component.state_fields[0].id].span.line, 4);
        assert_eq!(graph.provenance[&component.methods[0].id].span.line, 6);
        assert_eq!(graph.provenance[&component.actions[0].id].span.line, 7);
        assert_eq!(graph.provenance[&component.id.template()].span.line, 10);
        assert_eq!(graph.provenance[&render.event_handlers[0].id].span.line, 12);
    }

    #[test]
    fn assembles_application_semantic_model_from_existing_graphs() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");
        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];

        assert!(asm.diagnostics.is_empty());
        assert_eq!(asm.templates.len(), 1);
        assert!(asm.template_entities.len() >= 3);
        assert_eq!(asm.references.len(), 4);
        assert_eq!(asm.ownership.len(), asm.provenance.len());
        assert_eq!(asm.ownership[&component.id], SemanticOwner::Application);
        assert_eq!(
            asm.ownership[&component.state_fields[0].id],
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(
            asm.ownership[&component.actions[0].id],
            SemanticOwner::entity(component.methods[0].id.clone())
        );
        assert_eq!(
            asm.ownership[&asm.templates[0].id],
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(asm.provenance[&asm.templates[0].id].span.line, 10);

        assert!(matches!(
            asm.entity(&component.id),
            Some(SemanticEntity::Component(_))
        ));
        assert!(matches!(
            asm.entity(&component.state_fields[0].id),
            Some(SemanticEntity::StateField(_))
        ));
        assert_eq!(asm.component(&component.id), Some(component));
        assert_eq!(asm.template(&asm.templates[0].id), Some(&asm.templates[0]));
        assert_eq!(
            asm.owner(&component.actions[0].id),
            Some(&component.actions[0].owner)
        );
        assert_eq!(asm.provenance(&asm.templates[0].id).unwrap().span.line, 10);
        let template_binding = asm
            .template_entities
            .iter()
            .find(|entity| entity.kind == TemplateSemanticKind::Binding)
            .expect("template binding entity");
        assert!(matches!(
            asm.entity(&template_binding.id),
            Some(SemanticEntity::TemplateEntity(_))
        ));
        assert_eq!(
            asm.template_entities_for(&asm.templates[0].id).len(),
            asm.template_entities.len()
        );
        assert_eq!(asm.references_from(&component.actions[0].id).len(), 1);
        assert_eq!(asm.references_to(&component.state_fields[0].id).len(), 2);
        assert_eq!(asm.references_to(&component.methods[0].id).len(), 2);
        assert!(validate_application_semantic_model(&asm).is_empty());

        let mut invalid = asm.clone();
        invalid.provenance.remove(&component.actions[0].id);
        let diagnostics = validate_application_semantic_model(&invalid);
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert!(codes.contains(&"EZASM1002"));
        assert!(codes.contains(&"EZASM1006"));

        let dependencies = DependencyAnalysisPass.analyze(&asm);
        assert_eq!(dependencies.dependencies[&component.actions[0].id].len(), 1);
        assert_eq!(
            dependencies.dependents[&component.state_fields[0].id].len(),
            2
        );
    }

    #[test]
    fn carries_declared_state_types_into_component_and_asm_data() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  count: number = state(0);
}
"#,
        );

        let graph = build_component_graph_for_module(&parsed);
        let graph_type = graph.components[0].state_fields[0]
            .declared_type
            .as_ref()
            .expect("declared state type");

        assert_eq!(graph_type.text, "number");
        assert_eq!(graph_type.provenance.path, Path::new("src/Panel.tsx"));
        assert_eq!(graph_type.provenance.span.line, 4);
        assert_eq!(graph_type.provenance.span.column, 8);
        assert_eq!(graph_type.kind, Some(DeclaredStateTypeKind::Number));

        let asm = build_application_semantic_model(&parsed);
        assert_eq!(
            asm.components[0].state_fields[0].declared_type,
            Some(graph_type.clone())
        );
    }

    #[test]
    fn classifies_exact_primitive_declared_state_types() {
        let source =
            include_str!("../../../fixtures/0025-typed-state-annotations/input/TypedState.tsx");
        let parsed = ezc_parser::parse_file(
            "fixtures/0025-typed-state-annotations/input/TypedState.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let kinds = graph.components[0]
            .state_fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    field
                        .declared_type
                        .as_ref()
                        .and_then(|declared_type| declared_type.kind),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                ("count", Some(DeclaredStateTypeKind::Number)),
                ("status", None),
                ("title", Some(DeclaredStateTypeKind::String)),
                ("enabled", Some(DeclaredStateTypeKind::Boolean)),
                ("empty", Some(DeclaredStateTypeKind::Null)),
            ]
        );
    }

    #[test]
    fn reports_primitive_declared_state_initializer_mismatches() {
        let source = include_str!(
            "../../../fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let diagnostics = graph
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            diagnostics,
            vec![
                (
                    "EZC1016",
                    "state field `count` in class `InvalidTypedState` declares `number` but initializes with `string`",
                ),
                (
                    "EZC1016",
                    "state field `title` in class `InvalidTypedState` declares `string` but initializes with `number`",
                ),
                (
                    "EZC1016",
                    "state field `enabled` in class `InvalidTypedState` declares `boolean` but initializes with `null`",
                ),
                (
                    "EZC1016",
                    "state field `empty` in class `InvalidTypedState` declares `null` but initializes with `boolean`",
                ),
            ]
        );

        let provenance = graph.diagnostics[0]
            .provenance
            .as_ref()
            .expect("mismatch diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new("fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx")
        );
        assert_eq!(provenance.span.line, 3);
        assert_eq!(provenance.span.column, 8);
    }

    #[test]
    fn reports_primitive_declared_state_action_assignment_mismatches() {
        let source = include_str!(
            "../../../fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let diagnostics = graph
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            diagnostics,
            vec![
                (
                    "EZC1017",
                    "state field `count` in class `InvalidTypedActions` declares `number` but action `apply` assigns `string`",
                ),
                (
                    "EZC1017",
                    "state field `title` in class `InvalidTypedActions` declares `string` but action `apply` assigns `number`",
                ),
                (
                    "EZC1017",
                    "state field `enabled` in class `InvalidTypedActions` declares `boolean` but action `apply` assigns `null`",
                ),
                (
                    "EZC1017",
                    "state field `empty` in class `InvalidTypedActions` declares `null` but action `apply` assigns `boolean`",
                ),
            ]
        );

        let provenance = graph.diagnostics[0]
            .provenance
            .as_ref()
            .expect("action mismatch diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new(
                "fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx"
            )
        );
        assert_eq!(provenance.span.line, 11);
        assert_eq!(provenance.span.column, 5);
    }

    #[test]
    fn reports_non_boolean_primitive_toggle_actions() {
        let source = include_str!(
            "../../../fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let diagnostics = graph
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            diagnostics,
            vec![
                (
                    "EZC1018",
                    "state field `count` in class `InvalidTypedToggles` declares `number` but action `apply` applies a boolean toggle",
                ),
                (
                    "EZC1018",
                    "state field `title` in class `InvalidTypedToggles` declares `string` but action `apply` applies a boolean toggle",
                ),
                (
                    "EZC1018",
                    "state field `empty` in class `InvalidTypedToggles` declares `null` but action `apply` applies a boolean toggle",
                ),
            ]
        );

        let provenance = graph.diagnostics[0]
            .provenance
            .as_ref()
            .expect("toggle diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new(
                "fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx"
            )
        );
        assert_eq!(provenance.span.line, 10);
        assert_eq!(provenance.span.column, 5);
    }

    #[test]
    fn assembles_application_semantic_model_from_multiple_files() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/Zeta.tsx",
                r#"
@component("x-zeta")
class Zeta extends Component {
  render() {
    return <div>Zeta</div>;
  }
}
"#,
            ),
            (
                "src/Alpha.tsx",
                r#"
@component("x-alpha")
class Alpha extends Component {
  render() {
    return <div>Alpha</div>;
  }
}
"#,
            ),
        ]);

        let asm = build_application_semantic_model_for_unit(&unit);

        assert_eq!(
            unit.files()
                .iter()
                .map(|file| file.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["src/Alpha.tsx", "src/Zeta.tsx"]
        );
        assert_eq!(
            asm.components
                .iter()
                .map(|component| component.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "module:src/Alpha.tsx/component:x-alpha",
                "module:src/Zeta.tsx/component:x-zeta"
            ]
        );
        assert!(asm.diagnostics.is_empty());
        assert!(validate_application_semantic_model(&asm).is_empty());
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
        assert!(graph.references.is_empty());
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
            imports: Vec::new(),
            exports: Vec::new(),
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
                    jsx_roots: vec![ezc_parser::ParsedJsxNode::Element(
                        ezc_parser::ParsedJsxElement {
                            name: "button".to_string(),
                            span: test_span(),
                            attributes: Vec::new(),
                            event_handlers: vec![
                                ezc_parser::ParsedEventHandler {
                                    event: "click".to_string(),
                                    handler: "this.render".to_string(),
                                    span: test_span(),
                                },
                                ezc_parser::ParsedEventHandler {
                                    event: "click".to_string(),
                                    handler: "this.render".to_string(),
                                    span: test_span(),
                                },
                            ],
                            children: Vec::new(),
                        },
                    )],
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
                id: SemanticId::component(Some("x-nested-counter"), "NestedCounter")
                    .action("increment", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-nested-counter"), "NestedCounter")
                        .method("increment"),
                ),
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
                id: SemanticId::component(Some("x-decrement-counter"), "DecrementCounter")
                    .action("decrement", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-decrement-counter"), "DecrementCounter")
                        .method("decrement"),
                ),
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
                    id: SemanticId::component(Some("x-step-counter"), "StepCounter")
                        .action("addTwo", 0),
                    owner: SemanticOwner::entity(
                        SemanticId::component(Some("x-step-counter"), "StepCounter")
                            .method("addTwo"),
                    ),
                    method: "addTwo".to_string(),
                    operation: StateOperation::AddAssign(SerializableValue::Number(
                        "2".to_string()
                    )),
                    field: "count".to_string(),
                },
                ComponentAction {
                    id: SemanticId::component(Some("x-step-counter"), "StepCounter")
                        .action("subtractThree", 0),
                    owner: SemanticOwner::entity(
                        SemanticId::component(Some("x-step-counter"), "StepCounter")
                            .method("subtractThree"),
                    ),
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
                id: SemanticId::component(Some("x-reset-counter"), "ResetCounter")
                    .action("reset", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-reset-counter"), "ResetCounter").method("reset"),
                ),
                method: "reset".to_string(),
                operation: StateOperation::Assign(SerializableValue::Number("0".to_string())),
                field: "count".to_string(),
            }]
        );
    }

    #[test]
    fn builds_boolean_toggle_action_from_parsed_method_update() {
        let source = include_str!("../../../fixtures/0012-boolean-toggle/input/ToggleFlag.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0012-boolean-toggle/input/ToggleFlag.tsx", source);

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-toggle-flag"), "ToggleFlag").action("toggle", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-toggle-flag"), "ToggleFlag").method("toggle"),
                ),
                method: "toggle".to_string(),
                operation: StateOperation::Toggle,
                field: "enabled".to_string(),
            }]
        );
    }

    #[test]
    fn builds_multi_step_actions_from_parsed_method_updates_in_source_order() {
        let source =
            include_str!("../../../fixtures/0013-multi-step-action/input/BatchActionCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 0),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::AddAssign(SerializableValue::Number(
                        "2".to_string()
                    )),
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 1),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Decrement,
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 2),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Assign(SerializableValue::Number("8".to_string())),
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 3),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Increment,
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 4),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Toggle,
                    field: "enabled".to_string(),
                }
            ]
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
                    target: None,
                    element: None,
                    attribute: None,
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
    fn preserves_static_jsx_attributes_in_template_outputs() {
        let source =
            include_str!("../../../fixtures/0014-static-attributes/input/StaticAttributePanel.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0014-static-attributes/input/StaticAttributePanel.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root");

        assert_eq!(root.attributes.len(), 3);
        assert_eq!(root.attributes[0].name, "id");
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::Static("panel-root".to_string())
        );
        assert_eq!(root.attributes[1].name, "aria-label");
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::Static("Status \"Panel\"".to_string())
        );
        assert_eq!(root.attributes[2].name, "hidden");
        assert_eq!(root.attributes[2].value, AttributeValue::Boolean);

        let TemplateChild::Element(button) = &root.children[0] else {
            panic!("expected button child");
        };

        assert_eq!(button.attributes.len(), 4);
        assert_eq!(button.attributes[0].name, "type");
        assert_eq!(
            button.attributes[0].value,
            AttributeValue::Static("button".to_string())
        );
        assert_eq!(button.attributes[1].name, "data-mode");
        assert_eq!(
            button.attributes[1].value,
            AttributeValue::Static("safe & sound".to_string())
        );
        assert_eq!(button.attributes[2].name, "title");
        assert_eq!(
            button.attributes[2].value,
            AttributeValue::Static("Use <carefully>".to_string())
        );
        assert_eq!(button.attributes[3].name, "data-ez-bindings");

        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<section data-ez-node=\"n0\" id=\"panel-root\" aria-label=\"Status &quot;Panel&quot;\" hidden><button data-ez-node=\"n1\" type=\"button\" data-mode=\"safe &amp; sound\" title=\"Use &lt;carefully&gt;\" data-ez-bindings=\"this.label\">Label:<!-- ez-binding:n2:this.label -->Ready</button></section>\n"
        );
    }

    #[test]
    fn reports_static_attribute_semantic_errors() {
        let source = r#"
@component("x-bad-attrs")
class BadAttrs extends Component {
  disabled = state(false);

  render() {
    return <button type="button" type="submit" title={label} {...props}>Go</button>;
  }
}
"#;

        let parsed = ezc_parser::parse_file("BadAttrs.tsx", source);
        let graph = build_component_graph(&parsed);
        let codes = graph
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();

        assert!(codes.contains(&"EZC1007"));
        assert!(codes.contains(&"EZC1008"));
        assert!(codes.contains(&"EZC1009"));
    }

    #[test]
    fn builds_dynamic_attribute_bindings_in_template_outputs() {
        let source = include_str!(
            "../../../fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root");

        assert_eq!(root.attributes[0].name, "disabled");
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::Binding {
                id: TemplateNodeId("n1".to_string()),
                expression: "this.disabled".to_string(),
                initial_value: Some(SerializableValue::Boolean(false)),
            }
        );
        assert_eq!(root.attributes[1].name, "title");
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::Binding {
                id: TemplateNodeId("n2".to_string()),
                expression: "this.label".to_string(),
                initial_value: Some(SerializableValue::String("Ready".to_string())),
            }
        );

        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<button data-ez-node=\"n0\" title=\"Ready\" data-ez-on-click=\"this.lock\" data-ez-bindings=\"this.label\">Status:<!-- ez-binding:n3:this.label -->Ready</button>\n"
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
                    target: None,
                    element: None,
                    attribute: None,
                },
                ManifestNode::Element {
                    id: "n3".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n4".to_string(),
                    expression: "this.disabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(false)),
                    target: None,
                    element: None,
                    attribute: None,
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
                    target: None,
                    element: None,
                    attribute: None,
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
        assert_eq!(template.id.as_str(), "component:x-counter/template:render");
        assert_eq!(
            template.provenance.path,
            Path::new("fixtures/0001-source-summary/input/Counter.tsx")
        );
        assert_eq!(template.provenance.span.line, 10);
        assert_eq!(
            template.owner,
            SemanticOwner::entity(SemanticId::component(Some("x-counter"), "Counter"))
        );

        let root = template.root.as_ref().expect("expected template root");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.tag_name, "button");
        assert_eq!(root.span.line, 12);
        assert_eq!(root.span.column, 7);

        assert_eq!(root.attributes.len(), 2);
        assert_eq!(root.attributes[0].name, "data-ez-on-click");
        assert_eq!(
            root.attributes[0].span.expect("expected event span").line,
            12
        );
        assert_eq!(
            root.attributes[0].span.expect("expected event span").column,
            15
        );
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::EventHandler {
                event: "click".to_string(),
                handler: "this.increment".to_string(),
            }
        );

        assert_eq!(root.attributes[1].name, "data-ez-bindings");
        assert_eq!(root.attributes[1].span, None);
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::BindingList(vec!["this.count".to_string()])
        );

        assert_eq!(root.children.len(), 2);
        let TemplateChild::Text { value, span } = &root.children[0] else {
            panic!("expected text child");
        };
        assert_eq!(value, "Count:");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 9);

        let TemplateChild::Binding {
            id,
            expression,
            initial_value,
            span,
        } = &root.children[1]
        else {
            panic!("expected binding child");
        };
        assert_eq!(id.0, "n1");
        assert_eq!(expression, "this.count");
        assert_eq!(
            initial_value,
            &Some(SerializableValue::Number("0".to_string()))
        );
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 16);
    }

    #[test]
    fn semantic_ids_are_stable_when_component_declaration_order_changes() {
        let alpha = r#"
@component("x-alpha")
class Alpha extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <p>{this.count}</p>;
  }
}
"#;
        let beta = r#"
@component("x-beta")
class Beta extends Component {
  enabled = state(false);

  toggle() {
    this.enabled = !this.enabled;
  }

  render() {
    return <p>{this.enabled}</p>;
  }
}
"#;

        let ids_for = |source: &str| {
            let parsed = ezc_parser::parse_file("App.tsx", source);
            let component_graph = build_component_graph(&parsed);
            let template_graph = build_template_graph(&component_graph);
            let template_ids = template_graph
                .templates
                .iter()
                .map(|template| (template.component_name.as_str(), template.id.to_string()))
                .collect::<BTreeMap<_, _>>();

            component_graph
                .components
                .iter()
                .map(|component| {
                    let ids = component
                        .state_fields
                        .iter()
                        .map(|field| field.id.to_string())
                        .chain(component.methods.iter().map(|method| method.id.to_string()))
                        .chain(component.actions.iter().map(|action| action.id.to_string()))
                        .chain(std::iter::once(
                            template_ids
                                .get(component.class_name.as_str())
                                .expect("expected component template")
                                .clone(),
                        ))
                        .collect::<Vec<_>>();

                    (
                        component.class_name.clone(),
                        (component.id.to_string(), ids),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };

        assert_eq!(
            ids_for(&format!("{alpha}\n{beta}")),
            ids_for(&format!("{beta}\n{alpha}"))
        );
    }

    #[test]
    fn carries_source_spans_into_nested_template_nodes() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root");

        assert_eq!(root.tag_name, "section");
        assert_eq!(root.span.line, 12);
        assert_eq!(root.span.column, 7);

        let TemplateChild::Element(button) = &root.children[0] else {
            panic!("expected nested button");
        };

        assert_eq!(button.tag_name, "button");
        assert_eq!(button.span.line, 13);
        assert_eq!(button.span.column, 9);
        assert_eq!(button.attributes[0].name, "data-ez-on-click");
        assert_eq!(
            button.attributes[0].span.expect("expected event span").line,
            13
        );
        assert_eq!(
            button.attributes[0]
                .span
                .expect("expected event span")
                .column,
            17
        );
        assert_eq!(button.attributes[1].name, "data-ez-bindings");
        assert_eq!(button.attributes[1].span, None);

        let TemplateChild::Text { value, span } = &button.children[0] else {
            panic!("expected nested text");
        };
        assert_eq!(value, "Count:");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 50);

        let TemplateChild::Binding {
            expression, span, ..
        } = &button.children[1]
        else {
            panic!("expected nested binding");
        };
        assert_eq!(expression, "this.count");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 57);
    }

    #[test]
    fn preserves_fragment_siblings_without_wrapper_elements() {
        let source = include_str!("../../../fixtures/0016-fragments/input/FragmentPanel.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0016-fragments/input/FragmentPanel.tsx", source);

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let template = &template_graph.templates[0];
        let fragment = template
            .root_fragment
            .as_ref()
            .expect("expected fragment root");

        assert!(template.root.is_none());
        assert_eq!(fragment.id.0, "n0");
        assert_eq!(fragment.children.len(), 2);

        let TemplateChild::Element(heading) = &fragment.children[0] else {
            panic!("expected heading child");
        };
        assert_eq!(heading.id.0, "n1");
        assert_eq!(heading.tag_name, "h1");

        let TemplateChild::Fragment(nested) = &fragment.children[1] else {
            panic!("expected nested fragment child");
        };
        assert_eq!(nested.id.0, "n2");

        let TemplateChild::Element(paragraph) = &nested.children[0] else {
            panic!("expected paragraph child");
        };
        assert_eq!(paragraph.id.0, "n3");
        assert_eq!(paragraph.tag_name, "p");

        let html = generate_static_html(&template_graph);
        assert_eq!(
            html,
            "<h1 data-ez-node=\"n1\">Title</h1><p data-ez-node=\"n3\" data-ez-bindings=\"this.label\">Status:<!-- ez-binding:n4:this.label -->Ready</p><span data-ez-node=\"n5\">Done</span>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n1".to_string(),
                    tag: "h1".to_string(),
                },
                ManifestNode::Element {
                    id: "n3".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n4".to_string(),
                    expression: "this.label".to_string(),
                    initial_value: Some(SerializableValue::String("Ready".to_string())),
                    target: None,
                    element: None,
                    attribute: None,
                },
                ManifestNode::Element {
                    id: "n5".to_string(),
                    tag: "span".to_string(),
                },
            ]
        );
    }

    #[test]
    fn builds_conditional_template_boundaries_and_manifest_branch_html() {
        let source = include_str!(
            "../../../fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.attributes[1].name, "data-ez-bindings");

        let TemplateChild::Conditional(conditional) = &root.children[0] else {
            panic!("expected conditional child");
        };

        assert_eq!(conditional.id.0, "n1");
        assert_eq!(conditional.start_id.0, "n2");
        assert_eq!(conditional.end_id.0, "n3");
        assert_eq!(conditional.condition, "this.enabled");
        assert_eq!(
            conditional.initial_value,
            Some(SerializableValue::Boolean(true))
        );

        let TemplateChild::Element(when_true) = &conditional.when_true[0] else {
            panic!("expected true branch element");
        };
        assert_eq!(when_true.id.0, "n4");
        assert_eq!(when_true.tag_name, "span");

        let TemplateChild::Element(when_false) = &conditional.when_false[0] else {
            panic!("expected false branch element");
        };
        assert_eq!(when_false.id.0, "n5");
        assert_eq!(when_false.tag_name, "span");

        let html = generate_static_html(&template_graph);
        assert_eq!(
            html,
            "<button data-ez-node=\"n0\" data-ez-on-click=\"this.toggle\" data-ez-bindings=\"this.enabled\"><!-- ez-conditional-start:n2:this.enabled --><span data-ez-node=\"n4\">On</span><!-- ez-conditional-end:n3 --></button>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "button".to_string(),
                },
                ManifestNode::Conditional {
                    id: "n1".to_string(),
                    start: "n2".to_string(),
                    end: "n3".to_string(),
                    condition: "this.enabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(true)),
                    when_true_html: "<span data-ez-node=\"n4\">On</span>".to_string(),
                    when_false_html: "<span data-ez-node=\"n5\">Off</span>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn builds_logical_and_conditional_with_empty_false_branch() {
        let source = include_str!(
            "../../../fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");

        let TemplateChild::Conditional(conditional) = &root.children[0] else {
            panic!("expected conditional child");
        };

        assert_eq!(conditional.when_true.len(), 1);
        assert!(conditional.when_false.is_empty());

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes[1],
            ManifestNode::Conditional {
                id: "n1".to_string(),
                start: "n2".to_string(),
                end: "n3".to_string(),
                condition: "this.enabled".to_string(),
                initial_value: Some(SerializableValue::Boolean(true)),
                when_true_html: "<span data-ez-node=\"n4\">On</span>".to_string(),
                when_false_html: String::new(),
            }
        );
    }

    #[test]
    fn builds_empty_keyed_list_from_a_serializable_array() {
        let source =
            include_str!("../../../fixtures/0019-keyed-list-semantics/input/KeyedList.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0019-keyed-list-semantics/input/KeyedList.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.attributes[0].name, "data-ez-bindings");

        let TemplateChild::List(list) = &root.children[0] else {
            panic!("expected keyed list child");
        };

        assert_eq!(list.id.0, "n1");
        assert_eq!(list.start_id.0, "n2");
        assert_eq!(list.end_id.0, "n3");
        assert_eq!(list.iterable, "this.items");
        assert_eq!(
            list.initial_value,
            Some(SerializableValue::Array(Vec::new()))
        );
        assert_eq!(list.item_variable, "item");
        assert_eq!(list.index_variable.as_deref(), Some("index"));
        assert_eq!(list.key_expression, "item.id");

        let TemplateChild::Element(item_template) = &list.item_template[0] else {
            panic!("expected list item element");
        };
        assert_eq!(item_template.id.0, "n4");
        assert_eq!(item_template.tag_name, "li");

        assert_eq!(
            generate_static_html(&template_graph),
            "<ul data-ez-node=\"n0\" data-ez-bindings=\"this.items\"><!-- ez-list-start:n2:this.items --><!-- ez-list-end:n3 --></ul>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "ul".to_string(),
                },
                ManifestNode::List {
                    id: "n1".to_string(),
                    start: "n2".to_string(),
                    end: "n3".to_string(),
                    iterable: "this.items".to_string(),
                    initial_value: Some(SerializableValue::Array(Vec::new())),
                    item_variable: "item".to_string(),
                    index_variable: Some("index".to_string()),
                    key_expression: "item.id".to_string(),
                    item_root: "n4".to_string(),
                    item_template_html: "<li data-ez-node=\"n4:__ez_list_key__\" data-ez-bindings=\"index,item.label\"><!-- ez-binding:n5:__ez_list_key__:index -->__ez_list_index__<!-- ez-list-binding-end:n5:__ez_list_key__ -->:<!-- ez-binding:n6:__ez_list_key__:item.label --><!-- ez-list-binding-end:n6:__ez_list_key__ --></li>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn preserves_recursive_object_values_in_component_and_manifest_models() {
        let source = include_str!(
            "../../../fixtures/0023-recursive-object-values/input/RecursiveObjectValues.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0023-recursive-object-values/input/RecursiveObjectValues.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let profile = SerializableValue::Object(BTreeMap::from([
            (
                "name".to_string(),
                SerializableValue::String("North".to_string()),
            ),
            (
                "settings".to_string(),
                SerializableValue::Object(BTreeMap::from([
                    ("enabled".to_string(), SerializableValue::Boolean(true)),
                    (
                        "tags".to_string(),
                        SerializableValue::Array(vec![
                            SerializableValue::String("compiler".to_string()),
                            SerializableValue::Object(BTreeMap::from([
                                (
                                    "name".to_string(),
                                    SerializableValue::String("runtime".to_string()),
                                ),
                                (
                                    "rank".to_string(),
                                    SerializableValue::Number("2".to_string()),
                                ),
                            ])),
                        ]),
                    ),
                ])),
            ),
        ]));
        assert_eq!(
            component_graph.components[0].state_fields[0].initial_value,
            Some(profile.clone())
        );

        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes[3],
            ManifestNode::Binding {
                id: "n3".to_string(),
                expression: "this.profile".to_string(),
                initial_value: Some(profile),
                target: None,
                element: None,
                attribute: None,
            }
        );
        assert!(matches!(
            manifest.components[0].actions[0].operand,
            Some(SerializableValue::Object(_))
        ));
    }

    #[test]
    fn renders_static_object_list_members_and_keys() {
        let source = include_str!(
            "../../../fixtures/0024-static-object-keyed-list/input/StaticObjectKeyedList.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0024-static-object-keyed-list/input/StaticObjectKeyedList.tsx",
            source,
        );
        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        assert_eq!(
            generate_static_html(&template_graph),
            "<ol data-ez-node=\"n0\" data-ez-bindings=\"this.items\"><!-- ez-list-start:n2:this.items --><li data-ez-node=\"n4:north\" data-ez-bindings=\"index,item.label,item.details.region\"><!-- ez-binding:n5:north:index -->0<!-- ez-list-binding-end:n5:north -->:<!-- ez-binding:n6:north:item.label -->North<!-- ez-list-binding-end:n6:north -->(<!-- ez-binding:n7:north:item.details.region -->west<!-- ez-list-binding-end:n7:north -->)</li><li data-ez-node=\"n4:south\" data-ez-bindings=\"index,item.label,item.details.region\"><!-- ez-binding:n5:south:index -->1<!-- ez-list-binding-end:n5:south -->:<!-- ez-binding:n6:south:item.label -->South<!-- ez-list-binding-end:n6:south -->(<!-- ez-binding:n7:south:item.details.region -->east<!-- ez-list-binding-end:n7:south -->)</li><!-- ez-list-end:n3 --></ol>\n"
        );
    }

    #[test]
    fn renders_initial_keyed_list_items_from_a_serializable_array() {
        let source =
            include_str!("../../../fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx",
            source,
        );
        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");
        let TemplateChild::List(list) = &root.children[0] else {
            panic!("expected keyed list child");
        };

        assert_eq!(
            list.initial_value,
            Some(SerializableValue::Array(vec![
                SerializableValue::String("North".to_string()),
                SerializableValue::String("South".to_string()),
            ]))
        );

        assert_eq!(
            generate_static_html(&template_graph),
            "<ol data-ez-node=\"n0\" data-ez-bindings=\"this.labels\"><!-- ez-list-start:n2:this.labels --><li data-ez-node=\"n4:North\" data-ez-bindings=\"index,label\"><!-- ez-binding:n5:North:index -->0<!-- ez-list-binding-end:n5:North -->:<!-- ez-binding:n6:North:label -->North<!-- ez-list-binding-end:n6:North --></li><li data-ez-node=\"n4:South\" data-ez-bindings=\"index,label\"><!-- ez-binding:n5:South:index -->1<!-- ez-list-binding-end:n5:South -->:<!-- ez-binding:n6:South:label -->South<!-- ez-list-binding-end:n6:South --></li><!-- ez-list-end:n3 --></ol>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "ol".to_string(),
                },
                ManifestNode::List {
                    id: "n1".to_string(),
                    start: "n2".to_string(),
                    end: "n3".to_string(),
                    iterable: "this.labels".to_string(),
                    initial_value: Some(SerializableValue::Array(vec![
                        SerializableValue::String("North".to_string()),
                        SerializableValue::String("South".to_string()),
                    ])),
                    item_variable: "label".to_string(),
                    index_variable: Some("index".to_string()),
                    key_expression: "label".to_string(),
                    item_root: "n4".to_string(),
                    item_template_html: "<li data-ez-node=\"n4:__ez_list_key__\" data-ez-bindings=\"index,label\"><!-- ez-binding:n5:__ez_list_key__:index -->__ez_list_index__<!-- ez-list-binding-end:n5:__ez_list_key__ -->:<!-- ez-binding:n6:__ez_list_key__:label -->__ez_list_item__<!-- ez-list-binding-end:n6:__ez_list_key__ --></li>".to_string(),
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

        assert_eq!(manifest.schema_version, TEMPLATE_MANIFEST_SCHEMA_VERSION);
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
                    target: None,
                    element: None,
                    attribute: None,
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

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["schema_version"],
            serde_json::json!(TEMPLATE_MANIFEST_SCHEMA_VERSION)
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
    fn builds_template_manifest_for_boolean_toggle_action() {
        let source = include_str!("../../../fixtures/0012-boolean-toggle/input/ToggleFlag.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0012-boolean-toggle/input/ToggleFlag.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![ManifestAction {
                method: "toggle".to_string(),
                operation: ManifestOperation::Toggle,
                field: "enabled".to_string(),
                operand: None,
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("toggle")
        );
        assert!(manifest_value["components"][0]["actions"][0]
            .get("operand")
            .is_none());
    }

    #[test]
    fn builds_template_manifest_for_multi_step_action_in_source_order() {
        let source =
            include_str!("../../../fixtures/0013-multi-step-action/input/BatchActionCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![
                ManifestAction {
                    method: "apply".to_string(),
                    operation: ManifestOperation::AddAssign,
                    field: "count".to_string(),
                    operand: Some(SerializableValue::Number("2".to_string())),
                },
                ManifestAction {
                    method: "apply".to_string(),
                    operation: ManifestOperation::Decrement,
                    field: "count".to_string(),
                    operand: None,
                },
                ManifestAction {
                    method: "apply".to_string(),
                    operation: ManifestOperation::Assign,
                    field: "count".to_string(),
                    operand: Some(SerializableValue::Number("8".to_string())),
                },
                ManifestAction {
                    method: "apply".to_string(),
                    operation: ManifestOperation::Increment,
                    field: "count".to_string(),
                    operand: None,
                },
                ManifestAction {
                    method: "apply".to_string(),
                    operation: ManifestOperation::Toggle,
                    field: "enabled".to_string(),
                    operand: None,
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        let actions = manifest_value["components"][0]["actions"]
            .as_array()
            .expect("manifest actions should be an array");

        assert_eq!(
            actions
                .iter()
                .map(|action| action["operation"].as_str().expect("operation is a string"))
                .collect::<Vec<_>>(),
            vec!["add_assign", "decrement", "assign", "increment", "toggle"]
        );
        assert_eq!(actions[0]["method"], serde_json::json!("apply"));
        assert_eq!(actions[4]["method"], serde_json::json!("apply"));
        assert!(actions[1].get("operand").is_none());
        assert!(actions[3].get("operand").is_none());
        assert!(actions[4].get("operand").is_none());
    }

    #[test]
    fn builds_template_manifest_for_dynamic_attribute_bindings() {
        let source = include_str!(
            "../../../fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);
        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest.components[0].template.nodes[1],
            ManifestNode::Binding {
                id: "n1".to_string(),
                expression: "this.disabled".to_string(),
                initial_value: Some(SerializableValue::Boolean(false)),
                target: Some(ManifestBindingTarget::Attribute),
                element: Some("n0".to_string()),
                attribute: Some("disabled".to_string()),
            }
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["target"],
            serde_json::json!("attribute")
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["element"],
            serde_json::json!("n0")
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["attribute"],
            serde_json::json!("disabled")
        );
        assert!(manifest_value["components"][0]["template"]["nodes"][3]
            .get("target")
            .is_none());
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
