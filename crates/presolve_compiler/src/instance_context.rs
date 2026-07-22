use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    ComponentInstanceId, ComponentInstanceScopeGraph, ComponentRootId, ConsumerEntity, ConsumerId,
    ContextEntity, ContextId, ProviderEntity, ProviderId, SemanticId, SourceProvenance,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProviderInstanceId {
    pub provider: ProviderId,
    pub component_instance: ComponentInstanceId,
}

impl ProviderInstanceId {
    #[must_use]
    pub fn new(provider: ProviderId, component_instance: ComponentInstanceId) -> Self {
        Self {
            provider,
            component_instance,
        }
    }
}

impl std::fmt::Display for ProviderInstanceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "provider-instance:{}@{}",
            self.provider, self.component_instance
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConsumerInstanceId {
    pub consumer: ConsumerId,
    pub component_instance: ComponentInstanceId,
}

impl ConsumerInstanceId {
    #[must_use]
    pub fn new(consumer: ConsumerId, component_instance: ComponentInstanceId) -> Self {
        Self {
            consumer,
            component_instance,
        }
    }
}

impl std::fmt::Display for ConsumerInstanceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "consumer-instance:{}@{}",
            self.consumer, self.component_instance
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContextDefaultSourceInstanceId {
    pub context: ContextId,
    pub owner_root: ComponentRootId,
}

