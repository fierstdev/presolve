use serde::{Deserialize, Serialize};

use crate::{
    build_runtime_component_registry, ApplicationSemanticModel, OptimizedComponentIrReport,
    RuntimeComponentRegistry,
};

pub const RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION: u32 = 1;

/// Public H14 compiler artifact. All executable references are canonical IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeComponentArtifact {
    pub schema_version: u32,
    pub component_definitions: Vec<SerializedComponentDefinition>,
    pub instances: Vec<SerializedComponentInstance>,
    pub initialization_batches: Vec<SerializedComponentBatch>,
    pub slot_binding_programs: Vec<SerializedSlotBinding>,
    pub instance_context_bindings: Vec<SerializedInstanceContextBinding>,
    pub destruction: SerializedDestructionMetadata,
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
    pub storage_prefix: String,
    pub cache_prefix: String,
    pub context_prefix: String,
    pub instruction_indices: Vec<usize>,
    pub structural_region: Option<String>,
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

#[must_use]
pub fn build_runtime_component_artifact(
    model: &ApplicationSemanticModel,
    optimized: &OptimizedComponentIrReport,
) -> RuntimeComponentArtifact {
    artifact_from_registry(&build_runtime_component_registry(model, optimized))
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
                storage_prefix: r.instance_storage_prefix.clone(),
                cache_prefix: r.instance_cache_prefix.clone(),
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
        destruction: SerializedDestructionMetadata {
            operation: "destroy_component_instance".to_string(),
            enabled: false,
        },
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
pub fn validate_runtime_component_artifact(
    artifact: &RuntimeComponentArtifact,
) -> Result<(), String> {
    if artifact.schema_version != RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION {
        return Err("unsupported component runtime artifact schema version".to_string());
    }
    let instances = artifact
        .instances
        .iter()
        .map(|r| r.instance.as_str())
        .collect::<std::collections::BTreeSet<_>>();
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

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_component_artifact,
        runtime_component_artifact_json, validate_runtime_component_artifact,
    };
    #[test]
    fn serializes_deterministically_and_rejects_unknown_endpoints() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
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
}
