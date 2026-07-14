use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ComponentNode, ConsumerEntity, ConsumerId, ContextEntity, ContextId, ExpressionGraph,
    ProviderEntity, ProviderId, SemanticId, SourceProvenance,
};

/// Typed node identity for the immutable G6 Context ownership projection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextOwnershipNodeId {
    Component(SemanticId),
    Context(ContextId),
    Provider(ProviderId),
    Consumer(ConsumerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextOwnershipNodeKind {
    Component,
    Context,
    Provider,
    Consumer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextOwnershipNode {
    pub id: ContextOwnershipNodeId,
    pub kind: ContextOwnershipNodeKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextOwnershipOwnerId {
    Component(SemanticId),
    Context(ContextId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextOwnershipTargetId {
    Context(ContextId),
    Provider(ProviderId),
    Consumer(ConsumerId),
    Expression(SemanticId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextOwnershipEdgeKind {
    ComponentOwnsContext,
    ComponentOwnsProvider,
    ComponentOwnsConsumer,
    ContextOwnsDefaultExpression,
}

/// One directed ownership fact. Edges always point owner to owned target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextOwnershipEdge {
    pub owner: ContextOwnershipOwnerId,
    pub owned: ContextOwnershipTargetId,
    pub kind: ContextOwnershipEdgeKind,
    pub provenance: SourceProvenance,
}

/// Retained component-local ownership groups for later read-only consumers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextOwnedEntities {
    pub contexts: Vec<ContextId>,
    pub providers: Vec<ProviderId>,
    pub consumers: Vec<ConsumerId>,
}

/// Immutable G6 projection of canonical Context semantic ownership.
///
/// It deliberately does not contain component ancestry, Context request or
/// Provider selection relations. Those remain owned by G4 products.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextOwnershipGraph {
    pub nodes: Vec<ContextOwnershipNode>,
    pub edges: Vec<ContextOwnershipEdge>,
    contexts_by_component: BTreeMap<SemanticId, Vec<ContextId>>,
    providers_by_component: BTreeMap<SemanticId, Vec<ProviderId>>,
    consumers_by_component: BTreeMap<SemanticId, Vec<ConsumerId>>,
    context_owner: BTreeMap<ContextId, SemanticId>,
    provider_owner: BTreeMap<ProviderId, SemanticId>,
    consumer_owner: BTreeMap<ConsumerId, SemanticId>,
    default_by_context: BTreeMap<ContextId, SemanticId>,
}

impl ContextOwnershipGraph {
    #[must_use]
    pub fn owned_contexts(&self, component: &SemanticId) -> &[ContextId] {
        self.contexts_by_component
            .get(component)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn owned_providers(&self, component: &SemanticId) -> &[ProviderId] {
        self.providers_by_component
            .get(component)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn owned_consumers(&self, component: &SemanticId) -> &[ConsumerId] {
        self.consumers_by_component
            .get(component)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn owner_of_context(&self, context: &ContextId) -> Option<&SemanticId> {
        self.context_owner.get(context)
    }

    #[must_use]
    pub fn owner_of_provider(&self, provider: &ProviderId) -> Option<&SemanticId> {
        self.provider_owner.get(provider)
    }

    #[must_use]
    pub fn owner_of_consumer(&self, consumer: &ConsumerId) -> Option<&SemanticId> {
        self.consumer_owner.get(consumer)
    }

    #[must_use]
    pub fn context_default_expression(&self, context: &ContextId) -> Option<&SemanticId> {
        self.default_by_context.get(context)
    }

    #[must_use]
    pub fn context_owned_entities(&self, component: &SemanticId) -> ContextOwnedEntities {
        ContextOwnedEntities {
            contexts: self.owned_contexts(component).to_vec(),
            providers: self.owned_providers(component).to_vec(),
            consumers: self.owned_consumers(component).to_vec(),
        }
    }
}

/// Projects canonical G1--G3 ownership facts without interpreting source,
/// scope ancestry, resolution, compatibility, or runtime state.
///
/// # Panics
///
/// Panics when a canonical component owner lacks source provenance.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn collect_context_ownership_graph(
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    expression_graph: &ExpressionGraph,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> ContextOwnershipGraph {
    let owner_components = contexts
        .values()
        .filter_map(|context| context.owner.entity_id().cloned())
        .chain(
            providers
                .values()
                .filter_map(|provider| provider.owner.entity_id().cloned()),
        )
        .chain(
            consumers
                .values()
                .filter_map(|consumer| consumer.owner.entity_id().cloned()),
        )
        .collect::<BTreeSet<_>>();
    let mut nodes = components
        .iter()
        .filter(|component| owner_components.contains(&component.id))
        .map(|component| ContextOwnershipNode {
            id: ContextOwnershipNodeId::Component(component.id.clone()),
            kind: ContextOwnershipNodeKind::Component,
            provenance: provenance
                .get(&component.id)
                .cloned()
                .expect("canonical component owners should retain source provenance"),
        })
        .collect::<Vec<_>>();
    let mut edges = Vec::new();
    let mut contexts_by_component = BTreeMap::new();
    let mut providers_by_component = BTreeMap::new();
    let mut consumers_by_component = BTreeMap::new();
    let mut context_owner = BTreeMap::new();
    let mut provider_owner = BTreeMap::new();
    let mut consumer_owner = BTreeMap::new();
    let mut default_by_context = BTreeMap::new();

    for context in contexts.values() {
        let Some(owner) = context.owner.entity_id().cloned() else {
            continue;
        };
        nodes.push(ContextOwnershipNode {
            id: ContextOwnershipNodeId::Context(context.id.clone()),
            kind: ContextOwnershipNodeKind::Context,
            provenance: context.provenance.clone(),
        });
        contexts_by_component
            .entry(owner.clone())
            .or_insert_with(Vec::new)
            .push(context.id.clone());
        context_owner.insert(context.id.clone(), owner.clone());
        edges.push(ContextOwnershipEdge {
            owner: ContextOwnershipOwnerId::Component(owner),
            owned: ContextOwnershipTargetId::Context(context.id.clone()),
            kind: ContextOwnershipEdgeKind::ComponentOwnsContext,
            provenance: context.provenance.clone(),
        });
        if let Some(expression) = &context.default_expression {
            default_by_context.insert(context.id.clone(), expression.clone());
            edges.push(ContextOwnershipEdge {
                owner: ContextOwnershipOwnerId::Context(context.id.clone()),
                owned: ContextOwnershipTargetId::Expression(expression.clone()),
                kind: ContextOwnershipEdgeKind::ContextOwnsDefaultExpression,
                provenance: expression_graph.node(expression).map_or_else(
                    || context.provenance.clone(),
                    |node| node.provenance.clone(),
                ),
            });
        }
    }
    for provider in providers.values() {
        let Some(owner) = provider.owner.entity_id().cloned() else {
            continue;
        };
        nodes.push(ContextOwnershipNode {
            id: ContextOwnershipNodeId::Provider(provider.id.clone()),
            kind: ContextOwnershipNodeKind::Provider,
            provenance: provider.provenance.clone(),
        });
        providers_by_component
            .entry(owner.clone())
            .or_insert_with(Vec::new)
            .push(provider.id.clone());
        provider_owner.insert(provider.id.clone(), owner.clone());
        edges.push(ContextOwnershipEdge {
            owner: ContextOwnershipOwnerId::Component(owner),
            owned: ContextOwnershipTargetId::Provider(provider.id.clone()),
            kind: ContextOwnershipEdgeKind::ComponentOwnsProvider,
            provenance: provider.provenance.clone(),
        });
    }
    for consumer in consumers.values() {
        let Some(owner) = consumer.owner.entity_id().cloned() else {
            continue;
        };
        nodes.push(ContextOwnershipNode {
            id: ContextOwnershipNodeId::Consumer(consumer.id.clone()),
            kind: ContextOwnershipNodeKind::Consumer,
            provenance: consumer.provenance.clone(),
        });
        consumers_by_component
            .entry(owner.clone())
            .or_insert_with(Vec::new)
            .push(consumer.id.clone());
        consumer_owner.insert(consumer.id.clone(), owner.clone());
        edges.push(ContextOwnershipEdge {
            owner: ContextOwnershipOwnerId::Component(owner),
            owned: ContextOwnershipTargetId::Consumer(consumer.id.clone()),
            kind: ContextOwnershipEdgeKind::ComponentOwnsConsumer,
            provenance: consumer.provenance.clone(),
        });
    }
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    edges.sort_by(|left, right| {
        (&left.owner, left.kind, &left.owned).cmp(&(&right.owner, right.kind, &right.owned))
    });
    ContextOwnershipGraph {
        nodes,
        edges,
        contexts_by_component,
        providers_by_component,
        consumers_by_component,
        context_owner,
        provider_owner,
        consumer_owner,
        default_by_context,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        validate_application_semantic_model, CompilationUnit, ConsumerId, ContextOwnershipEdgeKind,
        ContextOwnershipTargetId, ProviderId,
    };

    #[test]
    fn projects_foreign_context_users_by_declaring_component() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/app-shell.tsx",
                r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme: string = "light";
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
  providedTheme: string = "dark";
  render() { return <main />; }
}
"#,
            ),
            (
                "src/toolbar.tsx",
                r#"
import { AppShell } from "./app-shell";
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: string;
  render() { return <main />; }
}
"#,
            ),
        ]);
        let asm = build_application_semantic_model_for_unit(&unit);
        let graph = &asm.context_ownership;
        let app_shell = &asm.components[0].id;
        let boundary = &asm.components[1].id;
        let toolbar = &asm.components[2].id;
        let context = asm.contexts()[0].id.clone();
        let provider = ProviderId::for_component(boundary, "providedTheme");
        let consumer = ConsumerId::for_component(toolbar, "theme");

        assert_eq!(graph.owner_of_context(&context), Some(app_shell));
        assert_eq!(graph.owner_of_provider(&provider), Some(boundary));
        assert_eq!(graph.owner_of_consumer(&consumer), Some(toolbar));
        assert_eq!(graph.owned_providers(app_shell), &[]);
        assert_eq!(graph.owned_consumers(app_shell), &[]);
        assert!(graph.context_default_expression(&context).is_some());
        assert!(graph.edges.iter().all(|edge| !matches!(
            edge.owned,
            ContextOwnershipTargetId::Expression(_) if edge.kind != ContextOwnershipEdgeKind::ContextOwnsDefaultExpression
        )));
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn retains_unresolved_consumers_and_is_deterministic() {
        let parsed = ezc_parser::parse_file(
            "src/toolbar.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: string;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume(AppShell.theme)
  theme!: string;
  render() { return <main />; }
}
"#,
        );
        let first = build_application_semantic_model(&parsed);
        let second = build_application_semantic_model(&parsed);
        let consumer = ConsumerId::for_component(&first.components[1].id, "theme");

        assert_eq!(first.context_ownership, second.context_ownership);
        assert_eq!(
            first.context_ownership.owner_of_consumer(&consumer),
            Some(&first.components[1].id)
        );
        assert_eq!(first.context_ownership.edges.len(), 2);
        let diagnostics = validate_application_semantic_model(&first);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }
}
