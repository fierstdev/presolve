use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CompatibilityStatus, ComponentNode, ComputedValue, ConsumerEntity, ConsumerId,
    ContextBindingCompatibility, ContextBindingTypeRecord, ContextEntity, ContextId,
    ContextResolution, ContextResolutionResult, ContextTypeRecord, ExpressionGraph,
    ExpressionNodeKind, ProviderEntity, ProviderId, ProviderTypeRecord, SemanticId,
    SourceProvenance,
};

/// Typed direct-value node identity for the immutable G7 dependency graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextDependencyNodeId {
    State(SemanticId),
    Computed(SemanticId),
    Context(ContextId),
    ContextDefault(ContextId),
    Provider(ProviderId),
    Consumer(ConsumerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextDependencyNodeKind {
    State,
    Computed,
    Context,
    ContextDefault,
    Provider,
    Consumer,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextDependencyNode {
    pub id: ContextDependencyNodeId,
    pub kind: ContextDependencyNodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextDependencyEdgeKind {
    ProviderReadsState,
    ProviderReadsComputed,
    ContextDefaultReadsState,
    ContextDefaultReadsComputed,
    ProviderSuppliesContext,
    ContextDefaultSuppliesContext,
    ConsumerReadsProvider,
    ConsumerReadsContextDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDependencyCompatibility {
    Compatible,
    Incompatible,
    Unknown,
    NotApplicable,
}

/// One direct, compiler-resolved value-flow relation. The edge points from the
/// dependent product to the prerequisite product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDependencyEdge {
    pub dependent: ContextDependencyNodeId,
    pub dependency: ContextDependencyNodeId,
    pub kind: ContextDependencyEdgeKind,
    pub compatibility: ContextDependencyCompatibility,
    pub provenance: SourceProvenance,
}

/// Immutable compiler-owned direct Context dependency topology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDependencyGraph {
    pub nodes: Vec<ContextDependencyNode>,
    pub edges: Vec<ContextDependencyEdge>,
    dependencies_by_node: BTreeMap<ContextDependencyNodeId, Vec<ContextDependencyNodeId>>,
    dependents_by_node: BTreeMap<ContextDependencyNodeId, Vec<ContextDependencyNodeId>>,
    provider_value_dependencies: BTreeMap<ProviderId, Vec<ContextDependencyNodeId>>,
    context_default_dependencies: BTreeMap<ContextId, Vec<ContextDependencyNodeId>>,
    consumer_binding_dependencies: BTreeMap<ConsumerId, ContextDependencyNodeId>,
    consumers_by_provider: BTreeMap<ProviderId, Vec<ConsumerId>>,
    consumers_by_default: BTreeMap<ContextId, Vec<ConsumerId>>,
}

impl ContextDependencyGraph {
    #[must_use]
    pub fn direct_dependencies(
        &self,
        node: &ContextDependencyNodeId,
    ) -> &[ContextDependencyNodeId] {
        self.dependencies_by_node
            .get(node)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn direct_dependents(&self, node: &ContextDependencyNodeId) -> &[ContextDependencyNodeId] {
        self.dependents_by_node.get(node).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn provider_value_dependencies(&self, provider: &ProviderId) -> &[ContextDependencyNodeId] {
        self.provider_value_dependencies
            .get(provider)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn context_default_dependencies(&self, context: &ContextId) -> &[ContextDependencyNodeId] {
        self.context_default_dependencies
            .get(context)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn consumer_binding_dependency(
        &self,
        consumer: &ConsumerId,
    ) -> Option<&ContextDependencyNodeId> {
        self.consumer_binding_dependencies.get(consumer)
    }

    #[must_use]
    pub fn consumers_bound_to_provider(&self, provider: &ProviderId) -> &[ConsumerId] {
        self.consumers_by_provider
            .get(provider)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn consumers_bound_to_default(&self, context: &ContextId) -> &[ConsumerId] {
        self.consumers_by_default
            .get(context)
            .map_or(&[], Vec::as_slice)
    }
}

/// Projects canonical G1--G5 products into direct Context value-flow facts.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[must_use]
pub fn collect_context_dependency_graph(
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    resolutions: &BTreeMap<ConsumerId, ContextResolution>,
    context_types: &BTreeMap<ContextId, ContextTypeRecord>,
    provider_types: &BTreeMap<ProviderId, ProviderTypeRecord>,
    binding_types: &BTreeMap<ConsumerId, ContextBindingTypeRecord>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
) -> ContextDependencyGraph {
    let mut nodes = contexts
        .keys()
        .cloned()
        .map(|id| ContextDependencyNode {
            id: ContextDependencyNodeId::Context(id),
            kind: ContextDependencyNodeKind::Context,
        })
        .chain(contexts.values().filter_map(|context| {
            context
                .default_expression
                .as_ref()
                .map(|_| ContextDependencyNode {
                    id: ContextDependencyNodeId::ContextDefault(context.id.clone()),
                    kind: ContextDependencyNodeKind::ContextDefault,
                })
        }))
        .chain(providers.keys().cloned().map(|id| ContextDependencyNode {
            id: ContextDependencyNodeId::Provider(id),
            kind: ContextDependencyNodeKind::Provider,
        }))
        .chain(consumers.keys().cloned().map(|id| ContextDependencyNode {
            id: ContextDependencyNodeId::Consumer(id),
            kind: ContextDependencyNodeKind::Consumer,
        }))
        .collect::<Vec<_>>();
    let mut edges = Vec::new();

    for context in contexts.values() {
        let Some(default) = &context.default_expression else {
            continue;
        };
        let default_node = ContextDependencyNodeId::ContextDefault(context.id.clone());
        edges.push(ContextDependencyEdge {
            dependent: default_node.clone(),
            dependency: ContextDependencyNodeId::Context(context.id.clone()),
            kind: ContextDependencyEdgeKind::ContextDefaultSuppliesContext,
            compatibility: context_types
                .get(&context.id)
                .and_then(|record| record.default_compatibility)
                .map_or(
                    ContextDependencyCompatibility::Unknown,
                    compatibility_from_status,
                ),
            provenance: expression_graph.node(default).map_or_else(
                || context.provenance.clone(),
                |node| node.provenance.clone(),
            ),
        });
        edges.extend(expression_read_edges(
            &default_node,
            context.owner.entity_id(),
            expression_graph,
            components,
            computed_values,
            ContextDependencyEdgeKind::ContextDefaultReadsState,
            ContextDependencyEdgeKind::ContextDefaultReadsComputed,
        ));
    }
    for provider in providers.values() {
        let provider_node = ContextDependencyNodeId::Provider(provider.id.clone());
        edges.push(ContextDependencyEdge {
            dependent: provider_node.clone(),
            dependency: ContextDependencyNodeId::Context(provider.context.clone()),
            kind: ContextDependencyEdgeKind::ProviderSuppliesContext,
            compatibility: provider_types
                .get(&provider.id)
                .map_or(ContextDependencyCompatibility::Unknown, |record| {
                    compatibility_from_status(record.declaration_to_context)
                }),
            provenance: provider.context_designator.provenance.clone(),
        });
        edges.extend(expression_read_edges(
            &provider_node,
            provider.owner.entity_id(),
            expression_graph,
            components,
            computed_values,
            ContextDependencyEdgeKind::ProviderReadsState,
            ContextDependencyEdgeKind::ProviderReadsComputed,
        ));
    }
    for consumer in consumers.values() {
        let Some(resolution) = resolutions.get(&consumer.id) else {
            continue;
        };
        let dependent = ContextDependencyNodeId::Consumer(consumer.id.clone());
        let compatibility = binding_types
            .get(&consumer.id)
            .map_or(ContextDependencyCompatibility::Unknown, |binding| {
                compatibility_from_binding(binding.overall)
            });
        let edge = match &resolution.result {
            ContextResolutionResult::Provider { provider, .. } => Some(ContextDependencyEdge {
                dependent,
                dependency: ContextDependencyNodeId::Provider(provider.clone()),
                kind: ContextDependencyEdgeKind::ConsumerReadsProvider,
                compatibility,
                provenance: consumer.context_designator.provenance.clone(),
            }),
            ContextResolutionResult::ContextDefault { context, .. } => {
                Some(ContextDependencyEdge {
                    dependent,
                    dependency: ContextDependencyNodeId::ContextDefault(context.clone()),
                    kind: ContextDependencyEdgeKind::ConsumerReadsContextDefault,
                    compatibility,
                    provenance: consumer.context_designator.provenance.clone(),
                })
            }
            ContextResolutionResult::Unresolved
            | ContextResolutionResult::Ambiguous { .. }
            | ContextResolutionResult::InvalidContextReference => None,
        };
        if let Some(edge) = edge {
            edges.push(edge);
        }
    }

    edges.sort_by(|left, right| {
        (
            &left.dependent,
            left.kind,
            &left.dependency,
            left.provenance.span.start,
        )
            .cmp(&(
                &right.dependent,
                right.kind,
                &right.dependency,
                right.provenance.span.start,
            ))
    });
    edges.dedup_by(|left, right| {
        left.dependent == right.dependent
            && left.kind == right.kind
            && left.dependency == right.dependency
    });

    let referenced_values = edges
        .iter()
        .filter_map(|edge| match &edge.dependency {
            ContextDependencyNodeId::State(id) => Some(ContextDependencyNode {
                id: ContextDependencyNodeId::State(id.clone()),
                kind: ContextDependencyNodeKind::State,
            }),
            ContextDependencyNodeId::Computed(id) => Some(ContextDependencyNode {
                id: ContextDependencyNodeId::Computed(id.clone()),
                kind: ContextDependencyNodeKind::Computed,
            }),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    nodes.extend(referenced_values);
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes.dedup_by(|left, right| left.id == right.id);

    let mut dependencies_by_node = BTreeMap::new();
    let mut dependents_by_node = BTreeMap::new();
    let mut provider_value_dependencies = BTreeMap::new();
    let mut context_default_dependencies = BTreeMap::new();
    let mut consumer_binding_dependencies = BTreeMap::new();
    let mut consumers_by_provider = BTreeMap::new();
    let mut consumers_by_default = BTreeMap::new();
    for edge in &edges {
        dependencies_by_node
            .entry(edge.dependent.clone())
            .or_insert_with(Vec::new)
            .push(edge.dependency.clone());
        dependents_by_node
            .entry(edge.dependency.clone())
            .or_insert_with(Vec::new)
            .push(edge.dependent.clone());
        match (&edge.dependent, &edge.dependency) {
            (ContextDependencyNodeId::Provider(provider), dependency)
                if matches!(
                    edge.kind,
                    ContextDependencyEdgeKind::ProviderReadsState
                        | ContextDependencyEdgeKind::ProviderReadsComputed
                ) =>
            {
                provider_value_dependencies
                    .entry(provider.clone())
                    .or_insert_with(Vec::new)
                    .push(dependency.clone());
            }
            (ContextDependencyNodeId::ContextDefault(context), dependency)
                if matches!(
                    edge.kind,
                    ContextDependencyEdgeKind::ContextDefaultReadsState
                        | ContextDependencyEdgeKind::ContextDefaultReadsComputed
                ) =>
            {
                context_default_dependencies
                    .entry(context.clone())
                    .or_insert_with(Vec::new)
                    .push(dependency.clone());
            }
            (
                ContextDependencyNodeId::Consumer(consumer),
                ContextDependencyNodeId::Provider(provider),
            ) => {
                consumer_binding_dependencies.insert(consumer.clone(), edge.dependency.clone());
                consumers_by_provider
                    .entry(provider.clone())
                    .or_insert_with(Vec::new)
                    .push(consumer.clone());
            }
            (
                ContextDependencyNodeId::Consumer(consumer),
                ContextDependencyNodeId::ContextDefault(context),
            ) => {
                consumer_binding_dependencies.insert(consumer.clone(), edge.dependency.clone());
                consumers_by_default
                    .entry(context.clone())
                    .or_insert_with(Vec::new)
                    .push(consumer.clone());
            }
            _ => {}
        }
    }
    for dependencies in dependencies_by_node.values_mut() {
        dependencies.sort();
        dependencies.dedup();
    }
    for dependents in dependents_by_node.values_mut() {
        dependents.sort();
        dependents.dedup();
    }
    for dependencies in provider_value_dependencies.values_mut() {
        dependencies.sort();
        dependencies.dedup();
    }
    for dependencies in context_default_dependencies.values_mut() {
        dependencies.sort();
        dependencies.dedup();
    }
    for consumers in consumers_by_provider.values_mut() {
        consumers.sort();
        consumers.dedup();
    }
    for consumers in consumers_by_default.values_mut() {
        consumers.sort();
        consumers.dedup();
    }
    ContextDependencyGraph {
        nodes,
        edges,
        dependencies_by_node,
        dependents_by_node,
        provider_value_dependencies,
        context_default_dependencies,
        consumer_binding_dependencies,
        consumers_by_provider,
        consumers_by_default,
    }
}

#[allow(clippy::too_many_arguments)]
fn expression_read_edges(
    dependent: &ContextDependencyNodeId,
    owner: Option<&SemanticId>,
    expression_graph: &ExpressionGraph,
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    state_kind: ContextDependencyEdgeKind,
    computed_kind: ContextDependencyEdgeKind,
) -> Vec<ContextDependencyEdge> {
    let Some(owner) = owner else {
        return Vec::new();
    };
    let Some(component) = components.iter().find(|component| component.id == *owner) else {
        return Vec::new();
    };
    expression_graph
        .nodes_for(match dependent {
            ContextDependencyNodeId::Provider(provider) => provider.as_semantic_id(),
            ContextDependencyNodeId::ContextDefault(context) => context.as_semantic_id(),
            _ => return Vec::new(),
        })
        .into_iter()
        .filter_map(|node| {
            let ExpressionNodeKind::ThisMember { name } = &node.kind else {
                return None;
            };
            let (dependency, kind) = if let Some(state) = component
                .state_fields
                .iter()
                .find(|field| field.name == *name)
            {
                (ContextDependencyNodeId::State(state.id.clone()), state_kind)
            } else if let Some(computed) = computed_values.get(&component.id.computed(name)) {
                (
                    ContextDependencyNodeId::Computed(computed.id.clone()),
                    computed_kind,
                )
            } else {
                return None;
            };
            Some(ContextDependencyEdge {
                dependent: dependent.clone(),
                dependency,
                kind,
                compatibility: ContextDependencyCompatibility::NotApplicable,
                provenance: node.provenance.clone(),
            })
        })
        .collect()
}

fn compatibility_from_status(status: CompatibilityStatus) -> ContextDependencyCompatibility {
    match status {
        CompatibilityStatus::Compatible => ContextDependencyCompatibility::Compatible,
        CompatibilityStatus::Incompatible => ContextDependencyCompatibility::Incompatible,
        CompatibilityStatus::Unknown => ContextDependencyCompatibility::Unknown,
    }
}

fn compatibility_from_binding(
    status: ContextBindingCompatibility,
) -> ContextDependencyCompatibility {
    match status {
        ContextBindingCompatibility::Compatible => ContextDependencyCompatibility::Compatible,
        ContextBindingCompatibility::Incompatible => ContextDependencyCompatibility::Incompatible,
        ContextBindingCompatibility::Unknown => ContextDependencyCompatibility::Unknown,
        ContextBindingCompatibility::Unresolved
        | ContextBindingCompatibility::Ambiguous
        | ContextBindingCompatibility::InvalidContextReference => {
            ContextDependencyCompatibility::NotApplicable
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        validate_application_semantic_model, CompilationUnit, ConsumerId,
        ContextDependencyCompatibility, ContextDependencyEdgeKind, ContextDependencyNodeId,
        ProviderId,
    };

    #[test]
    fn projects_provider_reads_and_exact_selected_provider_bindings() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/app-shell.tsx",
                r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  theme!: string;
  @context()
  mode!: string;
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
  selected = state("dark");
  @computed()
  get derivedTheme(): string { return this.selected; }
  @provide(AppShell.theme)
  providedTheme: string = this.selected;
  @provide(AppShell.mode)
  providedMode: string = this.derivedTheme;
  @consume(AppShell.theme)
  theme!: string;
  render() { return <main />; }
}
"#,
            ),
        ]);
        let asm = build_application_semantic_model_for_unit(&unit);
        let component = &asm.components[1].id;
        let provider = ProviderId::for_component(component, "providedTheme");
        let computed_provider = ProviderId::for_component(component, "providedMode");
        let consumer = ConsumerId::for_component(component, "theme");
        let state = ContextDependencyNodeId::State(component.state_field("selected"));
        let graph = asm.context_dependency_graph();

        assert_eq!(graph.provider_value_dependencies(&provider), &[state]);
        assert_eq!(
            graph.provider_value_dependencies(&computed_provider),
            &[ContextDependencyNodeId::Computed(
                component.computed("derivedTheme")
            )]
        );
        assert_eq!(
            graph.consumer_binding_dependency(&consumer),
            Some(&ContextDependencyNodeId::Provider(provider.clone()))
        );
        assert_eq!(graph.consumers_bound_to_provider(&provider), &[consumer]);
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == ContextDependencyEdgeKind::ProviderSuppliesContext
                && edge.compatibility == ContextDependencyCompatibility::Compatible
        }));
    }

    #[test]
    fn retains_default_and_unresolved_consumer_topology_without_hidden_providers() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/app.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context()
  locale: string = "en";
  @context()
  mode!: string;
  @consume(App.locale)
  selectedLocale!: string;
  render() { return <main />; }
}
@component("x-toolbar")
class Toolbar extends Component {
  @consume(App.mode)
  mode!: string;
  render() { return <main />; }
}
"#,
        ));
        let app = &asm.components[0].id;
        let toolbar = &asm.components[1].id;
        let context = asm.contexts()[0].id.clone();
        let default = ContextDependencyNodeId::ContextDefault(context.clone());
        let selected = ConsumerId::for_component(app, "selectedLocale");
        let unresolved = ConsumerId::for_component(toolbar, "mode");
        let graph = asm.context_dependency_graph();

        assert_eq!(graph.consumer_binding_dependency(&selected), Some(&default));
        assert_eq!(graph.consumers_bound_to_default(&context), &[selected]);
        assert_eq!(graph.consumer_binding_dependency(&unresolved), None);
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == ContextDependencyEdgeKind::ContextDefaultSuppliesContext
                && edge.dependent == default
        }));
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn preserves_incompatible_selected_provider_and_deterministic_ordering() {
        let parsed = presolve_parser::parse_file(
            "src/app.tsx",
            r#"
@component("x-app")
class App extends Component {
  @context()
  theme!: number;
  @provide(App.theme)
  providedTheme: string = "dark";
  @consume(App.theme)
  themeValue!: number;
  render() { return <main />; }
}
"#,
        );
        let first = build_application_semantic_model(&parsed);
        let second = build_application_semantic_model(&parsed);
        let component = &first.components[0].id;
        let consumer = ConsumerId::for_component(component, "themeValue");
        let provider = ProviderId::for_component(component, "providedTheme");
        let edge = first
            .context_dependency_graph()
            .edges
            .iter()
            .find(|edge| edge.kind == ContextDependencyEdgeKind::ConsumerReadsProvider)
            .unwrap();

        assert_eq!(first.context_dependency, second.context_dependency);
        assert_eq!(edge.dependent, ContextDependencyNodeId::Consumer(consumer));
        assert_eq!(edge.dependency, ContextDependencyNodeId::Provider(provider));
        assert_eq!(
            edge.compatibility,
            ContextDependencyCompatibility::Incompatible
        );
    }
}
