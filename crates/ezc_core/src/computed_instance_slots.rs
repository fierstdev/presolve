//! J1-C compiler-owned instance qualification for E12 computed runtime slots.

use crate::{
    build_runtime_computed_registry, ApplicationSemanticModel, ComponentInstanceId,
    ComponentInstanceStatus, ComputedInstanceCacheSlotId, ComputedInstanceDirtySlotId,
    IntermediateRepresentation, SemanticId,
};

pub const COMPUTED_INSTANCE_SLOT_REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedInstanceSlotRecord {
    pub component_instance_id: ComponentInstanceId,
    pub component_id: SemanticId,
    pub computed_id: SemanticId,
    pub cache_slot_id: ComputedInstanceCacheSlotId,
    pub dirty_slot_id: ComputedInstanceDirtySlotId,
    pub dirty_initial_value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedInstanceSlotRegistry {
    pub version: u32,
    pub records: Vec<ComputedInstanceSlotRecord>,
}

#[must_use]
pub fn build_computed_instance_slot_registry(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
) -> ComputedInstanceSlotRegistry {
    let declaration_records = build_runtime_computed_registry(model, ir);
    let mut records = Vec::new();
    for instance in model.component_instance_plan.instances.values() {
        if instance.status != ComponentInstanceStatus::Planned {
            continue;
        }
        for computed in model
            .computed_values
            .values()
            .filter(|computed| computed.owner.entity_id() == Some(&instance.component))
        {
            let Some(declaration) = declaration_records.record(&computed.id) else {
                continue;
            };
            records.push(ComputedInstanceSlotRecord {
                component_instance_id: instance.id.clone(),
                component_id: instance.component.clone(),
                computed_id: computed.id.clone(),
                cache_slot_id: ComputedInstanceCacheSlotId::for_component_instance_cache_slot(
                    instance.id.clone(),
                    declaration.cache_slot.clone(),
                ),
                dirty_slot_id: ComputedInstanceDirtySlotId::for_component_instance_dirty_flag(
                    instance.id.clone(),
                    declaration.dirty_flag.id.clone(),
                ),
                dirty_initial_value: declaration.dirty_flag.initial_value,
            });
        }
    }
    records.sort_by(|left, right| {
        (&left.component_instance_id, &left.computed_id)
            .cmp(&(&right.component_instance_id, &right.computed_id))
    });
    ComputedInstanceSlotRegistry {
        version: COMPUTED_INSTANCE_SLOT_REGISTRY_VERSION,
        records,
    }
}

/// # Errors
///
/// Returns an error when a registry is not an exact immutable projection.
pub fn validate_computed_instance_slot_registry(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
    registry: &ComputedInstanceSlotRegistry,
) -> Result<(), String> {
    if registry.version != COMPUTED_INSTANCE_SLOT_REGISTRY_VERSION {
        return Err("unsupported computed instance slot registry version".to_string());
    }
    if registry != &build_computed_instance_slot_registry(model, ir) {
        return Err("computed instance slot registry drifted from canonical products".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_distinct_cache_and_dirty_slots_for_repeated_instances() {
        let model = crate::build_application_semantic_model(&ezc_parser::parse_file(
            "src/RepeatedComputed.tsx",
            r#"@component("x-child") class Child { count = state(1); @computed() get doubled() { return this.count * 2; } render() { return <span>{this.doubled}</span>; } }
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }"#,
        ));
        let ir = crate::lower_components_to_ir(&model);
        let registry = build_computed_instance_slot_registry(&model, &ir);
        assert_eq!(registry.records.len(), 2);
        assert_ne!(
            registry.records[0].cache_slot_id,
            registry.records[1].cache_slot_id
        );
        assert_ne!(
            registry.records[0].dirty_slot_id,
            registry.records[1].dirty_slot_id
        );
        assert_eq!(
            registry.records[0].computed_id,
            registry.records[1].computed_id
        );
        assert!(registry
            .records
            .iter()
            .all(|record| record.dirty_initial_value));
        assert!(validate_computed_instance_slot_registry(&model, &ir, &registry).is_ok());
    }
}
