use serde::{Deserialize, Serialize};

use crate::{
    build_computed_instance_slot_registry, build_ordinary_template_instance_registry,
    build_runtime_component_registry, build_state_instance_storage_registry,
    lower_components_to_ir, semantic_type_text, ApplicationSemanticModel,
    OptimizedComponentIrReport, OrdinaryTemplateBindingKind, OrdinaryTemplateTargetKind,
    RuntimeComponentRegistry, SerializationCompatibility,
};

pub const RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION: u32 = 4;

/// Public H14 compiler artifact. All executable references are canonical IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeComponentArtifact {
    pub schema_version: u32,
    pub component_definitions: Vec<SerializedComponentDefinition>,
    pub instances: Vec<SerializedComponentInstance>,
    pub initialization_batches: Vec<SerializedComponentBatch>,
    pub slot_binding_programs: Vec<SerializedSlotBinding>,
    pub instance_context_bindings: Vec<SerializedInstanceContextBinding>,
    pub ordinary_template_targets: Vec<SerializedOrdinaryTemplateTarget>,
    pub ordinary_template_bindings: Vec<SerializedOrdinaryTemplateBinding>,
    pub ordinary_template_events: Vec<SerializedOrdinaryTemplateEvent>,
    pub destruction: SerializedDestructionMetadata,
    pub structural_programs: Vec<SerializedStructuralComponentProgram>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponentDefinition {
    pub component: String,
    pub template: String,
    pub slots: Vec<String>,
    pub boundary: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponentInstance {
    pub instance: String,
    pub component: String,
    pub invocation: Option<String>,
    pub parent: Option<String>,
    pub depth: usize,
    pub initialization_batch: usize,
    pub ordinary_template_targets: Vec<String>,
    pub ordinary_template_bindings: Vec<String>,
    pub ordinary_template_events: Vec<String>,
    pub state_slots: Vec<SerializedRuntimeStateSlot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub computed_slots: Vec<SerializedRuntimeComputedSlot>,
    pub context_prefix: String,
    pub instruction_indices: Vec<usize>,
    pub structural_region: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedRuntimeStateSlot {
    pub slot_id: String,
    pub state_id: String,
    pub storage_id: String,
    pub initial_value: crate::SerializableValue,
    pub semantic_type: String,
    pub serializable: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedRuntimeComputedSlot {
    pub computed_id: String,
    pub cache_slot_id: String,
    pub dirty_slot_id: String,
    pub dirty_initial_value: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedOrdinaryTemplateTarget {
    pub id: String,
    pub component_instance_id: String,
    pub component_id: String,
    pub template_entity_id: String,
    pub kind: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedOrdinaryTemplateBinding {
    pub id: String,
    pub component_instance_id: String,
    pub component_id: String,
    pub declaration_binding_id: String,
    pub target_id: String,
    pub kind: String,
    pub state_storage_ids: Vec<String>,
    pub computed_ids: Vec<String>,
    pub program_id: String,
    pub expression: Option<String>,
    pub attribute_name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedOrdinaryTemplateEvent {
    pub component_instance_id: String,
    pub component_id: String,
    pub target_id: String,
    pub declaration_event_id: String,
    pub event_type: String,
    pub handler_method_id: String,
    pub action_batch_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<crate::component_graph::SerializableValue>,
    pub program_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponentBatch {
    pub index: usize,
    pub instances: Vec<String>,
    pub context_sources: Vec<String>,
    pub slot_bindings: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedSlotBinding {
    pub binding: String,
    pub caller_instance: String,
    pub callee_instance: String,
    pub slot: String,
    pub outlet: String,
    pub fragment: String,
    pub content_owner_instance: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedInstanceContextBinding {
    pub consumer_instance: String,
    pub selected_source: String,
    pub provider_source: Option<String>,
    pub runtime_slot: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedDestructionMetadata {
    pub operation: String,
    pub enabled: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedStructuralComponentProgram {
    pub region: String,
    pub template_instances: Vec<String>,
    pub destroy_order: Vec<String>,
    pub create_order: Vec<String>,
}

const fn target_kind_text(kind: OrdinaryTemplateTargetKind) -> &'static str {
    match kind {
        OrdinaryTemplateTargetKind::Element => "element",
        OrdinaryTemplateTargetKind::AttributeOrPropertyHost => "attribute_or_property_host",
        OrdinaryTemplateTargetKind::EventHost => "event_host",
        OrdinaryTemplateTargetKind::ConditionalBoundary => "conditional_boundary",
        OrdinaryTemplateTargetKind::ListBoundary => "list_boundary",
        OrdinaryTemplateTargetKind::FormControlHost => "form_control_host",
        OrdinaryTemplateTargetKind::FormSubmissionHost => "form_submission_host",
    }
}

const fn binding_kind_text(kind: OrdinaryTemplateBindingKind) -> &'static str {
    match kind {
        OrdinaryTemplateBindingKind::Text => "text",
        OrdinaryTemplateBindingKind::Attribute => "attribute",
        OrdinaryTemplateBindingKind::Property => "property",
        OrdinaryTemplateBindingKind::Conditional => "conditional",
        OrdinaryTemplateBindingKind::List => "list",
        OrdinaryTemplateBindingKind::FormControl => "form_control",
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_runtime_component_artifact(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedComponentIrReport,
) -> RuntimeComponentArtifact {
    let ordinary = build_ordinary_template_instance_registry(model);
    let ir = lower_components_to_ir(model);
    let state_slots = build_state_instance_storage_registry(model, &ir);
    let computed_slots = build_computed_instance_slot_registry(model, &ir);
    let mut artifact = artifact_from_registry(&build_runtime_component_registry(model, optimized));
    artifact.ordinary_template_targets = ordinary
        .targets
        .iter()
        .map(|target| SerializedOrdinaryTemplateTarget {
            id: target.target_id.to_string(),
            component_instance_id: target.component_instance_id.to_string(),
            component_id: target.component_id.to_string(),
            template_entity_id: target.template_entity_id.to_string(),
            kind: target_kind_text(target.target_kind).to_string(),
        })
        .collect();
    artifact.ordinary_template_bindings = ordinary
        .bindings
        .iter()
        .map(|binding| SerializedOrdinaryTemplateBinding {
            id: binding.instance_binding_id.to_string(),
            component_instance_id: binding.component_instance_id.to_string(),
            component_id: binding.component_id.to_string(),
            declaration_binding_id: binding.declaration_binding_id.to_string(),
            target_id: binding.target_id.to_string(),
            kind: binding_kind_text(binding.binding_kind).to_string(),
            state_storage_ids: binding
                .state_storage_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            computed_ids: binding
                .computed_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            program_id: binding.existing_program_identity.to_string(),
            expression: binding.expression.clone(),
            attribute_name: binding.attribute_name.clone(),
        })
        .collect();
    artifact.ordinary_template_events = ordinary
        .events
        .iter()
        .map(|event| SerializedOrdinaryTemplateEvent {
            component_instance_id: event.component_instance_id.to_string(),
            component_id: event.component_id.to_string(),
            target_id: event.target_id.to_string(),
            declaration_event_id: event.declaration_event_id.to_string(),
            event_type: event.event_type.clone(),
            handler_method_id: event.handler_method_id.to_string(),
            action_batch_id: event.action_batch_id.as_ref().map(ToString::to_string),
            arguments: event.arguments.clone(),
            program_id: event.existing_event_program_identity.to_string(),
        })
        .collect();
    for instance in &mut artifact.instances {
        instance.ordinary_template_targets = artifact
            .ordinary_template_targets
            .iter()
            .filter(|target| target.component_instance_id == instance.instance)
            .map(|target| target.id.clone())
            .collect();
        instance.ordinary_template_bindings = artifact
            .ordinary_template_bindings
            .iter()
            .filter(|binding| binding.component_instance_id == instance.instance)
            .map(|binding| binding.id.clone())
            .collect();
        instance.ordinary_template_events = artifact
            .ordinary_template_events
            .iter()
            .filter(|event| event.component_instance_id == instance.instance)
            .map(|event| event.declaration_event_id.clone())
            .collect();
        instance.state_slots = state_slots
            .records
            .iter()
            .filter(|slot| slot.component_instance_id.to_string() == instance.instance)
            .map(|slot| SerializedRuntimeStateSlot {
                slot_id: slot.slot_id.to_string(),
                state_id: slot.state_id.to_string(),
                storage_id: slot.storage_id.to_string(),
                initial_value: slot.initial_value.clone(),
                semantic_type: semantic_type_text(&slot.semantic_type),
                serializable: slot.serialization == SerializationCompatibility::Serializable,
            })
            .collect();
        instance.computed_slots = computed_slots
            .records
            .iter()
            .filter(|slot| slot.component_instance_id.to_string() == instance.instance)
            .map(|slot| SerializedRuntimeComputedSlot {
                computed_id: slot.computed_id.to_string(),
                cache_slot_id: slot.cache_slot_id.to_string(),
                dirty_slot_id: slot.dirty_slot_id.to_string(),
                dirty_initial_value: slot.dirty_initial_value,
            })
            .collect();
    }
    let mut programs = std::collections::BTreeMap::<String, Vec<String>>::new();
    for instance in model.component_instance_plan.instances.values() {
        if instance.status == crate::ComponentInstanceStatus::StructuralTemplate {
            if let Some(region) = &instance.structural_region {
                programs
                    .entry(region.to_string())
                    .or_default()
                    .push(instance.id.to_string());
            }
        }
    }
    artifact.structural_programs = programs
        .into_iter()
        .map(
            |(region, template_instances)| SerializedStructuralComponentProgram {
                region,
                create_order: template_instances.clone(),
                destroy_order: template_instances.iter().rev().cloned().collect(),
                template_instances,
            },
        )
        .collect();
    artifact
}

#[must_use]
pub fn artifact_from_registry(registry: &RuntimeComponentRegistry) -> RuntimeComponentArtifact {
    RuntimeComponentArtifact {
        schema_version: RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION,
        component_definitions: registry
            .component_definitions
            .iter()
            .map(|r| SerializedComponentDefinition {
                component: r.component.to_string(),
                template: r.template.to_string(),
                slots: r.declared_slots.iter().map(ToString::to_string).collect(),
                boundary: "client".to_string(),
            })
            .collect(),
        instances: registry
            .instances
            .iter()
            .map(|r| SerializedComponentInstance {
                instance: r.instance.to_string(),
                component: r.component.to_string(),
                invocation: r.invocation.as_ref().map(ToString::to_string),
                parent: r.parent.as_ref().map(ToString::to_string),
                depth: r.depth,
                initialization_batch: r.initialization_batch,
                ordinary_template_targets: Vec::new(),
                ordinary_template_bindings: Vec::new(),
                ordinary_template_events: Vec::new(),
                state_slots: Vec::new(),
                computed_slots: Vec::new(),
                context_prefix: r.instance_context_prefix.clone(),
                instruction_indices: r.optimized_instruction_indices.clone(),
                structural_region: r.structural_region.as_ref().map(ToString::to_string),
            })
            .collect(),
        initialization_batches: registry
            .initialization_batches
            .iter()
            .map(|r| SerializedComponentBatch {
                index: r.index,
                instances: r.instances.iter().map(ToString::to_string).collect(),
                context_sources: r.context_sources.iter().map(ToString::to_string).collect(),
                slot_bindings: r.slot_bindings.iter().map(ToString::to_string).collect(),
            })
            .collect(),
        slot_binding_programs: registry
            .slot_bindings
            .iter()
            .map(|r| SerializedSlotBinding {
                binding: r.binding.to_string(),
                caller_instance: r.caller_instance.to_string(),
                callee_instance: r.callee_instance.to_string(),
                slot: r.slot.to_string(),
                outlet: r.outlet.to_string(),
                fragment: r.fragment.to_string(),
                content_owner_instance: r.content_owner_instance.to_string(),
            })
            .collect(),
        instance_context_bindings: registry
            .instance_context_bindings
            .iter()
            .map(|r| SerializedInstanceContextBinding {
                consumer_instance: r.consumer_instance.to_string(),
                selected_source: r.selected_source.to_string(),
                provider_source: r.provider_source.as_ref().map(ToString::to_string),
                runtime_slot: r.runtime_slot.to_string(),
            })
            .collect(),
        ordinary_template_targets: Vec::new(),
        ordinary_template_bindings: Vec::new(),
        ordinary_template_events: Vec::new(),
        destruction: SerializedDestructionMetadata {
            operation: "destroy_component_instance".to_string(),
            enabled: true,
        },
        structural_programs: Vec::new(),
    }
}

/// # Panics
///
/// Panics only if compiler-owned artifact data cannot be serialized.
#[must_use]
pub fn runtime_component_artifact_json(artifact: &RuntimeComponentArtifact) -> String {
    serde_json::to_string_pretty(artifact).expect("component runtime artifact serializes") + "\n"
}

/// # Errors
///
/// Returns an error for an unsupported schema or invalid canonical endpoints/order.
#[allow(clippy::too_many_lines)]
pub fn validate_runtime_component_artifact(
    artifact: &RuntimeComponentArtifact,
) -> Result<(), String> {
    if artifact.schema_version != RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION {
        return Err("unsupported component runtime artifact schema version".to_string());
    }
    if artifact.structural_programs.iter().any(|program| {
        program.create_order != program.template_instances
            || program
                .destroy_order
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                != program.template_instances
    }) {
        return Err("component artifact has invalid structural program ordering".to_string());
    }
    let instances = artifact
        .instances
        .iter()
        .map(|r| r.instance.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let target_ids = artifact
        .ordinary_template_targets
        .iter()
        .map(|target| target.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let binding_ids = artifact
        .ordinary_template_bindings
        .iter()
        .map(|binding| binding.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let event_keys = artifact
        .ordinary_template_events
        .iter()
        .map(|event| {
            (
                event.component_instance_id.as_str(),
                event.target_id.as_str(),
                event.event_type.as_str(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut computed_cache_slots = std::collections::BTreeSet::new();
    let mut computed_dirty_slots = std::collections::BTreeSet::new();
    let mut computed_instance_pairs = std::collections::BTreeSet::new();
    let mut state_slots = std::collections::BTreeSet::new();
    let mut state_instance_pairs = std::collections::BTreeSet::new();
    if target_ids.len() != artifact.ordinary_template_targets.len()
        || binding_ids.len() != artifact.ordinary_template_bindings.len()
        || event_keys.len() != artifact.ordinary_template_events.len()
        || artifact.ordinary_template_targets.iter().any(|target| {
            !instances.contains(target.component_instance_id.as_str())
                || !target.id.starts_with(&format!(
                    "{}/template-target:",
                    target.component_instance_id
                ))
        })
        || artifact.ordinary_template_bindings.iter().any(|binding| {
            !instances.contains(binding.component_instance_id.as_str())
                || !target_ids.contains(binding.target_id.as_str())
                || !binding.id.starts_with(&format!(
                    "{}/template-binding:",
                    binding.component_instance_id
                ))
        })
        || artifact.ordinary_template_events.iter().any(|event| {
            !instances.contains(event.component_instance_id.as_str())
                || !target_ids.contains(event.target_id.as_str())
                || event.action_batch_id.is_none()
        })
        || artifact.instances.iter().any(|instance| {
            instance
                .ordinary_template_targets
                .iter()
                .any(|id| !target_ids.contains(id.as_str()))
                || instance
                    .ordinary_template_bindings
                    .iter()
                    .any(|id| !binding_ids.contains(id.as_str()))
                || instance.ordinary_template_targets
                    != artifact
                        .ordinary_template_targets
                        .iter()
                        .filter(|target| target.component_instance_id == instance.instance)
                        .map(|target| target.id.clone())
                        .collect::<Vec<_>>()
                || instance.ordinary_template_bindings
                    != artifact
                        .ordinary_template_bindings
                        .iter()
                        .filter(|binding| binding.component_instance_id == instance.instance)
                        .map(|binding| binding.id.clone())
                        .collect::<Vec<_>>()
                || instance.ordinary_template_events
                    != artifact
                        .ordinary_template_events
                        .iter()
                        .filter(|event| event.component_instance_id == instance.instance)
                        .map(|event| event.declaration_event_id.clone())
                        .collect::<Vec<_>>()
                || instance.state_slots.iter().any(|slot| {
                    slot.slot_id != canonical_state_slot_text(&instance.instance, &slot.storage_id)
                        || slot.storage_id != format!("storage:{}", slot.state_id)
                        || !state_slots.insert(slot.slot_id.as_str())
                        || !state_instance_pairs
                            .insert((instance.instance.as_str(), slot.storage_id.as_str()))
                })
                || instance.computed_slots.iter().any(|slot| {
                    !slot
                        .cache_slot_id
                        .starts_with(&format!("{}/computed-cache:", instance.instance))
                        || !slot
                            .dirty_slot_id
                            .starts_with(&format!("{}/computed-dirty:", instance.instance))
                        || !computed_cache_slots.insert(slot.cache_slot_id.as_str())
                        || !computed_dirty_slots.insert(slot.dirty_slot_id.as_str())
                        || !computed_instance_pairs
                            .insert((instance.instance.as_str(), slot.computed_id.as_str()))
                })
        })
    {
        return Err("component artifact has invalid ordinary template projection".to_string());
    }
    if artifact.instances.iter().any(|r| {
        r.parent
            .as_deref()
            .is_some_and(|parent| !instances.contains(parent))
    }) {
        return Err("component artifact has an unknown parent instance".to_string());
    }
    if artifact.slot_binding_programs.iter().any(|r| {
        !instances.contains(r.caller_instance.as_str())
            || !instances.contains(r.callee_instance.as_str())
    }) {
        return Err("component artifact has an unknown Slot-binding endpoint".to_string());
    }
    if artifact
        .initialization_batches
        .iter()
        .enumerate()
        .any(|(index, batch)| {
            batch.index != index
                || batch
                    .instances
                    .iter()
                    .any(|id| !instances.contains(id.as_str()))
        })
    {
        return Err("component artifact has invalid initialization ordering".to_string());
    }
    Ok(())
}

fn canonical_state_slot_text(instance: &str, storage: &str) -> String {
    let encoded = storage.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
        encoded
    });
    format!("{instance}/state-slot:{encoded}")
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_component_artifact,
        runtime_component_artifact_json, validate_runtime_component_artifact,
    };
    #[test]
    fn serializes_deterministically_and_rejects_unknown_endpoints() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Artifact.tsx",
            r#"@component("x-page") class Page extends Component { render() { return <main />; } }"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        assert!(validate_runtime_component_artifact(&artifact).is_ok());
        assert_eq!(
            runtime_component_artifact_json(&artifact),
            runtime_component_artifact_json(&build_runtime_component_artifact(
                &model,
                &model.component_ir_optimization
            ))
        );
        artifact.instances[0].parent = Some("missing".to_string());
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }

    #[test]
    fn projects_exact_computed_slots_for_each_repeated_instance() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/RepeatedComputedArtifact.tsx",
            r#"@component("x-child") class Child { count = state(1); @computed() get doubled() { return this.count * 2; } render() { return <span>{this.doubled}</span>; } }
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        let slots = artifact
            .instances
            .iter()
            .flat_map(|instance| instance.computed_slots.iter())
            .collect::<Vec<_>>();
        assert_eq!(slots.len(), 2);
        assert_ne!(slots[0].cache_slot_id, slots[1].cache_slot_id);
        assert_ne!(slots[0].dirty_slot_id, slots[1].dirty_slot_id);
        assert!(validate_runtime_component_artifact(&artifact).is_ok());
        let computed_instances = artifact
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.computed_slots.is_empty())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let duplicate = artifact.instances[computed_instances[0]].computed_slots[0].clone();
        artifact.instances[computed_instances[1]].computed_slots[0].cache_slot_id =
            duplicate.cache_slot_id;
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }

    #[test]
    fn projects_and_validates_exact_state_slots_for_repeated_instances() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/RepeatedStateArtifact.tsx",
            r#"@component("x-child") class Child { count = state(1); render() { return <span>{this.count}</span>; } }
@component("x-parent") class Parent { render() { return <><Child /><Child /></>; } }"#,
        ));
        let mut artifact =
            build_runtime_component_artifact(&model, &model.component_ir_optimization);
        let state_instances = artifact
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| !instance.state_slots.is_empty())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        assert_eq!(state_instances.len(), 2);
        let first = &artifact.instances[state_instances[0]].state_slots[0];
        let second = &artifact.instances[state_instances[1]].state_slots[0];
        assert_eq!(first.storage_id, second.storage_id);
        assert_ne!(first.slot_id, second.slot_id);
        assert!(validate_runtime_component_artifact(&artifact).is_ok());

        artifact.instances[state_instances[1]].state_slots[0].slot_id = first.slot_id.clone();
        assert!(validate_runtime_component_artifact(&artifact).is_err());
    }
}
