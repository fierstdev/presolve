//! J1-A compiler-owned instance qualification for canonical State storage.

use std::collections::BTreeMap;

use crate::{
    serialization_compatibility, ApplicationSemanticModel, ComponentInstanceId,
    ComponentInstanceStatus, IntermediateRepresentation, IrStorage, IrStorageId, SemanticId,
    SemanticType, SerializableValue, SerializationCompatibility, SourceProvenance,
    StateInstanceSlotId,
};

pub const STATE_INSTANCE_STORAGE_REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateInstanceStorageRecord {
    pub slot_id: StateInstanceSlotId,
    pub component_instance_id: ComponentInstanceId,
    pub component_id: SemanticId,
    pub state_id: SemanticId,
    pub storage_id: IrStorageId,
    pub initial_value: SerializableValue,
    pub semantic_type: SemanticType,
    pub serialization: SerializationCompatibility,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateInstanceStorageRegistry {
    pub version: u32,
    pub records: Vec<StateInstanceStorageRecord>,
}

impl StateInstanceStorageRegistry {
    #[must_use]
    pub fn record(
        &self,
        component_instance_id: &ComponentInstanceId,
        storage_id: &IrStorageId,
    ) -> Option<&StateInstanceStorageRecord> {
        self.records.iter().find(|record| {
            record.component_instance_id == *component_instance_id
                && record.storage_id == *storage_id
        })
    }

    pub fn records_for_instance<'a>(
        &'a self,
        component_instance_id: &'a ComponentInstanceId,
    ) -> impl Iterator<Item = &'a StateInstanceStorageRecord> + 'a {
        self.records
            .iter()
            .filter(move |record| record.component_instance_id == *component_instance_id)
    }
}

#[must_use]
pub fn build_state_instance_storage_registry(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
) -> StateInstanceStorageRegistry {
    let storages = ir
        .modules
        .iter()
        .flat_map(|module| &module.storages)
        .map(|storage| (storage.semantic_origin.clone(), storage))
        .collect::<BTreeMap<_, _>>();
    let mut records = Vec::new();

    for instance in model.component_instance_plan.instances.values() {
        if instance.status != ComponentInstanceStatus::Planned {
            continue;
        }
        let Some(component) = model
            .components
            .iter()
            .find(|component| component.id == instance.component)
        else {
            continue;
        };
        let mut component_storages = component
            .state_fields
            .iter()
            .filter_map(|state| {
                canonical_state_storage_record(model, instance.id.clone(), &storages, state)
            })
            .collect::<Vec<_>>();
        component_storages.sort_by(|left, right| left.storage_id.cmp(&right.storage_id));
        records.extend(component_storages);
    }

    StateInstanceStorageRegistry {
        version: STATE_INSTANCE_STORAGE_REGISTRY_VERSION,
        records,
    }
}

fn canonical_state_storage_record(
    model: &ApplicationSemanticModel,
    component_instance_id: ComponentInstanceId,
    storages: &BTreeMap<SemanticId, &IrStorage>,
    state: &crate::StateField,
) -> Option<StateInstanceStorageRecord> {
    let storage = storages.get(&state.id)?;
    let initial_value = storage.initial_value.clone()?;
    let semantic_type = model.semantic_type_of(&state.id)?.clone();
    let component_id = state.owner.entity_id()?.clone();
    Some(StateInstanceStorageRecord {
        slot_id: StateInstanceSlotId::for_component_instance_storage(
            component_instance_id.clone(),
            storage.id.clone(),
        ),
        component_instance_id,
        component_id,
        state_id: state.id.clone(),
        storage_id: storage.id.clone(),
        initial_value,
        serialization: serialization_compatibility(&semantic_type),
        semantic_type,
        provenance: storage.provenance.clone(),
    })
}

/// # Errors
///
/// Returns an error when the retained registry is not the exact canonical
/// projection of component instances and definition-level IR storage.
pub fn validate_state_instance_storage_registry(
    model: &ApplicationSemanticModel,
    ir: &IntermediateRepresentation,
    registry: &StateInstanceStorageRegistry,
) -> Result<(), String> {
    if registry.version != STATE_INSTANCE_STORAGE_REGISTRY_VERSION {
        return Err("unsupported State instance storage registry version".to_string());
    }
    if registry != &build_state_instance_storage_registry(model, ir) {
        return Err("State instance storage registry drifted from canonical products".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_instances_receive_distinct_slots_for_one_ir_storage() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/RepeatedState.tsx",
            r#"@component("x-child") class Child { count = state(1); render() { return <button>{this.count}</button>; } }
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }"#,
        ));
        let ir = crate::lower_components_to_ir(&model);
        let registry = build_state_instance_storage_registry(&model, &ir);
        assert_eq!(registry.records.len(), 2);
        assert_eq!(
            registry.records[0].storage_id,
            registry.records[1].storage_id
        );
        assert_ne!(registry.records[0].slot_id, registry.records[1].slot_id);
        assert_eq!(
            registry
                .records_for_instance(&registry.records[0].component_instance_id)
                .count(),
            1
        );
        assert_eq!(
            registry
                .record(
                    &registry.records[1].component_instance_id,
                    &registry.records[1].storage_id
                )
                .map(|record| &record.slot_id),
            Some(&registry.records[1].slot_id)
        );
        assert!(validate_state_instance_storage_registry(&model, &ir, &registry).is_ok());
    }

    #[test]
    fn registry_is_deterministic_under_reversed_compilation_input() {
        let first = presolve_parser::parse_file(
            "src/Child.tsx",
            r#"@component("x-child") class Child { count = state(1); render() { return <b>{this.count}</b>; } }"#,
        );
        let second = presolve_parser::parse_file(
            "src/Page.tsx",
            r#"@component("x-page") class Page { render() { return <><Child /><Child /></>; } }"#,
        );
        let forward = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![first.clone(), second.clone()]),
        );
        let reverse = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![second, first]),
        );
        assert_eq!(
            build_state_instance_storage_registry(
                &forward,
                &crate::lower_components_to_ir(&forward)
            ),
            build_state_instance_storage_registry(
                &reverse,
                &crate::lower_components_to_ir(&reverse)
            )
        );
    }
}
