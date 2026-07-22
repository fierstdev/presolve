use std::collections::BTreeMap;

use presolve_parser::ParsedFile;

use crate::{
    BindingTable, ComponentInvocationId, ComponentNode, ImportBindingTarget, SemanticId,
    SourceProvenance, SymbolKind, TemplateNode, TemplatePositionId, TemplateSemanticEntity,
    TemplateSemanticKind,
};

/// One immutable compiler-owned use of a component definition in a caller template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInvocationEntity {
    pub id: ComponentInvocationId,
    pub owner_component: SemanticId,
    pub target_component: Option<SemanticId>,
    pub authored_symbol: String,
    pub template_entity: SemanticId,
    pub source_position: TemplatePositionId,
    pub status: ComponentInvocationResolutionStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentInvocationResolutionStatus {
    Resolved,
    UnresolvedSymbol,
    ResolvedNonComponent,
    Ambiguous,
    UnsupportedDynamicTarget,
}

/// Lower `PascalCase` component uses from canonical template traversal and binding facts.
#[must_use]
pub fn collect_component_invocations(
    components: &[ComponentNode],
    templates: &[TemplateNode],
    template_entities: &[TemplateSemanticEntity],
    files: &[ParsedFile],
    bindings: Option<&BindingTable>,
    component_provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> BTreeMap<ComponentInvocationId, ComponentInvocationEntity> {
    template_entities
        .iter()
        .filter(|entity| entity.kind == TemplateSemanticKind::Element)
        .filter_map(|entity| {
            let authored_symbol = entity.tag_name.as_ref()?;
            if !is_pascal_identifier(authored_symbol)
                && !looks_like_dynamic_component_target(authored_symbol)
            {
                return None;
            }

            let template_id = entity.owner.entity_id()?;
            let template = templates
                .iter()
                .find(|template| template.id == *template_id)?;
            let owner_component = template.owner.entity_id()?.clone();
            let (target_component, status) = resolve_invocation_target(
                authored_symbol,
                &entity.provenance.path,
                components,
                files,
                bindings,
                component_provenance,
            );
            let identity_target = target_component.as_ref().map_or_else(
                || format!("{}:{}", status_identity_prefix(status), authored_symbol),
                |target| target.as_str().to_string(),
            );
            let id = ComponentInvocationId::for_template_entity(&entity.id, &identity_target);
            let source_position = TemplatePositionId::for_template_entity(&entity.id);
            let provenance = entity
                .tag_name_provenance
                .clone()
                .unwrap_or_else(|| entity.provenance.clone());

            Some((
                id.clone(),
                ComponentInvocationEntity {
                    id,
                    owner_component,
                    target_component,
                    authored_symbol: authored_symbol.clone(),
                    template_entity: entity.id.clone(),
                    source_position,
                    status,
                    provenance,
                },
            ))
        })
        .collect()
}

fn resolve_invocation_target(
    authored_symbol: &str,
    module_path: &std::path::Path,
    components: &[ComponentNode],
    files: &[ParsedFile],
    bindings: Option<&BindingTable>,
    component_provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> (Option<SemanticId>, ComponentInvocationResolutionStatus) {
    if !is_pascal_identifier(authored_symbol) {
        return (
            None,
            ComponentInvocationResolutionStatus::UnsupportedDynamicTarget,
        );
    }

    let local_components = components
        .iter()
        .filter(|component| {
            component.class_name == authored_symbol
                && component.element_name.is_some()
                && component_provenance
                    .get(&component.id)
                    .is_some_and(|provenance| provenance.path == module_path)
        })
        .collect::<Vec<_>>();
    if local_components.len() > 1 {
        return (None, ComponentInvocationResolutionStatus::Ambiguous);
    }
    if let Some(component) = local_components.first() {
        return (
            Some(component.id.clone()),
            ComponentInvocationResolutionStatus::Resolved,
        );
    }

    if let Some(binding) =
        bindings.and_then(|bindings| bindings.resolve_import(module_path, authored_symbol))
    {
        return match &binding.target {
            ImportBindingTarget::Symbol(symbol)
                if symbol.kind == SymbolKind::Component
                    && components.iter().any(|component| {
                        component.id == symbol.id && component.element_name.is_some()
                    }) =>
            {
                (
                    Some(symbol.id.clone()),
                    ComponentInvocationResolutionStatus::Resolved,
                )
            }
            ImportBindingTarget::Symbol(_)
            | ImportBindingTarget::Namespace { .. }
            | ImportBindingTarget::SemanticPackage { .. } => (
                None,
                ComponentInvocationResolutionStatus::ResolvedNonComponent,
            ),
        };
    }

    let is_local_non_component = files
        .iter()
        .filter(|file| file.path == module_path)
        .any(|file| {
            file.type_aliases
                .iter()
                .any(|alias| alias.name == authored_symbol)
                || file.classes.iter().any(|class| {
                    class.name == authored_symbol
                        && !local_components
                            .iter()
                            .any(|component| component.class_name == class.name)
                })
        });
    if is_local_non_component {
        return (
            None,
            ComponentInvocationResolutionStatus::ResolvedNonComponent,
        );
    }

    (None, ComponentInvocationResolutionStatus::UnresolvedSymbol)
}

fn is_pascal_identifier(symbol: &str) -> bool {
    symbol
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        && symbol
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '$'))
}