impl std::fmt::Display for ContextDefaultSourceInstanceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "context-default-instance:{}@{}",
            self.context, self.owner_root
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ContextSourceInstanceOwner {
    Provider(ProviderInstanceId),
    RootDefault(ContextDefaultSourceInstanceId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContextSourceInstanceId {
    pub context: ContextId,
    pub owner: ContextSourceInstanceOwner,
}

impl std::fmt::Display for ContextSourceInstanceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "context-source:{}@", self.context)?;
        match &self.owner {
            ContextSourceInstanceOwner::Provider(provider) => provider.fmt(formatter),
            ContextSourceInstanceOwner::RootDefault(default) => default.fmt(formatter),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceContextValueSlotId(String);

impl InstanceContextValueSlotId {
    #[must_use]
    pub fn for_source(source: &ContextSourceInstanceId) -> Self {
        Self(format!("context-instance-slot:{source}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InstanceContextValueSlotId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInstanceRecord {
    pub id: ProviderInstanceId,
    pub context: ContextId,
    pub component: SemanticId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerInstanceRecord {
    pub id: ConsumerInstanceId,
    pub context: Option<ContextId>,
    pub component: SemanticId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceContextResolutionStatus {
    ProviderSelected,
    ContextDefaultSelected,
    Unresolved,
    Ambiguous,
    InvalidContextReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceContextResolution {
    pub consumer_instance: ConsumerInstanceId,
    pub context: Option<ContextId>,
    pub selected_source: Option<ContextSourceInstanceId>,
    pub provider_instance: Option<ProviderInstanceId>,
    pub default_source: Option<ContextDefaultSourceInstanceId>,
    pub value_slot: Option<InstanceContextValueSlotId>,
    pub ancestry_distance: Option<u32>,
    pub candidate_provider_instances: Vec<ProviderInstanceId>,
    pub status: InstanceContextResolutionStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstanceContextRegistry {
    pub provider_instances: BTreeMap<ProviderInstanceId, ProviderInstanceRecord>,
    pub consumer_instances: BTreeMap<ConsumerInstanceId, ConsumerInstanceRecord>,
    pub default_sources: BTreeSet<ContextDefaultSourceInstanceId>,
    pub resolutions: BTreeMap<ConsumerInstanceId, InstanceContextResolution>,
}

impl InstanceContextRegistry {
    #[must_use]
    pub fn resolution(&self, consumer: &ConsumerInstanceId) -> Option<&InstanceContextResolution> {
        self.resolutions.get(consumer)
    }

    #[must_use]
    pub fn resolutions_for_declaration(
        &self,
        consumer: &ConsumerId,
    ) -> Vec<&InstanceContextResolution> {
        self.resolutions
            .values()
            .filter(|resolution| &resolution.consumer_instance.consumer == consumer)
            .collect()
    }
}

/// Reproject exact Context declarations through canonical H5 instance ancestry.
#[must_use]
pub fn collect_instance_context_registry(
    scope: &ComponentInstanceScopeGraph,
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
) -> InstanceContextRegistry {
    let providers_by_component = provider_index(providers);
    let consumers_by_component = consumer_index(consumers);
    let mut registry = InstanceContextRegistry::default();

    for node in scope.nodes.values() {
        for provider in providers.values().filter(|provider| {
            provider
                .owner
                .entity_id()
                .is_some_and(|owner| owner == &node.component)
        }) {
            let id = ProviderInstanceId::new(provider.id.clone(), node.id.clone());
            registry.provider_instances.insert(
                id.clone(),
                ProviderInstanceRecord {
                    id,
                    context: provider.context.clone(),
                    component: node.component.clone(),
                    provenance: provider.provenance.clone(),
                },
            );
        }
    }

    for node in scope.nodes.values() {
        let Some(component_consumers) = consumers_by_component.get(&node.component) else {
            continue;
        };
        for consumer in component_consumers {
            let id = ConsumerInstanceId::new(consumer.id.clone(), node.id.clone());
            registry.consumer_instances.insert(
                id.clone(),
                ConsumerInstanceRecord {
                    id: id.clone(),
                    context: consumer.context().cloned(),
                    component: node.component.clone(),
                    provenance: consumer.context_designator.provenance.clone(),
                },
            );
            let resolution = resolve_consumer_instance(
                id.clone(),
                consumer,
                scope,
                contexts,
                &providers_by_component,
            );
            if let Some(default) = &resolution.default_source {
                registry.default_sources.insert(default.clone());
            }
            registry.resolutions.insert(id, resolution);
        }
    }

    registry
}

#[derive(Debug, Clone)]
struct IndexedProvider {
    provider: ProviderId,
}

fn provider_index(
    providers: &BTreeMap<ProviderId, ProviderEntity>,
) -> BTreeMap<(SemanticId, ContextId), Vec<IndexedProvider>> {
    let mut index = BTreeMap::<(SemanticId, ContextId), Vec<IndexedProvider>>::new();
    for provider in providers.values() {
        let Some(component) = provider.owner.entity_id() else {
            continue;
        };
        index
            .entry((component.clone(), provider.context.clone()))
            .or_default()
            .push(IndexedProvider {
                provider: provider.id.clone(),
            });
    }
    for providers in index.values_mut() {
        providers.sort_by(|left, right| left.provider.cmp(&right.provider));
    }
    index
}

fn consumer_index(
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
) -> BTreeMap<SemanticId, Vec<&ConsumerEntity>> {
    let mut index = BTreeMap::<SemanticId, Vec<&ConsumerEntity>>::new();
    for consumer in consumers.values() {
        if let Some(component) = consumer.owner.entity_id() {
            index.entry(component.clone()).or_default().push(consumer);
        }
    }
    for consumers in index.values_mut() {
        consumers.sort_by(|left, right| left.id.cmp(&right.id));
    }
    index
}

#[allow(clippy::too_many_lines)]
fn resolve_consumer_instance(
    consumer_instance: ConsumerInstanceId,
    consumer: &ConsumerEntity,
    scope: &ComponentInstanceScopeGraph,
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers_by_component: &BTreeMap<(SemanticId, ContextId), Vec<IndexedProvider>>,
) -> InstanceContextResolution {
    let provenance = consumer.context_designator.provenance.clone();
    let Some(context) = consumer.context().cloned() else {
        return InstanceContextResolution {
            consumer_instance,
            context: None,
            selected_source: None,
            provider_instance: None,
            default_source: None,
            value_slot: None,
            ancestry_distance: None,
            candidate_provider_instances: Vec::new(),
            status: InstanceContextResolutionStatus::InvalidContextReference,
            provenance,
        };
    };
    let mut scopes = vec![&consumer_instance.component_instance];
    scopes.extend(scope.ancestors(&consumer_instance.component_instance));

    for (distance, instance_id) in scopes.into_iter().enumerate() {
        let Some(node) = scope.node(instance_id) else {
            continue;
        };
        let candidates = providers_by_component
            .get(&(node.component.clone(), context.clone()))
            .into_iter()
            .flatten()
            .map(|provider| ProviderInstanceId::new(provider.provider.clone(), instance_id.clone()))
            .collect::<Vec<_>>();
        match candidates.as_slice() {
            [] => {}
            [provider_instance] => {
                let selected_source = ContextSourceInstanceId {
                    context: context.clone(),
                    owner: ContextSourceInstanceOwner::Provider(provider_instance.clone()),
                };
                return InstanceContextResolution {
                    consumer_instance,
                    context: Some(context),
                    selected_source: Some(selected_source.clone()),
                    provider_instance: Some(provider_instance.clone()),
                    default_source: None,
                    value_slot: Some(InstanceContextValueSlotId::for_source(&selected_source)),
                    ancestry_distance: Some(
                        u32::try_from(distance).expect("instance ancestry depth fits in u32"),
                    ),
                    candidate_provider_instances: candidates,
                    status: InstanceContextResolutionStatus::ProviderSelected,
                    provenance,
                };
            }
            _ => {
                return InstanceContextResolution {
                    consumer_instance,
                    context: Some(context),
                    selected_source: None,
                    provider_instance: None,
                    default_source: None,
                    value_slot: None,
                    ancestry_distance: Some(
                        u32::try_from(distance).expect("instance ancestry depth fits in u32"),
                    ),
                    candidate_provider_instances: candidates,
                    status: InstanceContextResolutionStatus::Ambiguous,
                    provenance,
                };
            }
        }
    }

    let owner_root = scope
        .node(&consumer_instance.component_instance)
        .expect("consumer instances belong to scope nodes")
        .owner_root
        .clone();
    if contexts
        .get(&context)
        .is_some_and(|context| context.default_expression.is_some())
    {
        let default_source = ContextDefaultSourceInstanceId {
            context: context.clone(),
            owner_root,
        };
        let selected_source = ContextSourceInstanceId {
            context: context.clone(),
            owner: ContextSourceInstanceOwner::RootDefault(default_source.clone()),
        };
        return InstanceContextResolution {
            consumer_instance,
            context: Some(context),
            selected_source: Some(selected_source.clone()),
            provider_instance: None,
            default_source: Some(default_source),
            value_slot: Some(InstanceContextValueSlotId::for_source(&selected_source)),
            ancestry_distance: None,
            candidate_provider_instances: Vec::new(),
            status: InstanceContextResolutionStatus::ContextDefaultSelected,
            provenance,
        };
    }

    InstanceContextResolution {
        consumer_instance,
        context: Some(context),
        selected_source: None,
        provider_instance: None,
        default_source: None,
        value_slot: None,
        ancestry_distance: None,
        candidate_provider_instances: Vec::new(),
        status: InstanceContextResolutionStatus::Unresolved,
        provenance,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, validate_application_semantic_model,
        ContextResolutionResult, InstanceContextResolutionStatus,
    };

    #[test]
    fn resolves_repeated_consumer_definitions_to_distinct_provider_instances() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/InstanceContext.tsx",
            r#"
@component("x-theme") class Theme extends Component {
  @context() color!: string;
  render() { return <div />; }
}
@component("x-leaf") class Leaf extends Component {
  @consume(Theme.color) color!: string;
  render() { return <span />; }
}
@component("x-light") class Light extends Component {
  @provide(Theme.color) color: string = "light";
  render() { return <Leaf />; }
}
@component("x-dark") class Dark extends Component {
  @provide(Theme.color) color: string = "dark";
  render() { return <Leaf />; }
}
@component("x-page") @route("/") class Page extends Component {
  render() { return <main><Light /><Dark /></main>; }
}
"#,
        ));
        let consumer = asm.consumers().first().unwrap().id.clone();
        let resolutions = asm.instance_context.resolutions_for_declaration(&consumer);

        assert_eq!(resolutions.len(), 2);
        assert!(resolutions.iter().all(|resolution| {
            resolution.status == InstanceContextResolutionStatus::ProviderSelected
                && resolution.ancestry_distance == Some(1)
                && resolution.value_slot.is_some()
        }));
        assert_ne!(
            resolutions[0].provider_instance,
            resolutions[1].provider_instance
        );
        assert_ne!(
            resolutions[0].selected_source,
            resolutions[1].selected_source
        );
        assert_ne!(resolutions[0].value_slot, resolutions[1].value_slot);
        assert!(matches!(
            asm.context_resolution(&consumer).unwrap().result,
            ContextResolutionResult::Unresolved
        ));
    }

    #[test]
    fn uses_root_qualified_defaults_and_never_reselects_for_type_failure() {
        let default_asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Defaults.tsx",
            r#"
@component("x-theme") class Theme extends Component {
  @context() color: string = "blue";
  render() { return <div />; }
}
@component("x-leaf") class Leaf extends Component {
  @consume(Theme.color) color!: string;
  render() { return <span />; }
}
@component("x-a") @route("/a") class A extends Component { render() { return <Leaf />; } }
@component("x-b") @route("/b") class B extends Component { render() { return <Leaf />; } }
"#,
        ));
        let defaults = default_asm
            .instance_context
            .resolutions
            .values()
            .filter_map(|resolution| resolution.default_source.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(defaults.len(), 2);
        assert_ne!(defaults[0].owner_root, defaults[1].owner_root);

        let incompatible = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Incompatible.tsx",
            r#"
@component("x-theme") class Theme extends Component {
  @context() color!: string;
  render() { return <div />; }
}
@component("x-leaf") class Leaf extends Component {
  @consume(Theme.color) color!: string;
  render() { return <span />; }
}
@component("x-page") @route("/") class Page extends Component {
  @provide(Theme.color) color: number = 1;
  render() { return <Leaf />; }
}
"#,
        ));
        let resolution = incompatible
            .instance_context
            .resolutions
            .values()
            .next()
            .unwrap();
        assert_eq!(
            resolution.status,
            InstanceContextResolutionStatus::ProviderSelected
        );
        assert!(resolution.provider_instance.is_some());
        assert!(resolution.value_slot.is_some());
    }

    #[test]
    fn asm_validation_rejects_noncanonical_instance_context_resolution() {
        let mut asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Validate.tsx",
            r#"
@component("x-theme") class Theme extends Component {
  @context() color: string = "blue";
  render() { return <div />; }
}
@component("x-page")
@route("/")
class Page extends Component {
  @consume(Theme.color) color!: string;
  render() { return <main />; }
}
"#,
        ));
        assert!(validate_application_semantic_model(&asm).is_empty());

        asm.instance_context
            .resolutions
            .values_mut()
            .next()
            .unwrap()
            .status = InstanceContextResolutionStatus::Unresolved;
        let diagnostics = validate_application_semantic_model(&asm);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PSASM1194"
                && diagnostic.message
                    == "instance Context registry does not match canonical declarations and H5 ancestry"
        }));
    }
}
