use std::collections::BTreeMap;

use crate::{
    ComponentNode, ContextId, DeclaredStateType, ExecutionBoundary, ExpressionGraph, SemanticId,
    SemanticOwner, SemanticTypeId, SourceProvenance,
};

/// First-class compiler-owned semantic entity for one G1 `@context()` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEntity {
    pub id: ContextId,
    pub owner: SemanticOwner,
    pub authored_field: SemanticId,
    pub name: String,
    pub declared_type: DeclaredStateType,
    pub declared_type_id: SemanticTypeId,
    pub default_expression: Option<SemanticId>,
    pub execution_boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

/// Lower valid authored Context declarations into stable ASM entities.
#[must_use]
pub fn collect_context_entities(
    components: &[ComponentNode],
    expression_graph: &ExpressionGraph,
) -> BTreeMap<ContextId, ContextEntity> {
    components
        .iter()
        .flat_map(|component| {
            component
                .context_declarations
                .iter()
                .map(move |declaration| {
                    let id = ContextId::for_component(&component.id, &declaration.name);
                    let semantic_id = id.as_semantic_id().clone();
                    (
                        id.clone(),
                        ContextEntity {
                            declared_type_id: SemanticTypeId::for_subject(&semantic_id),
                            default_expression: expression_graph.root_for(&semantic_id).cloned(),
                            id,
                            owner: SemanticOwner::entity(component.id.clone()),
                            authored_field: declaration.authored_field.clone(),
                            name: declaration.name.clone(),
                            declared_type: declaration.declared_type.clone(),
                            execution_boundary: ExecutionBoundary::Client,
                            provenance: declaration.provenance.clone(),
                        },
                    )
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, ContextId, ExecutionBoundary, SemanticOwner};

    #[test]
    fn lowers_typed_context_entities_with_literal_defaults() {
        let parsed = presolve_parser::parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;

  @context()
  locale: string = "en";

  render() { return <main />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let locale_id = ContextId::for_component(&component.id, "locale");
        let locale = asm.context(&locale_id).expect("locale context");

        assert_eq!(asm.contexts().len(), 2);
        assert_eq!(
            locale.id.as_str(),
            "module:src/AppShell.tsx/component:x-app-shell/context:locale"
        );
        assert_eq!(locale.owner, SemanticOwner::entity(component.id.clone()));
        assert_eq!(
            locale.authored_field.as_str(),
            "module:src/AppShell.tsx/component:x-app-shell/context-field:locale"
        );
        assert_eq!(locale.declared_type.text, "string");
        assert_eq!(locale.execution_boundary, ExecutionBoundary::Client);
        assert!(locale.default_expression.is_some());
        assert_eq!(
            asm.expression_owner(locale.default_expression.as_ref().unwrap()),
            Some(locale.id.as_semantic_id())
        );
        assert!(component.state_fields.is_empty());
    }

    #[test]
    fn retains_invalid_context_candidates_for_g18_diagnostics() {
        let parsed = presolve_parser::parse_file(
            "src/InvalidContexts.tsx",
            r#"
@component("x-invalid-contexts")
class InvalidContexts extends Component {
  @context("theme")
  argument: string;

  @context()
  missingType;

  @context()
  static staticField: string;

  @context()
  nonliteral: string = createLocale();

  @context()
  get accessor(): string { return "en"; }

  render() { return <main />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);

        assert_eq!(asm.contexts().len(), 1);
        assert_eq!(asm.contexts()[0].name, "staticField");
        assert!(asm.components[0].state_fields.is_empty());
        assert_eq!(
            asm.context_declaration_candidates()
                .invalid_candidates()
                .len(),
            4
        );
        assert_eq!(
            asm.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
            vec!["PSC1052", "PSC1052", "PSC1052", "PSC1052"]
        );
    }

    #[test]
    fn accepts_static_contexts_and_qualified_string_designators() {
        let parsed = presolve_parser::parse_file(
            "src/Theme.tsx",
            r#"
@component("x-theme")
class Theme extends Component {
  @context()
  static mode: string = "light";

  @provide("Theme.mode")
  providedMode: string = "dark";

  @consume("Theme.mode")
  consumedMode!: string;

  render() { return <main />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];

        assert_eq!(asm.contexts().len(), 1);
        assert_eq!(component.provider_declarations.len(), 1);
        assert_eq!(component.consumer_declarations.len(), 1);
        assert_eq!(
            component.provider_declarations[0]
                .context_designator
                .component_symbol,
            "Theme"
        );
        assert_eq!(
            component.consumer_declarations[0]
                .context_designator
                .context_member,
            "mode"
        );
        assert!(asm.diagnostics.is_empty(), "{:#?}", asm.diagnostics);
    }

    #[test]
    fn keeps_same_context_names_distinct_by_component() {
        let parsed = presolve_parser::parse_file(
            "src/Contexts.tsx",
            r#"
@component("x-left")
class Left extends Component {
  @context()
  theme: string;
  render() { return <main />; }
}

@component("x-right")
class Right extends Component {
  @context()
  theme: string;
  render() { return <main />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);

        assert_eq!(asm.contexts().len(), 2);
        assert_ne!(asm.contexts()[0].id, asm.contexts()[1].id);
        assert!(asm.contexts().iter().all(|context| context.name == "theme"));
    }

    #[test]
    fn asm_validation_accepts_context_ownership_type_and_default_contracts() {
        let source = r#"
@component("x-context-validation")
class ContextValidation extends Component {
  @context()
  locale: string = "en";
  render() { return <main />; }
}
"#;
        let parsed = presolve_parser::parse_file("src/ContextValidation.tsx", source);
        let asm = build_application_semantic_model(&parsed);
        let context = asm.contexts()[0];

        assert_eq!(
            asm.semantic_type_of(context.id.as_semantic_id()),
            Some(&crate::SemanticType::String)
        );
        assert_eq!(
            context.provenance.span.start,
            source.find("@context()").unwrap()
        );
        let diagnostics = crate::validate_application_semantic_model(&asm);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }
}