fn looks_like_dynamic_component_target(symbol: &str) -> bool {
    let Some(final_segment) = symbol.rsplit(['.', ':']).next() else {
        return false;
    };
    (symbol.contains('.') || symbol.contains(':'))
        && final_segment
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
}

const fn status_identity_prefix(status: ComponentInvocationResolutionStatus) -> &'static str {
    match status {
        ComponentInvocationResolutionStatus::Resolved => "resolved",
        ComponentInvocationResolutionStatus::UnresolvedSymbol => "unresolved",
        ComponentInvocationResolutionStatus::ResolvedNonComponent => "non-component",
        ComponentInvocationResolutionStatus::Ambiguous => "ambiguous",
        ComponentInvocationResolutionStatus::UnsupportedDynamicTarget => "dynamic",
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, validate_application_semantic_model, CompilationUnit,
        ComponentInvocationResolutionStatus, SemanticEntityKind,
    };

    #[test]
    fn resolves_local_repeated_invocations_with_distinct_stable_identities() {
        let source = r#"
@component("x-card") class Card extends Component {
  render() { return <article />; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Card /><Card /></main>; }
}
"#;
        let first =
            build_application_semantic_model(&presolve_parser::parse_file("src/Page.tsx", source));
        let second = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Page.tsx",
            &source.replace(
                "render() { return <main>",
                "title = state(\"x\"); render() { return <main>",
            ),
        ));
        let invocations = first.component_invocations();

        assert_eq!(invocations.len(), 2);
        assert!(invocations.iter().all(|invocation| {
            invocation.status == ComponentInvocationResolutionStatus::Resolved
                && invocation
                    .target_component
                    .as_ref()
                    .is_some_and(|target| target.as_str().ends_with("component:x-card"))
                && invocation
                    .owner_component
                    .as_str()
                    .ends_with("component:x-page")
        }));
        assert_ne!(invocations[0].id, invocations[1].id);
        assert_ne!(
            invocations[0].source_position,
            invocations[1].source_position
        );
        assert_eq!(
            first
                .component_invocations
                .keys()
                .map(crate::ComponentInvocationId::as_str)
                .collect::<Vec<_>>(),
            second
                .component_invocations
                .keys()
                .map(crate::ComponentInvocationId::as_str)
                .collect::<Vec<_>>()
        );
        assert!(first
            .entity(invocations[0].id.as_semantic_id())
            .is_some_and(|entity| entity.kind() == SemanticEntityKind::ComponentInvocation));
        assert!(
            validate_application_semantic_model(&first).is_empty(),
            "canonical component invocations should pass ASM validation"
        );
        let graph = build_semantic_graph(&first);
        assert_eq!(graph.schema_version, 6);
        assert!(graph.nodes.iter().all(|node| {
            first
                .component_invocations
                .keys()
                .all(|invocation| node.id != *invocation.as_semantic_id())
        }));
    }

    #[test]
    fn resolves_import_aliases_and_retains_every_blocked_resolution_status() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/Card.tsx",
                r#"
