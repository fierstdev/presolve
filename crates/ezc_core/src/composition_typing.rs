use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CompatibilityStatus, ComponentInvocationEntity, ComponentInvocationId,
    ComponentInvocationResolutionStatus, ConsumerInstanceId, ContextBindingLifetimeStatus,
    ContextBindingTypeRecord, ContextDefaultSourceInstanceId, ContextId, ContextLifetimeAnalysis,
    ContextSerializationCompatibility, ContextTypeRecord, ExecutionBoundary,
    InstanceContextRegistry, InstanceContextResolutionStatus, ProviderId, ProviderTypeRecord,
    SemanticId, SemanticOwner, SemanticReference, SemanticType, SlotBinding, SlotBindingId,
    SlotBindingRegistry, SlotBindingStatus, SlotContentFragment, SlotContentFragmentId, SlotEntity,
    SlotId, SlotOutlet, SlotOutletId, SourceProvenance,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompositionCompatibility {
    Compatible,
    Incompatible,
    Unknown,
    Unresolved,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInvocationTypeRecord {
    pub invocation: ComponentInvocationId,
    pub caller_boundary: ExecutionBoundary,
    pub target_boundary: Option<ExecutionBoundary>,
    pub boundary_compatibility: CompositionCompatibility,
    pub overall: CompositionCompatibility,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotBindingTypeRecord {
    pub binding: SlotBindingId,
    pub slot: Option<SlotId>,
    pub slot_type: Option<SemanticType>,
    pub content_type: Option<SemanticType>,
    pub type_compatibility: CompositionCompatibility,
    pub caller_dependencies_valid: bool,
    pub outlet_ownership_valid: bool,
    pub cardinality_valid: bool,
    pub overall: CompositionCompatibility,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceContextBindingTypeRecord {
    pub consumer_instance: ConsumerInstanceId,
    pub context: Option<ContextId>,
    pub provider: Option<ProviderId>,
    pub default_source: Option<ContextDefaultSourceInstanceId>,
    pub type_compatibility: CompositionCompatibility,
    pub lifetime_compatibility: CompositionCompatibility,
    pub serialization_compatibility: CompositionCompatibility,
    pub boundary_compatibility: CompositionCompatibility,
    pub overall: CompositionCompatibility,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompositionTypeProducts {
    pub invocations: BTreeMap<ComponentInvocationId, ComponentInvocationTypeRecord>,
    pub slot_bindings: BTreeMap<SlotBindingId, SlotBindingTypeRecord>,
    pub instance_context_bindings: BTreeMap<ConsumerInstanceId, InstanceContextBindingTypeRecord>,
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn collect_composition_type_products(
    components: &BTreeSet<SemanticId>,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    slot_bindings: &SlotBindingRegistry,
    slots: &BTreeMap<SlotId, SlotEntity>,
    fragments: &BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    outlets: &BTreeMap<SlotOutletId, SlotOutlet>,
    instance_context: &InstanceContextRegistry,
    context_binding_types: &BTreeMap<crate::ConsumerId, ContextBindingTypeRecord>,
    context_types: &BTreeMap<ContextId, ContextTypeRecord>,
    provider_types: &BTreeMap<ProviderId, ProviderTypeRecord>,
    context_lifetime: &ContextLifetimeAnalysis,
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    references: &[SemanticReference],
) -> CompositionTypeProducts {
    let invocation_records = invocations
        .values()
        .map(|invocation| {
            let resolved = invocation.status == ComponentInvocationResolutionStatus::Resolved
                && invocation
                    .target_component
                    .as_ref()
                    .is_some_and(|target| components.contains(target));
            let overall = if resolved {
                CompositionCompatibility::Compatible
            } else {
                CompositionCompatibility::Unresolved
            };
            (
                invocation.id.clone(),
                ComponentInvocationTypeRecord {
                    invocation: invocation.id.clone(),
                    caller_boundary: ExecutionBoundary::Client,
                    target_boundary: resolved.then_some(ExecutionBoundary::Client),
                    boundary_compatibility: overall,
                    overall,
                    provenance: invocation.provenance.clone(),
                },
            )
        })
        .collect();
    let slot_records = slot_bindings
        .bindings
        .values()
        .map(|binding| {
            let record = collect_slot_binding_type(
                binding,
                invocations
                    .get(&binding.invocation)
                    .map(|invocation| &invocation.owner_component),
                slots,
                fragments,
                outlets,
                components,
                ownership,
                references,
            );
            (binding.id.clone(), record)
        })
        .collect();
    let context_records = instance_context
        .resolutions
        .values()
        .map(|resolution| {
            let declaration = context_binding_types.get(&resolution.consumer_instance.consumer);
            let provider = resolution
                .provider_instance
                .as_ref()
                .map(|provider| provider.provider.clone());
            let record = collect_instance_context_type(
                resolution,
                declaration,
                provider
                    .as_ref()
                    .and_then(|provider| provider_types.get(provider)),
                resolution
                    .context
                    .as_ref()
                    .and_then(|context| context_types.get(context)),
                context_lifetime,
            );
            (resolution.consumer_instance.clone(), record)
        })
        .collect();

    CompositionTypeProducts {
        invocations: invocation_records,
        slot_bindings: slot_records,
        instance_context_bindings: context_records,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_slot_binding_type(
    binding: &SlotBinding,
    caller_component: Option<&SemanticId>,
    slots: &BTreeMap<SlotId, SlotEntity>,
    fragments: &BTreeMap<SlotContentFragmentId, SlotContentFragment>,
    outlets: &BTreeMap<SlotOutletId, SlotOutlet>,
    components: &BTreeSet<SemanticId>,
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    references: &[SemanticReference],
) -> SlotBindingTypeRecord {
    let slot = binding.slot.as_ref().and_then(|slot| slots.get(slot));
    let fragment = binding
        .content_fragment
        .as_ref()
        .and_then(|fragment| fragments.get(fragment));
    let slot_type = slot.map(|slot| slot.semantic_type.clone());
    let content_type = fragment.map(|_| SemanticType::SlotContent);
    let type_compatibility = match (&slot_type, &content_type) {
        (Some(SemanticType::SlotContent), Some(SemanticType::SlotContent) | None) => {
            CompositionCompatibility::Compatible
        }
        (None, _) => CompositionCompatibility::Unknown,
        _ => CompositionCompatibility::Incompatible,
    };
    let caller_dependencies_valid = fragment.is_none_or(|fragment| {
        fragment.content_template_entities.iter().all(|entity| {
            component_owner(entity, components, ownership).as_ref() == caller_component
                && references
                    .iter()
                    .filter(|reference| reference.source == *entity)
                    .all(|reference| {
                        component_owner(&reference.target, components, ownership).as_ref()
                            == caller_component
                    })
        })
    });
    let outlet_ownership_valid = binding.outlet.as_ref().is_none_or(|outlet| {
        outlets.get(outlet).is_some_and(|outlet| {
            binding.slot.as_ref() == outlet.slot.as_ref()
                && slots
                    .get(binding.slot.as_ref().expect("bound outlets name a Slot"))
                    .is_some_and(|slot| slot.owner == outlet.owner_component)
        })
    });
    let cardinality_valid = matches!(
        binding.status,
        SlotBindingStatus::Bound | SlotBindingStatus::Empty
    );
    let overall = if matches!(binding.status, SlotBindingStatus::BlockedInvocation) {
        CompositionCompatibility::Blocked
    } else if type_compatibility == CompositionCompatibility::Compatible
        && caller_dependencies_valid
        && outlet_ownership_valid
        && cardinality_valid
    {
        CompositionCompatibility::Compatible
    } else {
        CompositionCompatibility::Incompatible
    };
    SlotBindingTypeRecord {
        binding: binding.id.clone(),
        slot: binding.slot.clone(),
        slot_type,
        content_type,
        type_compatibility,
        caller_dependencies_valid,
        outlet_ownership_valid,
        cardinality_valid,
        overall,
        provenance: binding.provenance.clone(),
    }
}

fn component_owner(
    entity: &SemanticId,
    components: &BTreeSet<SemanticId>,
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Option<SemanticId> {
    let mut current = entity;
    let mut seen = BTreeSet::new();
    loop {
        if components.contains(current) {
            return Some(current.clone());
        }
        if !seen.insert(current.clone()) {
            return None;
        }
        let SemanticOwner::Entity(owner) = ownership.get(current)? else {
            return None;
        };
        current = owner;
    }
}

fn collect_instance_context_type(
    resolution: &crate::InstanceContextResolution,
    declaration: Option<&ContextBindingTypeRecord>,
    provider: Option<&ProviderTypeRecord>,
    context: Option<&ContextTypeRecord>,
    lifetime: &ContextLifetimeAnalysis,
) -> InstanceContextBindingTypeRecord {
    let type_compatibility = match resolution.status {
        InstanceContextResolutionStatus::ProviderSelected => {
            provider.map_or(CompositionCompatibility::Unknown, |provider| {
                combine_compatibility([
                    provider.value_to_declaration,
                    provider.declaration_to_context,
                    declaration.map_or(CompatibilityStatus::Unknown, |record| {
                        record.context_to_consumer
                    }),
                ])
            })
        }
        InstanceContextResolutionStatus::ContextDefaultSelected => combine_compatibility([
            context
                .and_then(|record| record.default_compatibility)
                .unwrap_or(CompatibilityStatus::Unknown),
            declaration.map_or(CompatibilityStatus::Unknown, |record| {
                record.context_to_consumer
            }),
        ]),
        InstanceContextResolutionStatus::Unresolved
        | InstanceContextResolutionStatus::Ambiguous
        | InstanceContextResolutionStatus::InvalidContextReference => {
            CompositionCompatibility::Unresolved
        }
    };
    let lifetime_compatibility = match resolution.status {
        InstanceContextResolutionStatus::ProviderSelected
        | InstanceContextResolutionStatus::ContextDefaultSelected => {
            match lifetime
                .context_binding_lifetime(&resolution.consumer_instance.consumer)
                .map(|record| record.compatibility)
            {
                Some(ContextBindingLifetimeStatus::Incompatible) => {
                    CompositionCompatibility::Incompatible
                }
                Some(ContextBindingLifetimeStatus::Unknown) => CompositionCompatibility::Unknown,
                _ => CompositionCompatibility::Compatible,
            }
        }
        _ => CompositionCompatibility::Unresolved,
    };
    let serialization = provider.map_or_else(
        || context.map(|record| record.serialization),
        |record| Some(record.serialization),
    );
    let serialization_compatibility = serialization.map_or(
        CompositionCompatibility::Unknown,
        |serialization| match serialization {
            ContextSerializationCompatibility::Serializable => CompositionCompatibility::Compatible,
            ContextSerializationCompatibility::NonSerializable => {
                CompositionCompatibility::Incompatible
            }
            ContextSerializationCompatibility::Unknown => CompositionCompatibility::Unknown,
        },
    );
    let boundary = provider.map_or_else(
        || context.map(|record| record.boundary_compatibility),
        |record| Some(record.boundary_compatibility),
    );
    let boundary_compatibility =
        boundary.map_or(CompositionCompatibility::Unknown, map_compatibility);
    let overall = combine_composition([
        type_compatibility,
        lifetime_compatibility,
        serialization_compatibility,
        boundary_compatibility,
    ]);
    InstanceContextBindingTypeRecord {
        consumer_instance: resolution.consumer_instance.clone(),
        context: resolution.context.clone(),
        provider: resolution
            .provider_instance
            .as_ref()
            .map(|provider| provider.provider.clone()),
        default_source: resolution.default_source.clone(),
        type_compatibility,
        lifetime_compatibility,
        serialization_compatibility,
        boundary_compatibility,
        overall,
        provenance: resolution.provenance.clone(),
    }
}

fn combine_compatibility<const N: usize>(
    statuses: [CompatibilityStatus; N],
) -> CompositionCompatibility {
    combine_composition(statuses.map(map_compatibility))
}

fn map_compatibility(status: CompatibilityStatus) -> CompositionCompatibility {
    match status {
        CompatibilityStatus::Compatible => CompositionCompatibility::Compatible,
        CompatibilityStatus::Incompatible => CompositionCompatibility::Incompatible,
        CompatibilityStatus::Unknown => CompositionCompatibility::Unknown,
    }
}

fn combine_composition<const N: usize>(
    statuses: [CompositionCompatibility; N],
) -> CompositionCompatibility {
    if statuses.contains(&CompositionCompatibility::Incompatible) {
        CompositionCompatibility::Incompatible
    } else if statuses.contains(&CompositionCompatibility::Unresolved) {
        CompositionCompatibility::Unresolved
    } else if statuses.contains(&CompositionCompatibility::Blocked) {
        CompositionCompatibility::Blocked
    } else if statuses.contains(&CompositionCompatibility::Unknown) {
        CompositionCompatibility::Unknown
    } else {
        CompositionCompatibility::Compatible
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, validate_application_semantic_model,
        CompositionCompatibility, ExecutionBoundary,
    };

    #[test]
    fn types_valid_invocations_slots_and_instance_context_without_reselection() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/CompositionTypes.tsx",
            r#"
@component("x-theme") class Theme extends Component {
  @context() color!: string;
  render() { return <div />; }
}
@component("x-leaf") class Leaf extends Component {
  @consume(Theme.color) color!: string;
  render() { return <span />; }
}
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  @provide(Theme.color) color: string = "blue";
  render() { return <article><Leaf /><slot /></article>; }
}
@component("x-page") class Page extends Component {
  render() { return <Card><p>caller</p></Card>; }
}
"#,
        ));
        assert!(asm.composition_types.invocations.values().all(|record| {
            record.caller_boundary == ExecutionBoundary::Client
                && record.target_boundary == Some(ExecutionBoundary::Client)
                && record.overall == CompositionCompatibility::Compatible
        }));
        assert!(asm.composition_types.slot_bindings.values().all(|record| {
            record.overall == CompositionCompatibility::Compatible
                && record.caller_dependencies_valid
                && record.outlet_ownership_valid
                && record.cardinality_valid
        }));
        let context = asm
            .composition_types
            .instance_context_bindings
            .values()
            .next()
            .unwrap();
        assert_eq!(context.overall, CompositionCompatibility::Compatible);
        assert_eq!(
            context.provider,
            asm.instance_context
                .resolutions
                .values()
                .next()
                .unwrap()
                .provider_instance
                .as_ref()
                .map(|provider| provider.provider.clone())
        );
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn retains_unresolved_blocked_and_incompatible_composition_facts() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/InvalidCompositionTypes.tsx",
            r#"
@component("x-theme") class Theme extends Component {
  @context() color!: string;
  render() { return <div />; }
}
@component("x-leaf") class Leaf extends Component {
  @consume(Theme.color) color!: string;
  render() { return <span />; }
}
@component("x-page") class Page extends Component {
  @provide(Theme.color) color: number = 1;
  render() { return <main><Leaf /><Missing><p /></Missing></main>; }
}
"#,
        ));
        assert!(asm
            .composition_types
            .invocations
            .values()
            .any(|record| record.overall == CompositionCompatibility::Unresolved));
        assert!(asm
            .composition_types
            .slot_bindings
            .values()
            .any(|record| record.overall == CompositionCompatibility::Blocked));
        let context = asm
            .composition_types
            .instance_context_bindings
            .values()
            .next()
            .unwrap();
        assert_eq!(context.overall, CompositionCompatibility::Incompatible);
        assert!(context.provider.is_some());
    }

    #[test]
    fn asm_validation_rejects_mutated_composition_typing() {
        let mut asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/ValidateCompositionTypes.tsx",
            r#"
@component("x-card") class Card extends Component { render() { return <div />; } }
@component("x-page") class Page extends Component { render() { return <Card />; } }
"#,
        ));
        asm.composition_types
            .invocations
            .values_mut()
            .next()
            .unwrap()
            .target_boundary = None;
        assert!(validate_application_semantic_model(&asm)
            .iter()
            .any(|diagnostic| diagnostic.code == "EZASM1196"));
    }
}
