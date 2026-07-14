use std::collections::BTreeMap;

use crate::{
    BindingTable, ComponentNode, ConsumerId, ContextDesignator, ContextEntity, ContextId,
    ExecutionBoundary, ImportBindingTarget, SemanticId, SemanticOwner, SemanticTypeId,
    SourceProvenance, SymbolKind,
};

/// Canonical resolution of a G3 Consumer's compile-time-only Context designator.
///
/// This deliberately records Context identity only. Provider visibility and
/// selection remain separate compiler products for G4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextResolutionState {
    Resolved(ContextId),
    Unresolved,
}

/// First-class compiler-owned semantic entity for one G3 `@consume()` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerEntity {
    pub id: ConsumerId,
    pub owner: SemanticOwner,
    pub authored_field: SemanticId,
    pub name: String,
    pub context_designator: ContextDesignator,
    pub context_resolution: ContextResolutionState,
    pub requested_type: crate::DeclaredStateType,
    pub requested_type_id: SemanticTypeId,
    pub execution_boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

impl ConsumerEntity {
    #[must_use]
    pub fn context(&self) -> Option<&ContextId> {
        match &self.context_resolution {
            ContextResolutionState::Resolved(context) => Some(context),
            ContextResolutionState::Unresolved => None,
        }
    }
}

/// Lower valid G3 Consumer declarations without resolving any Provider.
#[must_use]
pub fn collect_consumer_entities(
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    bindings: Option<&BindingTable>,
) -> BTreeMap<ConsumerId, ConsumerEntity> {
    let mut consumers = BTreeMap::new();

    for component in components {
        for declaration in &component.consumer_declarations {
            let Some(context) = resolve_context(
                &declaration.context_designator,
                components,
                contexts,
                bindings,
            ) else {
                // The retained declaration candidate carries the unresolved
                // designator fact.  An invalid Consumer must not enter G4-G17.
                continue;
            };
            let id = ConsumerId::for_component(&component.id, &declaration.name);
            let semantic_id = id.as_semantic_id().clone();
            consumers.insert(
                id.clone(),
                ConsumerEntity {
                    requested_type_id: SemanticTypeId::for_subject(&semantic_id),
                    id,
                    owner: SemanticOwner::entity(component.id.clone()),
                    authored_field: declaration.authored_field.clone(),
                    name: declaration.name.clone(),
                    context_designator: declaration.context_designator.clone(),
                    context_resolution: ContextResolutionState::Resolved(context),
                    requested_type: declaration.requested_type.clone(),
                    execution_boundary: ExecutionBoundary::Client,
                    provenance: declaration.provenance.clone(),
                },
            );
        }
    }

    consumers
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
        validate_application_semantic_model, CompilationUnit, ConsumerId, ContextResolutionState,
        SemanticReferenceKind,
    };

    #[test]
    fn lowers_same_module_consumers_without_provider_resolution() {
        let parsed = ezc_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: Theme;
  @consume(AppShell.theme)
  menuTheme!: Theme;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let toolbar = &asm.components[1];
        let consumer = asm
            .consumer(&ConsumerId::for_component(&toolbar.id, "theme"))
            .expect("consumer");

        assert_eq!(asm.consumers().len(), 2);
        assert_eq!(consumer.owner.entity_id(), Some(&toolbar.id));
        assert_eq!(consumer.authored_field, toolbar.id.consumer_field("theme"));
        assert!(matches!(
            consumer.context_resolution,
            ContextResolutionState::Resolved(_)
        ));
        assert_eq!(
            asm.consumers_for_context(consumer.context().unwrap()).len(),
            2
        );
        assert!(asm.references.iter().any(|reference| {
            reference.kind == SemanticReferenceKind::ConsumesContext
                && reference.source == *consumer.id.as_semantic_id()
                && reference.target == *consumer.context().unwrap().as_semantic_id()
        }));
        assert!(asm.providers().is_empty());
        assert!(validate_application_semantic_model(&asm).is_empty());
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
                "src/toolbar.tsx",
                r#"
import { AppShell } from "./app-shell";
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: Theme;
  render() { return <main />; }
}
"#,
            ),
        ]);
        let asm = build_application_semantic_model_for_unit(&unit);

        assert_eq!(asm.consumers().len(), 1);
        assert_eq!(
            asm.consumers()[0].context().unwrap().as_str(),
            "module:src/app-shell.tsx/component:x-app-shell/context:theme"
        );
    }

    #[test]
    fn retains_unresolved_context_designators_as_invalid_candidates() {
        let parsed = ezc_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.notAContext)
  theme!: Theme;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);

        assert!(asm.consumers().is_empty());
        assert_eq!(
            asm.context_declaration_candidates()
                .invalid_candidates()
                .len(),
            1
        );
        assert!(asm.references.is_empty());
    }

    #[test]
    fn excludes_invalid_consumer_forms_and_conflicting_state() {
        let parsed = ezc_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: Theme;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume("theme")
  stringTarget!: Theme;
  @consume(AppShell.theme)
  missingType!;
  @consume(AppShell.theme)
  initialized: Theme = fallbackTheme;
  @consume(AppShell.theme)
  static staticConsumer!: Theme;
  @consume(AppShell.theme)
  declarationOnly: Theme;
  @consume(AppShell.theme)
  stateful: Theme = state("theme");
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);

        assert!(asm.consumers().is_empty());
        assert!(asm.components[1].state_fields.is_empty());
        assert!(asm.references.is_empty());
    }
}