@component("x-card")
export class Card extends Component {
  render() { return <article />; }
}
export type Model = string;
"#,
            ),
            (
                "src/Page.tsx",
                r#"
import { Card as ImportedCard, Model } from "./Card";
class Utility {}
@component("x-page") class Page extends Component {
  render() {
    return <><ImportedCard /><Missing /><Model /><Utility /><Registry.Card /></>;
  }
}
"#,
            ),
        ]);
        let asm = build_application_semantic_model_for_unit(&unit);
        let by_symbol = asm
            .component_invocations()
            .into_iter()
            .map(|invocation| (invocation.authored_symbol.as_str(), invocation))
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_eq!(
            by_symbol["ImportedCard"].status,
            ComponentInvocationResolutionStatus::Resolved
        );
        assert_eq!(
            by_symbol["Missing"].status,
            ComponentInvocationResolutionStatus::UnresolvedSymbol
        );
        assert_eq!(
            by_symbol["Model"].status,
            ComponentInvocationResolutionStatus::ResolvedNonComponent
        );
        assert_eq!(
            by_symbol["Utility"].status,
            ComponentInvocationResolutionStatus::ResolvedNonComponent
        );
        assert_eq!(
            by_symbol["Registry.Card"].status,
            ComponentInvocationResolutionStatus::UnsupportedDynamicTarget
        );
        assert!(by_symbol["ImportedCard"]
            .target_component
            .as_ref()
            .is_some_and(|target| target.as_str().ends_with("component:x-card")));
        assert_eq!(
            by_symbol["ImportedCard"].provenance.span.end
                - by_symbol["ImportedCard"].provenance.span.start,
            "ImportedCard".len()
        );
        assert!(by_symbol["Missing"].target_component.is_none());
    }

    #[test]
    fn retains_ambiguous_local_component_resolution_without_selecting_a_target() {
        let parsed = presolve_parser::parse_file(
            "src/Ambiguous.tsx",
            r#"
@component("x-card-a") class Card extends Component { render() { return <a />; } }
@component("x-card-b") class Card extends Component { render() { return <b />; } }
@component("x-page") class Page extends Component { render() { return <Card />; } }
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let invocation = asm
            .component_invocations()
            .into_iter()
            .find(|invocation| invocation.authored_symbol == "Card")
            .expect("ambiguous invocation candidate");

        assert_eq!(
            invocation.status,
            ComponentInvocationResolutionStatus::Ambiguous
        );
        assert!(invocation.target_component.is_none());
        assert!(invocation.id.as_str().contains("ambiguous:Card"));
    }

    #[test]
    fn invocation_order_is_deterministic_across_input_file_order() {
        let sources = [
            (
                "src/Card.tsx",
                "@component(\"x-card\")\nexport class Card extends Component { render() { return <article />; } }",
            ),
            (
                "src/Page.tsx",
                "import { Card } from \"./Card\"; @component(\"x-page\") class Page extends Component { render() { return <Card />; } }",
            ),
        ];
        let forward =
            build_application_semantic_model_for_unit(&CompilationUnit::parse_sources(sources));
        let reverse = build_application_semantic_model_for_unit(&CompilationUnit::parse_sources([
            sources[1], sources[0],
        ]));

        assert_eq!(forward.component_invocations, reverse.component_invocations);
    }
}
