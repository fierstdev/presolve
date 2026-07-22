use crate::{
    ApplicationSemanticModel, ComponentInstanceId, ComponentInstanceStatus, ComponentInvocationId,
    ContextSourceInstanceId, ExecutionBoundary, InstanceContextValueSlotId,
    OptimizedComponentIrReport, SemanticId, SlotBindingId, SlotContentFragmentId, SlotId,
    SlotOutletId, SourceProvenance,
};

/// Frozen H13 contract for compiler-owned component runtime metadata.
pub const RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION: u32 = 1;

/// Metadata consumed by the component runtime. It contains no authored-name
/// lookup table and never grants runtime authority to resolve composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComponentRegistry {
    pub schema_contract_version: u32,
    pub component_definitions: Vec<RuntimeComponentDefinitionRecord>,
    pub instances: Vec<RuntimeComponentInstanceRecord>,
    pub slot_bindings: Vec<RuntimeComponentSlotBindingRecord>,
    pub instance_context_bindings: Vec<RuntimeComponentContextBindingRecord>,
    pub initialization_batches: Vec<RuntimeComponentInitializationBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComponentDefinitionRecord {
    pub component: SemanticId,
    pub template: SemanticId,
    pub declared_slots: Vec<SlotId>,
    pub boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComponentInstanceRecord {
    pub instance: ComponentInstanceId,
    pub component: SemanticId,
    pub invocation: Option<ComponentInvocationId>,
    pub parent: Option<ComponentInstanceId>,
    pub depth: usize,
    pub initialization_batch: usize,
    pub instance_cache_prefix: String,
    pub instance_context_prefix: String,
    pub optimized_instruction_indices: Vec<usize>,
    pub structural_region: Option<crate::ComponentStructuralRegionId>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComponentSlotBindingRecord {
    pub binding: SlotBindingId,
    pub caller_instance: ComponentInstanceId,
    pub callee_instance: ComponentInstanceId,
    pub slot: SlotId,
    pub outlet: SlotOutletId,
    pub fragment: SlotContentFragmentId,
    pub content_owner_instance: ComponentInstanceId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComponentContextBindingRecord {
    pub consumer_instance: crate::ConsumerInstanceId,
    pub selected_source: ContextSourceInstanceId,
    pub provider_source: Option<crate::ProviderInstanceId>,
    pub runtime_slot: InstanceContextValueSlotId,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeComponentInitializationBatch {
    pub index: usize,
    pub instances: Vec<ComponentInstanceId>,
    pub context_sources: Vec<ContextSourceInstanceId>,
    pub slot_bindings: Vec<SlotBindingId>,
}

/// Projects H10/H12 into deterministic runtime metadata without resolving any
/// component, slot, Provider, Context, ancestry, or dependency at runtime.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_runtime_component_registry(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedComponentIrReport,
) -> RuntimeComponentRegistry {
    let executable = model
        .component_initialization
        .instance_batches
        .iter()
        .enumerate()
        .flat_map(|(batch, item)| item.instances.iter().cloned().map(move |id| (id, batch)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let component_ids = executable
        .keys()
        .filter_map(|id| model.component_instance_plan.instances.get(id))
        .map(|instance| instance.component.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let component_definitions = component_ids
        .iter()
        .filter_map(|component| {
            let provenance = model.provenance(component)?.clone();
            Some(RuntimeComponentDefinitionRecord {
                component: component.clone(),
                template: component.template(),
                declared_slots: model
                    .slots
                    .values()
                    .filter(|slot| slot.owner == *component)
                    .map(|slot| slot.id.clone())
                    .collect(),
                boundary: ExecutionBoundary::Client,
                provenance,
            })
        })
        .collect();
    let instances = executable
        .iter()
        .filter_map(|(id, batch)| instance_record(model, optimized, id, *batch))
        .collect();
    let slot_bindings = model
        .slot_bindings
        .bindings
        .values()
        .filter(|binding| executable.contains_key(&binding.callee_instance))
        .filter_map(|binding| {
            Some(RuntimeComponentSlotBindingRecord {
                binding: binding.id.clone(),
                caller_instance: binding.caller_instance.clone(),
                callee_instance: binding.callee_instance.clone(),
                slot: binding.slot.clone()?,
                outlet: binding.outlet.clone()?,
                fragment: binding.content_fragment.clone()?,
                content_owner_instance: binding.content_owner_instance.clone(),
                provenance: binding.provenance.clone(),
            })
        })
        .collect();
    let instance_context_bindings = model
        .instance_context
        .resolutions
        .values()
        .filter(|resolution| {
            executable.contains_key(&resolution.consumer_instance.component_instance)
                && model
                    .composition_types
                    .instance_context_bindings
                    .get(&resolution.consumer_instance)
                    .is_some_and(|record| {
                        record.overall == crate::CompositionCompatibility::Compatible
                    })
        })
        .filter_map(|resolution| {
            Some(RuntimeComponentContextBindingRecord {
                consumer_instance: resolution.consumer_instance.clone(),
                selected_source: resolution.selected_source.clone()?,
                provider_source: resolution.provider_instance.clone(),
                runtime_slot: resolution.value_slot.clone()?,
                provenance: resolution.provenance.clone(),
            })
        })
        .collect();
    let initialization_batches = model
        .component_initialization
        .instance_batches
        .iter()
        .map(|batch| RuntimeComponentInitializationBatch {
            index: batch.index,
            instances: batch.instances.clone(),
            context_sources: batch.context_sources.clone(),
            slot_bindings: model
                .component_initialization
                .slot_binding_batches
                .iter()
                .filter(|bindings| bindings.index == batch.index)
                .flat_map(|bindings| bindings.bindings.clone())
                .collect(),
        })
        .collect();
    RuntimeComponentRegistry {
        schema_contract_version: RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION,
        component_definitions,
        instances,
        slot_bindings,
        instance_context_bindings,
        initialization_batches,
    }
}

fn instance_record(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedComponentIrReport,
    id: &ComponentInstanceId,
    initialization_batch: usize,
) -> Option<RuntimeComponentInstanceRecord> {
    let instance = model.component_instance_plan.instances.get(id)?;
    (instance.status == ComponentInstanceStatus::Planned).then_some(RuntimeComponentInstanceRecord {
        instance: instance.id.clone(),
        component: instance.component.clone(),
        invocation: instance.invocation.clone(),
        parent: instance.parent_instance.clone(),
        depth: instance.depth,
        initialization_batch,
        instance_cache_prefix: format!("component-cache:{}", instance.id),
        instance_context_prefix: format!("component-context:{}", instance.id),
        optimized_instruction_indices: optimized
            .optimized_report
            .instructions
            .iter()
            .filter(|instruction| matches!(
                &instruction.operation,
                crate::ComponentIrOperation::CreateComponentInstance { instance: operation_instance, .. }
                    | crate::ComponentIrOperation::InitializeComponentInstance { instance: operation_instance, .. }
                    | crate::ComponentIrOperation::MaterializeComponentTemplate { instance: operation_instance, .. }
                if operation_instance == &instance.id
            ))
            .map(|instruction| instruction.index)
            .collect(),
        structural_region: instance.structural_region.clone(),
        provenance: instance.provenance.clone(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_component_registry,
        RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION,
    };

    #[test]
    fn projects_only_planned_instances_in_deterministic_id_order() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Registry.tsx",
            r#"
@component("x-leaf") class Leaf extends Component { render() { return <i />; } }
@component("x-page") class Page extends Component { render() { return <><Leaf /><Leaf /></>; } }
"#,
        ));
        let registry = build_runtime_component_registry(&model, &model.component_ir_optimization);
        assert_eq!(
            registry.schema_contract_version,
            RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION
        );
        assert_eq!(registry.instances.len(), 3);
        assert_ne!(
            registry.instances[1].instance,
            registry.instances[2].instance
        );
        assert!(registry
            .instances
            .windows(2)
            .all(|pair| pair[0].instance < pair[1].instance));
        assert!(registry
            .instances
            .iter()
            .all(|record| !record.optimized_instruction_indices.is_empty()));
    }
}
