use std::collections::BTreeMap;

use crate::{
    ComponentNode, ComponentScopeGraph, ComputedValue, ConsumerEntity, ConsumerId,
    ContextDependencyEdgeKind, ContextDependencyGraph, ContextDependencyNodeId, ContextEntity,
    ContextId, ContextOwnershipGraph, ContextResolution, ContextResolutionResult, ProviderEntity,
    ProviderId, SemanticId, SourceProvenance,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextLifetimeId {
    pub component: SemanticId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextLifetimeEntityId {
    Context(ContextId),
    Provider(ProviderId),
    Consumer(ConsumerId),
    ContextDefault(ContextId),
    State(SemanticId),
    Computed(SemanticId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeCompatibilityStatus {
    Compatible,
    Incompatible,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextBindingLifetimeStatus {
    Compatible,
    Incompatible,
    Unknown,
    Unresolved,
    Ambiguous,
    InvalidContextReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEntityLifetimeRecord {
    pub entity: ContextLifetimeEntityId,
    pub owner_component: SemanticId,
    pub lifetime: ContextLifetimeId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDependencyLifetimeRecord {
    pub dependent: ContextDependencyNodeId,
    pub dependency: ContextDependencyNodeId,
    pub dependent_owner: Option<SemanticId>,
    pub dependency_owner: Option<SemanticId>,
    pub compatibility: LifetimeCompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderLifetimeRecord {
    pub provider: ProviderId,
    pub owner_component: SemanticId,
    pub lifetime: ContextLifetimeId,
    pub compatibility: LifetimeCompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDefaultLifetimeRecord {
    pub context: ContextId,
    pub owner_component: SemanticId,
    pub lifetime: ContextLifetimeId,
    pub compatibility: LifetimeCompatibilityStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextBindingLifetimeSource {
    Provider(ProviderId),
    ContextDefault(ContextId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBindingLifetimeRecord {
    pub consumer: ConsumerId,
    pub consumer_owner: Option<SemanticId>,
    pub resolution: ContextResolutionResult,
    pub source: Option<ContextBindingLifetimeSource>,
    pub source_owner: Option<SemanticId>,
    pub compatibility: ContextBindingLifetimeStatus,
    pub distance: Option<u32>,
    pub provenance: SourceProvenance,
}

/// Immutable G8 component-scope availability facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifetimeAnalysis {
    pub entities: BTreeMap<ContextLifetimeEntityId, ContextEntityLifetimeRecord>,
    pub provider_lifetimes: BTreeMap<ProviderId, ProviderLifetimeRecord>,
    pub context_default_lifetimes: BTreeMap<ContextId, ContextDefaultLifetimeRecord>,
    pub dependency_lifetimes: Vec<ContextDependencyLifetimeRecord>,
    pub binding_lifetimes: BTreeMap<ConsumerId, ContextBindingLifetimeRecord>,
    dependency_index: BTreeMap<ContextDependencyNodeId, Vec<ContextDependencyLifetimeRecord>>,
}

impl ContextLifetimeAnalysis {
    #[must_use]
    pub fn context_entity_lifetime(
        &self,
        entity: &ContextLifetimeEntityId,
    ) -> Option<&ContextEntityLifetimeRecord> {
        self.entities.get(entity)
    }

    #[must_use]
    pub fn provider_lifetime(&self, provider: &ProviderId) -> Option<&ProviderLifetimeRecord> {
        self.provider_lifetimes.get(provider)
    }

    #[must_use]
    pub fn context_default_lifetime(
        &self,
        context: &ContextId,
    ) -> Option<&ContextDefaultLifetimeRecord> {
        self.context_default_lifetimes.get(context)
    }

    #[must_use]
    pub fn context_binding_lifetime(
        &self,
        consumer: &ConsumerId,
    ) -> Option<&ContextBindingLifetimeRecord> {
        self.binding_lifetimes.get(consumer)
    }

    #[must_use]
    pub fn lifetime_dependencies_of(
        &self,
        node: &ContextDependencyNodeId,
    ) -> &[ContextDependencyLifetimeRecord] {
        self.dependency_index.get(node).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn runtime_lifetime_eligible(&self, consumer: &ConsumerId) -> bool {
        self.context_binding_lifetime(consumer)
            .is_some_and(|record| record.compatibility == ContextBindingLifetimeStatus::Compatible)
    }
}

/// Derives G8 availability strictly from canonical ownership, G4 scope and
/// resolution, and G7 direct dependencies.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[must_use]
pub fn collect_context_lifetime_analysis(
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    ownership: &ContextOwnershipGraph,
    scope: &ComponentScopeGraph,
    resolutions: &BTreeMap<ConsumerId, ContextResolution>,
    dependencies: &ContextDependencyGraph,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> ContextLifetimeAnalysis {
    let mut entities = BTreeMap::new();
    for context in contexts.values() {
        insert_entity_lifetime(
            &mut entities,
            ContextLifetimeEntityId::Context(context.id.clone()),
            ownership.owner_of_context(&context.id).cloned(),
            context.provenance.clone(),
        );
        if let Some(default) = &context.default_expression {
            let default_provenance = default_provenance(context, dependencies, provenance, default);
            insert_entity_lifetime(
                &mut entities,
                ContextLifetimeEntityId::ContextDefault(context.id.clone()),
                ownership.owner_of_context(&context.id).cloned(),
                default_provenance,
            );
        }
    }
    for provider in providers.values() {
        insert_entity_lifetime(
            &mut entities,
            ContextLifetimeEntityId::Provider(provider.id.clone()),
            ownership.owner_of_provider(&provider.id).cloned(),
            provider.provenance.clone(),
        );
    }
    for consumer in consumers.values() {
        insert_entity_lifetime(
            &mut entities,
            ContextLifetimeEntityId::Consumer(consumer.id.clone()),
            ownership.owner_of_consumer(&consumer.id).cloned(),
            consumer.provenance.clone(),
        );
    }
    for node in &dependencies.nodes {
        match &node.id {
            ContextDependencyNodeId::State(id) => {
                if let Some(provenance) = provenance.get(id).cloned() {
                    insert_entity_lifetime(
                        &mut entities,
                        ContextLifetimeEntityId::State(id.clone()),
                        state_owner(components, id),
                        provenance,
                    );
                }
            }
            ContextDependencyNodeId::Computed(id) => {
                if let Some(computed) = computed_values.get(id) {
                    insert_entity_lifetime(
                        &mut entities,
                        ContextLifetimeEntityId::Computed(id.clone()),
                        computed.owner.entity_id().cloned(),
                        computed.provenance.clone(),
                    );
                }
            }
            _ => {}
        }
    }

    let mut dependency_lifetimes = dependencies
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.kind,
                ContextDependencyEdgeKind::ProviderReadsState
                    | ContextDependencyEdgeKind::ProviderReadsComputed
                    | ContextDependencyEdgeKind::ContextDefaultReadsState
                    | ContextDependencyEdgeKind::ContextDefaultReadsComputed
            )
        })
        .map(|edge| {
            let dependent_owner = lifetime_owner_for_node(&entities, &edge.dependent);
            let dependency_owner = lifetime_owner_for_node(&entities, &edge.dependency);
            ContextDependencyLifetimeRecord {
                dependent: edge.dependent.clone(),
                dependency: edge.dependency.clone(),
                dependent_owner: dependent_owner.clone(),
                dependency_owner: dependency_owner.clone(),
                compatibility: compare_owners(scope, dependency_owner, dependent_owner),
                provenance: edge.provenance.clone(),
            }
        })
        .collect::<Vec<_>>();
    dependency_lifetimes.sort_by(|left, right| {
        (&left.dependent, &left.dependency).cmp(&(&right.dependent, &right.dependency))
    });

    let provider_lifetimes = providers
        .values()
        .filter_map(|provider| {
            let owner = ownership.owner_of_provider(&provider.id)?.clone();
            let status = aggregate_lifetime(dependency_lifetimes.iter().filter(|record| {
                record.dependent == ContextDependencyNodeId::Provider(provider.id.clone())
            }));
            Some((
                provider.id.clone(),
                ProviderLifetimeRecord {
                    provider: provider.id.clone(),
                    owner_component: owner.clone(),
                    lifetime: ContextLifetimeId { component: owner },
                    compatibility: status,
                    provenance: provider.provenance.clone(),
                },
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let context_default_lifetimes = contexts
        .values()
        .filter_map(|context| {
            let default = context.default_expression.as_ref()?;
            let owner = ownership.owner_of_context(&context.id)?.clone();
            let status = aggregate_lifetime(dependency_lifetimes.iter().filter(|record| {
                record.dependent == ContextDependencyNodeId::ContextDefault(context.id.clone())
            }));
            Some((
                context.id.clone(),
                ContextDefaultLifetimeRecord {
                    context: context.id.clone(),
                    owner_component: owner.clone(),
                    lifetime: ContextLifetimeId { component: owner },
                    compatibility: status,
                    provenance: default_provenance(context, dependencies, provenance, default),
                },
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let binding_lifetimes = consumers
        .values()
        .filter_map(|consumer| {
            let resolution = resolutions.get(&consumer.id)?;
            let consumer_owner = ownership.owner_of_consumer(&consumer.id).cloned();
            let record = binding_lifetime_record(
                consumer,
                resolution,
                consumer_owner,
                &provider_lifetimes,
                &context_default_lifetimes,
                scope,
            );
            Some((consumer.id.clone(), record))
        })
        .collect::<BTreeMap<_, _>>();
    let mut dependency_index = BTreeMap::new();
    for record in &dependency_lifetimes {
        dependency_index
            .entry(record.dependent.clone())
            .or_insert_with(Vec::new)
            .push(record.clone());
    }
    ContextLifetimeAnalysis {
        entities,
        provider_lifetimes,
        context_default_lifetimes,
        dependency_lifetimes,
        binding_lifetimes,
        dependency_index,
    }
}

fn default_provenance(
    context: &ContextEntity,
    dependencies: &ContextDependencyGraph,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
    default: &SemanticId,
) -> SourceProvenance {
    dependencies
        .edges
        .iter()
        .find(|edge| {
            edge.kind == ContextDependencyEdgeKind::ContextDefaultSuppliesContext
                && edge.dependent == ContextDependencyNodeId::ContextDefault(context.id.clone())
        })
        .map_or_else(
            || {
                provenance
                    .get(default)
                    .cloned()
                    .unwrap_or_else(|| context.provenance.clone())
            },
            |edge| edge.provenance.clone(),
        )
}

fn insert_entity_lifetime(
    entities: &mut BTreeMap<ContextLifetimeEntityId, ContextEntityLifetimeRecord>,
    entity: ContextLifetimeEntityId,
    owner_component: Option<SemanticId>,
    provenance: SourceProvenance,
) {
    let Some(owner_component) = owner_component else {
        return;
    };
    entities.insert(
        entity.clone(),
        ContextEntityLifetimeRecord {
            entity,
            lifetime: ContextLifetimeId {
                component: owner_component.clone(),
            },
            owner_component,
            provenance,
        },
    );
}

fn state_owner(components: &[ComponentNode], state: &SemanticId) -> Option<SemanticId> {
    components.iter().find_map(|component| {
        component
            .state_fields
            .iter()
            .any(|field| field.id == *state)
            .then_some(component.id.clone())
    })
}

fn lifetime_owner_for_node(
    entities: &BTreeMap<ContextLifetimeEntityId, ContextEntityLifetimeRecord>,
    node: &ContextDependencyNodeId,
) -> Option<SemanticId> {
    let entity = match node {
        ContextDependencyNodeId::State(id) => ContextLifetimeEntityId::State(id.clone()),
        ContextDependencyNodeId::Computed(id) => ContextLifetimeEntityId::Computed(id.clone()),
        ContextDependencyNodeId::Context(id) => ContextLifetimeEntityId::Context(id.clone()),
        ContextDependencyNodeId::ContextDefault(id) => {
            ContextLifetimeEntityId::ContextDefault(id.clone())
        }
        ContextDependencyNodeId::Provider(id) => ContextLifetimeEntityId::Provider(id.clone()),
        ContextDependencyNodeId::Consumer(id) => ContextLifetimeEntityId::Consumer(id.clone()),
    };
    entities
        .get(&entity)
        .map(|record| record.owner_component.clone())
}

fn compare_owners(
    scope: &ComponentScopeGraph,
    source: Option<SemanticId>,
    dependent: Option<SemanticId>,
) -> LifetimeCompatibilityStatus {
    let (Some(source), Some(dependent)) = (source, dependent) else {
        return LifetimeCompatibilityStatus::Unknown;
    };
    if !scope.components.contains(&source) || !scope.components.contains(&dependent) {
        return LifetimeCompatibilityStatus::Unknown;
    }
    if scope.ancestor_chain(&dependent).contains(&source) {
        LifetimeCompatibilityStatus::Compatible
    } else {
        LifetimeCompatibilityStatus::Incompatible
    }
}

fn aggregate_lifetime<'a>(
    records: impl Iterator<Item = &'a ContextDependencyLifetimeRecord>,
) -> LifetimeCompatibilityStatus {
    let mut status = LifetimeCompatibilityStatus::Compatible;
    for record in records {
        match record.compatibility {
            LifetimeCompatibilityStatus::Incompatible => {
                return LifetimeCompatibilityStatus::Incompatible
            }
            LifetimeCompatibilityStatus::Unknown => status = LifetimeCompatibilityStatus::Unknown,
            LifetimeCompatibilityStatus::Compatible => {}
        }
    }
    status
}

fn binding_lifetime_record(
    consumer: &ConsumerEntity,
    resolution: &ContextResolution,
    consumer_owner: Option<SemanticId>,
    providers: &BTreeMap<ProviderId, ProviderLifetimeRecord>,
    defaults: &BTreeMap<ContextId, ContextDefaultLifetimeRecord>,
    scope: &ComponentScopeGraph,
) -> ContextBindingLifetimeRecord {
    let (source, source_owner, source_status, distance, compatibility) = match &resolution.result {
        ContextResolutionResult::Provider {
            provider, distance, ..
        } => {
            let source = providers.get(provider);
            let owner = source.map(|record| record.owner_component.clone());
            let scope_status = compare_owners(scope, owner.clone(), consumer_owner.clone());
            (
                Some(ContextBindingLifetimeSource::Provider(provider.clone())),
                owner,
                source.map(|record| record.compatibility),
                Some(*distance),
                combine_binding_status(scope_status, source.map(|record| record.compatibility)),
            )
        }
        ContextResolutionResult::ContextDefault { context, .. } => {
            let source = defaults.get(context);
            let owner = source.map(|record| record.owner_component.clone());
            let scope_status = compare_owners(scope, owner.clone(), consumer_owner.clone());
            let distance = owner.as_ref().and_then(|owner| {
                consumer_owner.as_ref().and_then(|consumer_owner| {
                    scope
                        .ancestor_chain(consumer_owner)
                        .iter()
                        .position(|candidate| candidate == owner)
                        .and_then(|index| u32::try_from(index).ok())
                })
            });
            (
                Some(ContextBindingLifetimeSource::ContextDefault(
                    context.clone(),
                )),
                owner,
                source.map(|record| record.compatibility),
                distance,
                combine_binding_status(scope_status, source.map(|record| record.compatibility)),
            )
        }
        ContextResolutionResult::Unresolved => (
            None,
            None,
            None,
            None,
            ContextBindingLifetimeStatus::Unresolved,
        ),
        ContextResolutionResult::Ambiguous { .. } => (
            None,
            None,
            None,
            None,
            ContextBindingLifetimeStatus::Ambiguous,
        ),
        ContextResolutionResult::InvalidContextReference => (
            None,
            None,
            None,
            None,
            ContextBindingLifetimeStatus::InvalidContextReference,
        ),
    };
    let _ = source_status;
    ContextBindingLifetimeRecord {
        consumer: consumer.id.clone(),
        consumer_owner,
        resolution: resolution.result.clone(),
        source,
        source_owner,
        compatibility,
        distance,
        provenance: consumer.context_designator.provenance.clone(),
    }
}

fn combine_binding_status(
    scope_status: LifetimeCompatibilityStatus,
    source_status: Option<LifetimeCompatibilityStatus>,
) -> ContextBindingLifetimeStatus {
    let source_status = source_status.unwrap_or(LifetimeCompatibilityStatus::Unknown);
    if scope_status == LifetimeCompatibilityStatus::Incompatible
        || source_status == LifetimeCompatibilityStatus::Incompatible
    {
        ContextBindingLifetimeStatus::Incompatible
    } else if scope_status == LifetimeCompatibilityStatus::Unknown
        || source_status == LifetimeCompatibilityStatus::Unknown
    {
        ContextBindingLifetimeStatus::Unknown
    } else {
        ContextBindingLifetimeStatus::Compatible
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::compare_owners;
    use crate::{
        build_application_semantic_model, build_component_graph, ComponentScopeGraph, ConsumerId,
        ContextBindingLifetimeStatus, ContextDependencyNodeId, LifetimeCompatibilityStatus,
        ProviderId,
    };

    #[test]
    fn retains_type_incompatible_but_lifetime_compatible_selected_bindings() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/app.tsx",
            r#"
@component("x-app")
class App extends Component {
  selected = state("dark");
  @context()
  theme!: number;
  @provide(App.theme)
  providedTheme: string = this.selected;
  @consume(App.theme)
  themeValue!: number;
  render() { return <main />; }
}
"#,
        ));
        let component = &asm.components[0].id;
        let provider = ProviderId::for_component(component, "providedTheme");
        let consumer = ConsumerId::for_component(component, "themeValue");
        let analysis = asm.context_lifetime_analysis();

        assert_eq!(
            analysis.provider_lifetime(&provider).unwrap().compatibility,
            LifetimeCompatibilityStatus::Compatible
        );
        assert_eq!(
            analysis
                .lifetime_dependencies_of(&ContextDependencyNodeId::Provider(provider.clone()))
                .first()
                .unwrap()
                .compatibility,
            LifetimeCompatibilityStatus::Compatible
        );
        assert_eq!(
            analysis
                .context_binding_lifetime(&consumer)
                .unwrap()
                .compatibility,
            ContextBindingLifetimeStatus::Compatible
        );
        assert!(analysis.runtime_lifetime_eligible(&consumer));
    }

    #[test]
    fn retains_default_and_unresolved_binding_statuses() {
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
        let selected = ConsumerId::for_component(&asm.components[0].id, "selectedLocale");
        let unresolved = ConsumerId::for_component(&asm.components[1].id, "mode");
        let analysis = asm.context_lifetime_analysis();

        assert_eq!(
            analysis
                .context_binding_lifetime(&selected)
                .unwrap()
                .compatibility,
            ContextBindingLifetimeStatus::Compatible
        );
        assert_eq!(
            analysis
                .context_binding_lifetime(&unresolved)
                .unwrap()
                .compatibility,
            ContextBindingLifetimeStatus::Unresolved
        );
    }

    #[test]
    fn uses_only_canonical_scope_ancestry_for_outlives() {
        let parsed = presolve_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-parent")
class Parent extends Component { render() { return <main />; } }
@component("x-child")
class Child extends Component { render() { return <main />; } }
"#,
        );
        let components = build_component_graph(&parsed).components;
        let parent = components[0].id.clone();
        let child = components[1].id.clone();
        let scope = ComponentScopeGraph::with_parent_relations(
            &components,
            BTreeMap::from([(child.clone(), parent.clone())]),
        );

        assert_eq!(
            compare_owners(&scope, Some(parent.clone()), Some(child.clone())),
            LifetimeCompatibilityStatus::Compatible
        );
        assert_eq!(
            compare_owners(&scope, Some(child), Some(parent)),
            LifetimeCompatibilityStatus::Incompatible
        );
    }
}
