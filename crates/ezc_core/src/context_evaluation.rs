use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CompatibilityStatus, ConsumerId, ContextBindingCompatibility, ContextBindingLifetimeStatus,
    ContextBindingTypeRecord, ContextDefaultLifetimeRecord, ContextDependencyNodeId, ContextEntity,
    ContextId, ContextLifetimeAnalysis, ContextResolution, ContextResolutionResult,
    ContextSerializationCompatibility, ContextTypeRecord, IrReactiveGraph, IrReactiveNode,
    IrReactiveNodeKind, IrUpdateScheduler, ProviderId, ProviderLifetimeRecord, ProviderTypeRecord,
    SemanticId, SourceProvenance,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextEvaluationPlanId {
    Initial,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextEvaluationBatchId {
    pub plan: ContextEvaluationPlanId,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextValueSourceId {
    ContextDefault(ContextId),
    Provider(ProviderId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSourcePlanStatus {
    Planned,
    Unused,
    BlockedType,
    BlockedSerialization,
    BlockedBoundary,
    BlockedLifetime,
    BlockedDependency,
    BlockedUnsupportedExpression,
    UnknownEligibility,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextSourceBlockReason {
    TypeIncompatible,
    TypeUnknown,
    NonSerializable,
    SerializationUnknown,
    BoundaryIncompatible,
    BoundaryUnknown,
    LifetimeIncompatible,
    LifetimeUnknown,
    MissingStateDependency(SemanticId),
    UnavailableComputedDependency(SemanticId),
    UnsupportedExpression,
    InvalidContextTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSourcePlanEntry {
    pub source: ContextValueSourceId,
    pub owner_component: SemanticId,
    pub context: ContextId,
    pub expression_root: SemanticId,
    pub status: ContextSourcePlanStatus,
    pub reasons: Vec<ContextSourceBlockReason>,
    pub required_state: Vec<SemanticId>,
    pub required_computed: Vec<SemanticId>,
    pub prerequisite_computed_batches: Vec<u32>,
    pub evaluation_batch: Option<u32>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationBatch {
    pub id: ContextEvaluationBatchId,
    pub sources: Vec<ContextValueSourceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextConsumerAvailabilityStatus {
    Available,
    BlockedSource,
    TypeIncompatible,
    SerializationIncompatible,
    BoundaryIncompatible,
    LifetimeIncompatible,
    UnknownEligibility,
    Unresolved,
    Ambiguous,
    InvalidContextReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextConsumerAvailabilityEntry {
    pub consumer: ConsumerId,
    pub resolution: ContextResolutionResult,
    pub selected_source: Option<ContextValueSourceId>,
    pub status: ContextConsumerAvailabilityStatus,
    pub source_batch: Option<u32>,
    pub required_computed: Vec<SemanticId>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEvaluationPlan {
    pub id: ContextEvaluationPlanId,
    pub source_entries: BTreeMap<ContextValueSourceId, ContextSourcePlanEntry>,
    pub consumer_entries: BTreeMap<ConsumerId, ContextConsumerAvailabilityEntry>,
    pub evaluation_batches: Vec<ContextEvaluationBatch>,
    pub blocked_entries: Vec<ContextValueSourceId>,
    planned_sources: Vec<ContextValueSourceId>,
    available_consumers: Vec<ConsumerId>,
    unavailable_consumers: Vec<ConsumerId>,
    batches_by_id: BTreeMap<ContextEvaluationBatchId, ContextEvaluationBatch>,
}

impl ContextEvaluationPlan {
    #[must_use]
    pub fn context_source_plan(
        &self,
        source: &ContextValueSourceId,
    ) -> Option<&ContextSourcePlanEntry> {
        self.source_entries.get(source)
    }
    #[must_use]
    pub fn context_consumer_availability(
        &self,
        consumer: &ConsumerId,
    ) -> Option<&ContextConsumerAvailabilityEntry> {
        self.consumer_entries.get(consumer)
    }
    #[must_use]
    pub fn context_evaluation_batch(
        &self,
        id: &ContextEvaluationBatchId,
    ) -> Option<&ContextEvaluationBatch> {
        self.batches_by_id.get(id)
    }
    #[must_use]
    pub fn planned_context_sources(&self) -> &[ContextValueSourceId] {
        &self.planned_sources
    }
    #[must_use]
    pub fn blocked_context_sources(&self) -> &[ContextValueSourceId] {
        &self.blocked_entries
    }
    #[must_use]
    pub fn available_context_consumers(&self) -> &[ConsumerId] {
        &self.available_consumers
    }
    #[must_use]
    pub fn unavailable_context_consumers(&self) -> &[ConsumerId] {
        &self.unavailable_consumers
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[must_use]
pub fn collect_context_evaluation_plan(
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, crate::ProviderEntity>,
    resolutions: &BTreeMap<ConsumerId, ContextResolution>,
    context_types: &BTreeMap<ContextId, ContextTypeRecord>,
    provider_types: &BTreeMap<ProviderId, ProviderTypeRecord>,
    binding_types: &BTreeMap<ConsumerId, ContextBindingTypeRecord>,
    lifetime: &ContextLifetimeAnalysis,
    dependency: &crate::ContextDependencyGraph,
    computed_plan: &crate::IrComputedEvaluationPlan,
    scope: &crate::ComponentScopeGraph,
) -> ContextEvaluationPlan {
    let mut entries = BTreeMap::new();
    for context in contexts.values() {
        if let Some(expression_root) = &context.default_expression {
            let source = ContextValueSourceId::ContextDefault(context.id.clone());
            let owner = lifetime
                .context_default_lifetime(&context.id)
                .map(|record| record.owner_component.clone());
            let (required_state, required_computed) = source_dependencies(dependency, &source);
            let provenance = dependency
                .edges
                .iter()
                .find(|edge| {
                    edge.dependent == ContextDependencyNodeId::ContextDefault(context.id.clone())
                        && edge.kind
                            == crate::ContextDependencyEdgeKind::ContextDefaultSuppliesContext
                })
                .map_or_else(
                    || context.provenance.clone(),
                    |edge| edge.provenance.clone(),
                );
            let reasons = default_reasons(
                context,
                context_types.get(&context.id),
                lifetime.context_default_lifetime(&context.id),
                &required_state,
                &required_computed,
                computed_plan,
            );
            if let Some(owner_component) = owner {
                entries.insert(
                    source.clone(),
                    ContextSourcePlanEntry {
                        source,
                        owner_component,
                        context: context.id.clone(),
                        expression_root: expression_root.clone(),
                        status: primary_status(&reasons),
                        reasons,
                        required_state,
                        required_computed: required_computed.clone(),
                        prerequisite_computed_batches: computed_batches(
                            &required_computed,
                            computed_plan,
                        ),
                        evaluation_batch: None,
                        provenance,
                    },
                );
            }
        }
    }
    for provider in providers.values() {
        let source = ContextValueSourceId::Provider(provider.id.clone());
        let owner = lifetime
            .provider_lifetime(&provider.id)
            .map(|record| record.owner_component.clone());
        let (required_state, required_computed) = source_dependencies(dependency, &source);
        let reasons = provider_reasons(
            provider,
            provider_types.get(&provider.id),
            lifetime.provider_lifetime(&provider.id),
            &required_state,
            &required_computed,
            computed_plan,
        );
        if let Some(owner_component) = owner {
            entries.insert(
                source.clone(),
                ContextSourcePlanEntry {
                    source,
                    owner_component,
                    context: provider.context.clone(),
                    expression_root: provider.value_expression.clone(),
                    status: primary_status(&reasons),
                    reasons,
                    required_state,
                    required_computed: required_computed.clone(),
                    prerequisite_computed_batches: computed_batches(
                        &required_computed,
                        computed_plan,
                    ),
                    evaluation_batch: None,
                    provenance: provider.provenance.clone(),
                },
            );
        }
    }
    let mut demand = BTreeSet::new();
    let mut selected = BTreeSet::new();
    for (consumer, resolution) in resolutions {
        let Some(source) = selected_source(&resolution.result) else {
            continue;
        };
        selected.insert(source.clone());
        if binding_types
            .get(consumer)
            .is_some_and(|binding| binding.overall == ContextBindingCompatibility::Compatible)
            && lifetime
                .context_binding_lifetime(consumer)
                .is_some_and(|record| {
                    record.compatibility == ContextBindingLifetimeStatus::Compatible
                })
        {
            demand.insert(source);
        }
    }
    for (source, entry) in &mut entries {
        if !selected.contains(source) {
            entry.status = ContextSourcePlanStatus::Unused;
        } else if entry.reasons.is_empty() && demand.contains(source) {
            entry.status = ContextSourcePlanStatus::Planned;
        } else if entry.reasons.is_empty() {
            entry.status = ContextSourcePlanStatus::Unused;
        }
    }
    let batches = schedule_sources(&entries, scope);
    for batch in &batches {
        for source in &batch.sources {
            if let Some(entry) = entries.get_mut(source) {
                entry.evaluation_batch = Some(batch.id.index);
            }
        }
    }
    let consumer_entries = resolutions
        .iter()
        .map(|(consumer, resolution)| {
            let selected_source = selected_source(&resolution.result);
            let source_entry = selected_source
                .as_ref()
                .and_then(|source| entries.get(source));
            let status = consumer_status(
                &resolution.result,
                binding_types.get(consumer),
                lifetime.context_binding_lifetime(consumer),
                source_entry,
            );
            let required_computed =
                source_entry.map_or_else(Vec::new, |entry| entry.required_computed.clone());
            (
                consumer.clone(),
                ContextConsumerAvailabilityEntry {
                    consumer: consumer.clone(),
                    resolution: resolution.result.clone(),
                    selected_source,
                    status,
                    source_batch: source_entry.and_then(|entry| entry.evaluation_batch),
                    required_computed,
                    provenance: resolution.provenance.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let planned_sources = entries
        .iter()
        .filter_map(|(source, entry)| {
            (entry.status == ContextSourcePlanStatus::Planned).then_some(source.clone())
        })
        .collect();
    let blocked_entries = entries
        .iter()
        .filter_map(|(source, entry)| {
            (!matches!(
                entry.status,
                ContextSourcePlanStatus::Planned | ContextSourcePlanStatus::Unused
            ))
            .then_some(source.clone())
        })
        .collect();
    let available_consumers = consumer_entries
        .iter()
        .filter_map(|(consumer, entry)| {
            (entry.status == ContextConsumerAvailabilityStatus::Available)
                .then_some(consumer.clone())
        })
        .collect();
    let unavailable_consumers = consumer_entries
        .iter()
        .filter_map(|(consumer, entry)| {
            (entry.status != ContextConsumerAvailabilityStatus::Available)
                .then_some(consumer.clone())
        })
        .collect();
    let batches_by_id = batches
        .iter()
        .map(|batch| (batch.id.clone(), batch.clone()))
        .collect();
    ContextEvaluationPlan {
        id: ContextEvaluationPlanId::Initial,
        source_entries: entries,
        consumer_entries,
        evaluation_batches: batches,
        blocked_entries,
        planned_sources,
        available_consumers,
        unavailable_consumers,
        batches_by_id,
    }
}

fn selected_source(result: &ContextResolutionResult) -> Option<ContextValueSourceId> {
    match result {
        ContextResolutionResult::Provider { provider, .. } => {
            Some(ContextValueSourceId::Provider(provider.clone()))
        }
        ContextResolutionResult::ContextDefault { context, .. } => {
            Some(ContextValueSourceId::ContextDefault(context.clone()))
        }
        _ => None,
    }
}

fn source_dependencies(
    graph: &crate::ContextDependencyGraph,
    source: &ContextValueSourceId,
) -> (Vec<SemanticId>, Vec<SemanticId>) {
    let node = match source {
        ContextValueSourceId::Provider(id) => ContextDependencyNodeId::Provider(id.clone()),
        ContextValueSourceId::ContextDefault(id) => {
            ContextDependencyNodeId::ContextDefault(id.clone())
        }
    };
    graph
        .direct_dependencies(&node)
        .iter()
        .fold((Vec::new(), Vec::new()), |mut acc, dependency| {
            match dependency {
                ContextDependencyNodeId::State(id) => acc.0.push(id.clone()),
                ContextDependencyNodeId::Computed(id) => acc.1.push(id.clone()),
                _ => {}
            }
            acc
        })
}

fn provider_reasons(
    provider: &crate::ProviderEntity,
    types: Option<&ProviderTypeRecord>,
    lifetime: Option<&ProviderLifetimeRecord>,
    states: &[SemanticId],
    computed: &[SemanticId],
    computed_plan: &crate::IrComputedEvaluationPlan,
) -> Vec<ContextSourceBlockReason> {
    let mut reasons = Vec::new();
    let Some(types) = types else {
        return vec![ContextSourceBlockReason::TypeUnknown];
    };
    status_reasons(types.value_to_declaration, &mut reasons);
    status_reasons(types.declaration_to_context, &mut reasons);
    serialization_reasons(types.serialization, &mut reasons);
    compatibility_reasons(
        types.boundary_compatibility,
        ContextSourceBlockReason::BoundaryIncompatible,
        ContextSourceBlockReason::BoundaryUnknown,
        &mut reasons,
    );
    lifetime_reasons(lifetime.map(|record| record.compatibility), &mut reasons);
    dependency_reasons(states, computed, computed_plan, &mut reasons);
    if provider.value_expression.as_str().is_empty() {
        reasons.push(ContextSourceBlockReason::UnsupportedExpression);
    }
    normalize_reasons(reasons)
}

fn default_reasons(
    context: &ContextEntity,
    types: Option<&ContextTypeRecord>,
    lifetime: Option<&ContextDefaultLifetimeRecord>,
    states: &[SemanticId],
    computed: &[SemanticId],
    computed_plan: &crate::IrComputedEvaluationPlan,
) -> Vec<ContextSourceBlockReason> {
    let mut reasons = Vec::new();
    let Some(types) = types else {
        return vec![ContextSourceBlockReason::TypeUnknown];
    };
    match types.default_compatibility {
        Some(compatibility) => status_reasons(compatibility, &mut reasons),
        None => reasons.push(ContextSourceBlockReason::TypeUnknown),
    }
    serialization_reasons(types.serialization, &mut reasons);
    compatibility_reasons(
        types.boundary_compatibility,
        ContextSourceBlockReason::BoundaryIncompatible,
        ContextSourceBlockReason::BoundaryUnknown,
        &mut reasons,
    );
    lifetime_reasons(lifetime.map(|record| record.compatibility), &mut reasons);
    dependency_reasons(states, computed, computed_plan, &mut reasons);
    if context.default_expression.is_none() {
        reasons.push(ContextSourceBlockReason::UnsupportedExpression);
    }
    normalize_reasons(reasons)
}

fn status_reasons(status: CompatibilityStatus, reasons: &mut Vec<ContextSourceBlockReason>) {
    match status {
        CompatibilityStatus::Compatible => {}
        CompatibilityStatus::Incompatible => {
            reasons.push(ContextSourceBlockReason::TypeIncompatible);
        }
        CompatibilityStatus::Unknown => reasons.push(ContextSourceBlockReason::TypeUnknown),
    }
}
fn serialization_reasons(
    status: ContextSerializationCompatibility,
    reasons: &mut Vec<ContextSourceBlockReason>,
) {
    match status {
        ContextSerializationCompatibility::Serializable => {}
        ContextSerializationCompatibility::NonSerializable => {
            reasons.push(ContextSourceBlockReason::NonSerializable);
        }
        ContextSerializationCompatibility::Unknown => {
            reasons.push(ContextSourceBlockReason::SerializationUnknown);
        }
    }
}
fn compatibility_reasons(
    status: CompatibilityStatus,
    incompatible: ContextSourceBlockReason,
    unknown: ContextSourceBlockReason,
    reasons: &mut Vec<ContextSourceBlockReason>,
) {
    match status {
        CompatibilityStatus::Compatible => {}
        CompatibilityStatus::Incompatible => reasons.push(incompatible),
        CompatibilityStatus::Unknown => reasons.push(unknown),
    }
}
fn lifetime_reasons(
    status: Option<crate::LifetimeCompatibilityStatus>,
    reasons: &mut Vec<ContextSourceBlockReason>,
) {
    match status {
        Some(crate::LifetimeCompatibilityStatus::Compatible) => {}
        Some(crate::LifetimeCompatibilityStatus::Incompatible) => {
            reasons.push(ContextSourceBlockReason::LifetimeIncompatible);
        }
        _ => reasons.push(ContextSourceBlockReason::LifetimeUnknown),
    }
}
fn dependency_reasons(
    states: &[SemanticId],
    computed: &[SemanticId],
    plan: &crate::IrComputedEvaluationPlan,
    reasons: &mut Vec<ContextSourceBlockReason>,
) {
    for state in states {
        if state.as_str().is_empty() {
            reasons.push(ContextSourceBlockReason::MissingStateDependency(
                state.clone(),
            ));
        }
    }
    for value in computed {
        if !plan.evaluation_order.contains(&value.as_str().to_string())
            || plan.unplanned.contains(&value.as_str().to_string())
        {
            reasons.push(ContextSourceBlockReason::UnavailableComputedDependency(
                value.clone(),
            ));
        }
    }
}
fn normalize_reasons(mut reasons: Vec<ContextSourceBlockReason>) -> Vec<ContextSourceBlockReason> {
    reasons.sort();
    reasons.dedup();
    reasons
}
fn primary_status(reasons: &[ContextSourceBlockReason]) -> ContextSourcePlanStatus {
    if reasons.iter().any(|reason| {
        matches!(
            reason,
            ContextSourceBlockReason::TypeIncompatible | ContextSourceBlockReason::TypeUnknown
        )
    }) {
        ContextSourcePlanStatus::BlockedType
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            ContextSourceBlockReason::NonSerializable
                | ContextSourceBlockReason::SerializationUnknown
        )
    }) {
        ContextSourcePlanStatus::BlockedSerialization
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            ContextSourceBlockReason::BoundaryIncompatible
                | ContextSourceBlockReason::BoundaryUnknown
        )
    }) {
        ContextSourcePlanStatus::BlockedBoundary
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            ContextSourceBlockReason::LifetimeIncompatible
                | ContextSourceBlockReason::LifetimeUnknown
        )
    }) {
        ContextSourcePlanStatus::BlockedLifetime
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            ContextSourceBlockReason::MissingStateDependency(_)
                | ContextSourceBlockReason::UnavailableComputedDependency(_)
        )
    }) {
        ContextSourcePlanStatus::BlockedDependency
    } else if reasons
        .iter()
        .any(|reason| matches!(reason, ContextSourceBlockReason::UnsupportedExpression))
    {
        ContextSourcePlanStatus::BlockedUnsupportedExpression
    } else {
        ContextSourcePlanStatus::UnknownEligibility
    }
}
fn computed_batches(values: &[SemanticId], plan: &crate::IrComputedEvaluationPlan) -> Vec<u32> {
    plan.update_batches
        .iter()
        .enumerate()
        .filter_map(|(index, batch)| {
            values
                .iter()
                .any(|value| batch.contains(&value.as_str().to_string()))
                .then_some(u32::try_from(index).ok())
                .flatten()
        })
        .collect()
}
fn schedule_sources(
    entries: &BTreeMap<ContextValueSourceId, ContextSourcePlanEntry>,
    scope: &crate::ComponentScopeGraph,
) -> Vec<ContextEvaluationBatch> {
    let source_keys = entries
        .values()
        .filter(|entry| entry.status == ContextSourcePlanStatus::Planned)
        .map(|entry| {
            let depth = scope.ancestor_chain(&entry.owner_component).len();
            (
                format!("{depth:08}:{:?}", entry.source),
                entry.source.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let nodes = source_keys
        .iter()
        .filter_map(|(id, source)| {
            entries.get(source).map(|entry| {
                (
                    id.clone(),
                    IrReactiveNode {
                        id: id.clone(),
                        kind: IrReactiveNodeKind::Effect,
                        provenance: entry.provenance.clone(),
                    },
                )
            })
        })
        .collect();
    let inspection = IrUpdateScheduler::new(IrReactiveGraph {
        nodes,
        edges: Vec::new(),
    })
    .inspect();
    inspection
        .batches
        .into_iter()
        .enumerate()
        .filter_map(|(index, batch)| {
            let index = u32::try_from(index).ok()?;
            let sources = batch
                .into_iter()
                .filter_map(|key| source_keys.get(&key).cloned())
                .collect::<Vec<_>>();
            (!sources.is_empty()).then_some(ContextEvaluationBatch {
                id: ContextEvaluationBatchId {
                    plan: ContextEvaluationPlanId::Initial,
                    index,
                },
                sources,
            })
        })
        .collect()
}
fn consumer_status(
    resolution: &ContextResolutionResult,
    binding: Option<&ContextBindingTypeRecord>,
    lifetime: Option<&crate::ContextBindingLifetimeRecord>,
    source: Option<&ContextSourcePlanEntry>,
) -> ContextConsumerAvailabilityStatus {
    match resolution {
        ContextResolutionResult::Unresolved => ContextConsumerAvailabilityStatus::Unresolved,
        ContextResolutionResult::Ambiguous { .. } => ContextConsumerAvailabilityStatus::Ambiguous,
        ContextResolutionResult::InvalidContextReference => {
            ContextConsumerAvailabilityStatus::InvalidContextReference
        }
        _ => {
            let Some(binding) = binding else {
                return ContextConsumerAvailabilityStatus::UnknownEligibility;
            };
            if binding.overall == ContextBindingCompatibility::Incompatible {
                if binding.serialization == ContextSerializationCompatibility::NonSerializable {
                    return ContextConsumerAvailabilityStatus::SerializationIncompatible;
                }
                if binding.boundary_compatibility == CompatibilityStatus::Incompatible {
                    return ContextConsumerAvailabilityStatus::BoundaryIncompatible;
                }
                return ContextConsumerAvailabilityStatus::TypeIncompatible;
            }
            if binding.overall == ContextBindingCompatibility::Unknown {
                return ContextConsumerAvailabilityStatus::UnknownEligibility;
            }
            match lifetime.map(|record| record.compatibility) {
                Some(ContextBindingLifetimeStatus::Incompatible) => {
                    ContextConsumerAvailabilityStatus::LifetimeIncompatible
                }
                Some(ContextBindingLifetimeStatus::Unknown) | None => {
                    ContextConsumerAvailabilityStatus::UnknownEligibility
                }
                Some(ContextBindingLifetimeStatus::Compatible)
                    if source
                        .is_some_and(|entry| entry.status == ContextSourcePlanStatus::Planned) =>
                {
                    ContextConsumerAvailabilityStatus::Available
                }
                _ => ContextConsumerAvailabilityStatus::BlockedSource,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, ConsumerId, ContextConsumerAvailabilityStatus,
        ContextSourcePlanStatus, ProviderId,
    };
    #[test]
    fn plans_one_shared_provider_for_eligible_consumers() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/app.tsx",
            r#"@component("x-app") class App extends Component { @context() theme!: string; @provide(App.theme) provided: string = "dark"; @consume(App.theme) a!: string; @consume(App.theme) b!: string; render() { return <main />; } }"#,
        ));
        let component = &asm.components[0].id;
        let provider = ProviderId::for_component(component, "provided");
        let plan = asm.context_evaluation_plan();
        assert_eq!(
            plan.context_source_plan(&crate::ContextValueSourceId::Provider(provider))
                .unwrap()
                .status,
            ContextSourcePlanStatus::Planned
        );
        assert_eq!(plan.evaluation_batches.len(), 1);
        assert_eq!(plan.available_context_consumers().len(), 2);
    }
    #[test]
    fn retains_unused_and_incompatible_consumer_facts() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/app.tsx",
            r#"@component("x-app") class App extends Component { @context() theme!: number; @provide(App.theme) provided: string = "dark"; @consume(App.theme) value!: number; render() { return <main />; } }"#,
        ));
        let component = &asm.components[0].id;
        let provider = ProviderId::for_component(component, "provided");
        let consumer = ConsumerId::for_component(component, "value");
        let plan = asm.context_evaluation_plan();
        assert_eq!(
            plan.context_source_plan(&crate::ContextValueSourceId::Provider(provider))
                .unwrap()
                .status,
            ContextSourcePlanStatus::BlockedType
        );
        assert_eq!(
            plan.context_consumer_availability(&consumer)
                .unwrap()
                .status,
            ContextConsumerAvailabilityStatus::TypeIncompatible
        );
    }
    #[test]
    fn plans_a_selected_context_default_once() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/app.tsx",
            r#"@component("x-app") class App extends Component { @context() locale: string = "en"; @consume(App.locale) selected!: string; render() { return <main />; } }"#,
        ));
        let component = &asm.components[0].id;
        let context = crate::ContextId::for_component(component, "locale");
        let consumer = ConsumerId::for_component(component, "selected");
        let plan = asm.context_evaluation_plan();
        assert_eq!(
            plan.context_source_plan(&crate::ContextValueSourceId::ContextDefault(context))
                .unwrap()
                .status,
            ContextSourcePlanStatus::Planned
        );
        assert_eq!(
            plan.context_consumer_availability(&consumer)
                .unwrap()
                .status,
            ContextConsumerAvailabilityStatus::Available
        );
    }
    #[test]
    fn retains_an_unselected_provider_without_an_executable_batch() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/app.tsx",
            r#"@component("x-app") class App extends Component { @context() theme!: string; @provide(App.theme) provided: string = "dark"; render() { return <main />; } }"#,
        ));
        let component = &asm.components[0].id;
        let provider = ProviderId::for_component(component, "provided");
        let plan = asm.context_evaluation_plan();
        assert_eq!(
            plan.context_source_plan(&crate::ContextValueSourceId::Provider(provider))
                .unwrap()
                .status,
            ContextSourcePlanStatus::Unused
        );
        assert!(plan.evaluation_batches.is_empty());
    }
    #[test]
    fn orders_same_scope_sources_by_stable_source_identity() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/app.tsx",
            r#"@component("x-app") class App extends Component { @context() alpha!: string; @context() beta!: string; @provide(App.beta) second: string = "b"; @provide(App.alpha) first: string = "a"; @consume(App.beta) betaValue!: string; @consume(App.alpha) alphaValue!: string; render() { return <main />; } }"#,
        ));
        let component = &asm.components[0].id;
        let plan = asm.context_evaluation_plan();
        assert_eq!(plan.evaluation_batches.len(), 1);
        assert_eq!(
            plan.evaluation_batches[0].sources,
            vec![
                crate::ContextValueSourceId::Provider(ProviderId::for_component(
                    component, "first"
                )),
                crate::ContextValueSourceId::Provider(ProviderId::for_component(
                    component, "second"
                )),
            ]
        );
    }
}
