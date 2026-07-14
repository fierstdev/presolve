use std::collections::BTreeMap;

use crate::{
    BindingTable, ComponentNode, ContextDesignator, ContextEntity, ContextId, ExecutionBoundary,
    ExpressionGraph, ImportBindingTarget, ProviderId, SemanticId, SemanticOwner, SemanticTypeId,
    SourceProvenance, SymbolKind,
};

/// First-class compiler-owned semantic entity for one G2 `@provide()` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEntity {
    pub id: ProviderId,
    pub owner: SemanticOwner,
    pub authored_field: SemanticId,
    pub name: String,
    pub context_designator: ContextDesignator,
    pub context: ContextId,
    pub declared_type: crate::DeclaredStateType,
    pub declared_type_id: SemanticTypeId,
    pub value_expression: SemanticId,
    pub execution_boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

/// A retained G2 declaration fact for a duplicate provider target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateProviderDeclaration {
    pub owner: SemanticId,
    pub context: ContextId,
    pub retained_provider: ProviderId,
    pub rejected_field: SemanticId,
    pub provenance: SourceProvenance,
}

/// Lower valid, uniquely-targeted Provider declarations into canonical entities.
#[must_use]
pub fn collect_provider_entities(
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    expression_graph: &ExpressionGraph,
    bindings: Option<&BindingTable>,
) -> (
    BTreeMap<ProviderId, ProviderEntity>,
    Vec<DuplicateProviderDeclaration>,
) {
    let mut providers = BTreeMap::new();
    let mut duplicates = Vec::new();
    let mut declarations = Vec::new();

    for component in components {
        for declaration in &component.provider_declarations {
            let Some(context) = resolve_context(
                &declaration.context_designator,
                components,
                contexts,
                bindings,
            ) else {
                continue;
            };
            let id = ProviderId::for_component(&component.id, &declaration.name);
            let semantic_id = id.as_semantic_id().clone();
            let Some(value_expression) = expression_graph.root_for(&semantic_id).cloned() else {
                continue;
            };
            declarations.push((component, declaration, context, id, value_expression));
        }
    }
    let mut counts = BTreeMap::<(SemanticId, ContextId), usize>::new();
    for (component, _, context, _, _) in &declarations {
        *counts
            .entry((component.id.clone(), context.clone()))
            .or_default() += 1;
    }
    for (component, declaration, context, id, value_expression) in declarations {
        if counts[&(component.id.clone(), context.clone())] > 1 {
            duplicates.push(DuplicateProviderDeclaration {
                owner: component.id.clone(),
                context,
                retained_provider: id,
                rejected_field: declaration.authored_field.clone(),
                provenance: declaration.provenance.clone(),
            });
            continue;
        }
        let semantic_id = id.as_semantic_id().clone();
        providers.insert(
            id.clone(),
            ProviderEntity {
                declared_type_id: SemanticTypeId::for_subject(&semantic_id),
                id,
                owner: SemanticOwner::entity(component.id.clone()),
                authored_field: declaration.authored_field.clone(),
                name: declaration.name.clone(),
                context_designator: declaration.context_designator.clone(),
                context,
                declared_type: declaration.declared_type.clone(),
                value_expression,
                execution_boundary: ExecutionBoundary::Client,
                provenance: declaration.provenance.clone(),
            },
        );
    }

    (providers, duplicates)
}

fn resolve_context(
    designator: &ContextDesignator,
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    bindings: Option<&BindingTable>,
) -> Option<ContextId> {
    let local_component = components
        .iter()
        .find(|component| {
            component.class_name == designator.component_symbol
                && contexts.values().any(|context| {
                    context.owner.entity_id() == Some(&component.id)
                        && context.provenance.path == designator.provenance.path
                })
        })
        .map(|component| component.id.clone());
    let imported_component = bindings
        .and_then(|bindings| {
            bindings.resolve_import(&designator.provenance.path, &designator.component_symbol)
        })
        .and_then(|binding| match &binding.target {
            ImportBindingTarget::Symbol(symbol) if symbol.kind == SymbolKind::Component => {
                Some(symbol.id.clone())
            }
            ImportBindingTarget::Symbol(_) | ImportBindingTarget::Namespace { .. } => None,
        });
    let component = local_component.or(imported_component)?;
    contexts
        .values()
        .find(|context| {
            context.owner.entity_id() == Some(&component)
                && context.name == designator.context_member
        })
        .map(|context| context.id.clone())
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        CompilationUnit, ContextId, ProviderId, SemanticReferenceKind,
    };

    #[test]
    fn lowers_same_component_provider_with_one_context_relation() {
        let parsed = ezc_parser::parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  @provide(AppShell.theme)
  providedTheme: Theme = this.selectedTheme;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let id = ProviderId::for_component(&component.id, "providedTheme");
        let provider = asm.provider(&id).expect("provider");

        assert_eq!(asm.providers().len(), 1);
        assert_eq!(
            provider.context,
            ContextId::for_component(&component.id, "theme")
        );
        assert_eq!(
            asm.expression_owner(&provider.value_expression),
            Some(id.as_semantic_id())
        );
        assert!(asm.references.iter().any(|reference| {
            reference.kind == SemanticReferenceKind::ProvidesContext
                && reference.source == *id.as_semantic_id()
                && reference.target == *provider.context.as_semantic_id()
        }));
        assert_eq!(
            asm.semantic_type_of(provider.id.as_semantic_id()),
            Some(&crate::SemanticType::Unknown)
        );
        let diagnostics = crate::validate_application_semantic_model(&asm);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn resolves_imported_context_owners_without_runtime_lookup() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/app-shell.tsx",
                r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  render() { return <main />; }
}
export { AppShell };
"#,
            ),
            (
                "src/theme-boundary.tsx",
                r#"
import { AppShell } from "./app-shell";
@component("x-theme-boundary")
class ThemeBoundary extends Component {
  @provide(AppShell.theme)
  providedTheme: Theme = this.theme;
  render() { return <main />; }
}
"#,
            ),
        ]);
        let asm = build_application_semantic_model_for_unit(&unit);

        assert_eq!(asm.providers().len(), 1);
        assert_eq!(
            asm.providers()[0].context.as_str(),
            "module:src/app-shell.tsx/component:x-app-shell/context:theme"
        );
    }

    #[test]
    fn retains_duplicate_providers_without_selecting_an_executable_winner() {
        let parsed = ezc_parser::parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme: string;
  @provide(AppShell.theme)
  primary: string = "one";
  @provide(AppShell.theme)
  secondary: string = "two";
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);

        assert!(asm.providers().is_empty());
        assert_eq!(asm.duplicate_provider_declarations.len(), 2);
        assert_eq!(
            asm.context_declaration_candidates()
                .invalid_candidates()
                .len(),
            2
        );
    }

    #[test]
    fn excludes_invalid_provider_forms_without_creating_runtime_facts() {
        let parsed = ezc_parser::parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme: string;

  @provide("theme")
  stringTarget: string = "light";

  @provide(AppShell.theme)
  missingType = "light";

  @provide(AppShell.theme)
  missingValue!: string;

  @provide(AppShell.theme)
  static staticProvider: string = "light";

  @provide(AppShell.theme)
  callProvider: string = createTheme();

  @provide(AppShell.notAContext)
  wrongTarget: string = "light";

  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);

        assert_eq!(asm.contexts().len(), 1);
        assert!(asm.providers().is_empty());
        assert!(asm.references.is_empty());
        assert!(asm.components[0].state_fields.is_empty());
    }
}
