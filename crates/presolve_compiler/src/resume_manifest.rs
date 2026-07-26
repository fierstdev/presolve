//! J9 sole-authority executable resume manifest v7.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::effect::{EffectExecutionPolicy, EffectRenderBoundary};
use crate::effect_resume::{EffectActivationSlotId, EffectActivationStatus};
use crate::semantic_type::ExecutionBoundary;
use crate::{
    build_resume_activation_plan, build_resume_anchor_plan, build_resume_boundary_graph,
    build_resume_capture_plan, build_resume_chunk_graph, build_resume_liveness_plan,
    build_resume_plan, build_resume_restore_plan, build_resume_schema_registry,
    build_runtime_component_artifact, build_runtime_computed_artifact,
    build_runtime_context_artifact, build_runtime_effect_artifact, build_runtime_forms_artifact,
    build_template_manifest_from_asm, generate_runtime_stub, lower_components_to_ir,
    optimize_context_ir, optimize_effect_ir, runtime_component_artifact_json,
    runtime_computed_artifact_json, runtime_context_artifact_json, runtime_effect_artifact_json,
    runtime_forms_artifact_json, semantic_type_text, template_manifest_json,
    ApplicationSemanticModel, ResumeActivationId, ResumeActivationPolicy,
    ResumeActivationPrerequisite, ResumeAnchorId, ResumeBoundaryId, ResumeBoundaryKind,
    ResumeBoundaryOwner, ResumeBuildId, ResumeCaptureInstruction, ResumeCaptureProgramId,
    ResumeChunkId, ResumeChunkProgram, ResumeChunkRootKind, ResumeEventId, ResumeExistingSlot,
    ResumeLivenessClassificationRef, ResumeRestoreInstruction, ResumeRestorePhase,
    ResumeRestoreProgramId, ResumeRetentionReason, ResumeSchemaId, ResumeSlotId, ResumeValueCodec,
    SourceProvenance,
};

pub const RESUME_MANIFEST_SCHEMA_VERSION: u32 = 7;
pub const RESUME_RUNTIME_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifest {
    pub schema_version: u32,
    pub build_id: ResumeBuildId,
    pub snapshot_schema_version: u32,
    pub runtime_protocol_version: u32,
    pub application_root_boundary_id: ResumeBoundaryId,
    pub boundaries: Vec<ResumeManifestBoundaryRecord>,
    pub slot_schemas: Vec<ResumeManifestSlotSchemaRecord>,
    pub capture_programs: Vec<ResumeManifestCaptureProgram>,
    pub restore_programs: Vec<ResumeManifestRestoreProgram>,
    pub chunks: Vec<ResumeManifestChunkRecord>,
    pub activations: Vec<ResumeManifestActivationRecord>,
    pub anchors: Vec<ResumeManifestAnchorRecord>,
    pub events: Vec<ResumeManifestEventRecord>,
    pub phase_i_component_resume_records: Vec<ResumeManifestPhaseIComponentResumeRecord>,
    pub phase_i_form_resume_records: Vec<crate::FormInstanceResumePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestBoundaryRecord {
    pub boundary_id: ResumeBoundaryId,
    pub kind: ResumeManifestBoundaryKind,
    pub owner_instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_boundary_id: Option<ResumeBoundaryId>,
    pub child_boundary_ids: Vec<ResumeBoundaryId>,
    pub activation_policy: ResumeManifestActivationPolicy,
    pub schema_id: ResumeSchemaId,
    pub capture_program_id: ResumeCaptureProgramId,
    pub restore_program_id: ResumeRestoreProgramId,
    pub anchor_ids: Vec<ResumeAnchorId>,
    pub event_ids: Vec<ResumeEventId>,
    pub provenance: ResumeManifestProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestBoundaryKind {
    ApplicationRoot,
    ComponentInstance,
    StructuralRegion,
    FormInstance,
    Interaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestActivationPolicy {
    Eager,
    Visible,
    Interaction,
    Manual,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestProvenance {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestSlotSchemaRecord {
    pub slot_id: ResumeSlotId,
    pub existing_storage_slot_id: String,
    pub owner_boundary_id: ResumeBoundaryId,
    pub semantic_type: String,
    pub value_codec: ResumeValueCodec,
    pub retention_reason: ResumeManifestRetentionReason,
    pub restore_phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recompute_program_id: Option<String>,
    pub provenance: ResumeManifestProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestRetentionReason {
    MutableState,
    SerializableResourceValue,
    RequiredContextValue,
    FormValue,
    FormDirty,
    FormTouched,
    FormRuleResult,
    FormSubmissionState,
    StructuralSelection,
    RequiredComputedCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestCaptureProgram {
    pub program_id: ResumeCaptureProgramId,
    pub boundary_id: ResumeBoundaryId,
    pub instructions: Vec<ResumeManifestCaptureInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResumeManifestCaptureInstruction {
    ReadSlot {
        slot_id: ResumeSlotId,
    },
    EncodeSlot {
        slot_id: ResumeSlotId,
        codec: ResumeValueCodec,
    },
    AppendValueRecord {
        value_record_id: crate::ResumeValueRecordId,
        slot_id: ResumeSlotId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestRestoreProgram {
    pub program_id: ResumeRestoreProgramId,
    pub boundary_id: ResumeBoundaryId,
    pub instructions: Vec<ResumeManifestRestoreInstructionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestRestoreInstructionRecord {
    pub phase: String,
    pub instruction: ResumeManifestRestoreInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResumeManifestRestoreInstruction {
    AllocateBoundaryRuntimeRecord,
    DecodeValue {
        slot_id: ResumeSlotId,
        codec: ResumeValueCodec,
    },
    WriteSlot {
        slot_id: ResumeSlotId,
    },
    RestoreStructuralSelection {
        region_id: String,
        slot_id: ResumeSlotId,
    },
    RestoreFormSlot {
        form_instance_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        field_id: Option<String>,
        slot_id: ResumeSlotId,
    },
    RecomputeComputed {
        component_instance_id: String,
        computed_id: String,
    },
    BindContextConsumer {
        consumer_instance_id: String,
        selected_source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_instance_id: Option<String>,
        value_slot_id: String,
    },
    InstallDomBinding {
        binding_id: String,
        anchor_id: ResumeAnchorId,
    },
    InstallEffectSubscription {
        effect_id: String,
    },
    MarkBoundaryRestored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestChunkRecord {
    pub chunk_id: ResumeChunkId,
    pub root_kind: ResumeManifestChunkRootKind,
    pub root_id: String,
    pub module_path: String,
    pub content_hash: String,
    pub required_boundary_ids: Vec<ResumeBoundaryId>,
    pub provided_program_ids: Vec<String>,
    pub dependency_chunk_ids: Vec<ResumeChunkId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestChunkRootKind {
    Eager,
    Interaction,
    Visible,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestActivationRecord {
    pub activation_id: ResumeActivationId,
    pub boundary_id: ResumeBoundaryId,
    pub policy: ResumeManifestActivationPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<ResumeEventId>,
    pub chunk_id: ResumeChunkId,
    pub prerequisite_boundary_ids: Vec<ResumeBoundaryId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestAnchorRecord {
    pub anchor_id: ResumeAnchorId,
    pub kind: String,
    pub boundary_id: ResumeBoundaryId,
    pub exact_target_id: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeManifestEventRecord {
    pub resume_event_id: ResumeEventId,
    pub existing_dom_event_id: String,
    pub event_type: String,
    pub event_phase: String,
    pub exact_target_anchor_id: ResumeAnchorId,
    pub owner_boundary_id: ResumeBoundaryId,
    pub action_or_submit_program_id: String,
    pub chunk_id: ResumeChunkId,
    pub native_default_policy: String,
    pub propagation_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "record_kind", rename_all = "snake_case")]
pub enum ResumeManifestPhaseIComponentResumeRecord {
    Component {
        component: crate::ResumeComponentPlan,
    },
    Effect {
        effect: ResumeManifestEffectRecord,
    },
    ContextSlot {
        context_slot: ResumeManifestContextSlotRecord,
    },
    ComponentInstance {
        component_instance: crate::ComponentInstanceResumePlan,
    },
    StructuralRegion {
        structural_region: crate::StructuralRegionResumePlan,
    },
    SlotBinding {
        slot_binding: crate::SlotBindingResumePlan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifestContextSlotRecord {
    pub source: String,
    pub context_id: String,
    pub runtime_slot_id: String,
    pub resume_slot_id: crate::ContextResumeSlotId,
    pub semantic_type: String,
    pub source_kind: ResumeManifestContextSourceKind,
    pub initial_status: crate::ContextSlotResumeStatus,
    pub action_batch_ids: Vec<String>,
    pub consumer_ids: Vec<String>,
    pub execution_boundary: ResumeManifestEffectExecutionBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestContextSourceKind {
    Provider,
    ContextDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifestEffectRecord {
    pub effect_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_slot_id: Option<EffectActivationSlotId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_status: Option<EffectActivationStatus>,
    pub execution_policy: ResumeManifestEffectExecutionPolicy,
    pub runtime_function_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_plan: Option<ResumeManifestInitialEffectPlan>,
    pub action_batch_ids: Vec<String>,
    pub execution_boundary: ResumeManifestEffectExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeManifestInitialEffectPlan {
    pub render_boundary: ResumeManifestEffectRenderBoundary,
    pub batch_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestEffectExecutionPolicy {
    AfterInitialRenderAndCompletedActionBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestEffectRenderBoundary {
    AfterInitialRender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeManifestEffectExecutionBoundary {
    Client,
    Server,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeManifestValidationDiagnostic {
    pub code: String,
    pub message: String,
}

/// Builds the sole executable resume-manifest authority from canonical ASM and
/// Phase J products.
///
/// # Panics
///
/// Panics when the semantic model has no canonical application-root boundary.
/// Callers must validate the semantic model before emitting build artifacts.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_resume_manifest(model: &ApplicationSemanticModel) -> ResumeManifest {
    let boundaries = build_resume_boundary_graph(model);
    let activation = build_resume_activation_plan(model);
    let anchor_plan = build_resume_anchor_plan(model);
    let schemas = build_resume_schema_registry(model);
    let liveness = build_resume_liveness_plan(model);
    let capture = build_resume_capture_plan(model);
    let restore = build_resume_restore_plan(model);
    let chunks = build_resume_chunk_graph(model);
    let phase_i = build_resume_plan(model);
    let application_root_boundary_id = boundaries
        .boundaries
        .iter()
        .find(|boundary| boundary.kind == ResumeBoundaryKind::ApplicationRoot)
        .map(|boundary| boundary.id.clone())
        .expect("canonical resume graph has an application root");
    let restore_assignments = restore
        .programs
        .iter()
        .flat_map(|program| {
            program
                .slot_assignments
                .iter()
                .map(|assignment| (assignment.slot.clone(), assignment))
        })
        .collect::<BTreeMap<_, _>>();
    let anchor_ids_by_boundary =
        anchor_plan
            .anchors
            .iter()
            .fold(BTreeMap::<_, Vec<_>>::new(), |mut index, anchor| {
                index
                    .entry(anchor.boundary_id.clone())
                    .or_default()
                    .push(anchor.anchor_id.clone());
                index
            });
    let event_ids_by_boundary =
        anchor_plan
            .events
            .iter()
            .fold(BTreeMap::<_, Vec<_>>::new(), |mut index, event| {
                index
                    .entry(event.interaction_boundary_id.clone())
                    .or_default()
                    .push(event.resume_event_id.clone());
                index
            });
    let event_by_interaction = anchor_plan
        .events
        .iter()
        .map(|event| {
            (
                event.interaction_boundary_id.clone(),
                event.resume_event_id.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let boundary_records = boundaries
        .boundaries
        .iter()
        .map(|boundary| ResumeManifestBoundaryRecord {
            boundary_id: boundary.id.clone(),
            kind: manifest_boundary_kind(boundary.kind),
            owner_instance_id: boundary_owner_text(&boundary.owner),
            parent_boundary_id: boundary.ownership_parent.clone(),
            child_boundary_ids: boundaries.children(&boundary.id).to_vec(),
            activation_policy: activation
                .decision(&boundary.id)
                .map_or(ResumeManifestActivationPolicy::None, |decision| {
                    manifest_activation_policy(decision.policy)
                }),
            schema_id: ResumeSchemaId::for_boundary(&boundary.id),
            capture_program_id: ResumeCaptureProgramId::for_boundary(&boundary.id),
            restore_program_id: ResumeRestoreProgramId::for_boundary(&boundary.id),
            anchor_ids: anchor_ids_by_boundary
                .get(&boundary.id)
                .cloned()
                .unwrap_or_default(),
            event_ids: event_ids_by_boundary
                .get(&boundary.id)
                .cloned()
                .unwrap_or_default(),
            provenance: manifest_provenance(&boundary.provenance),
        })
        .collect();
    let slot_schemas = schemas
        .schemas
        .iter()
        .flat_map(|schema| {
            schema.slots.iter().filter_map(|slot| {
                let classification = liveness.classification(&slot.existing_slot)?;
                let (reason, recompute_program_id) = match classification {
                    ResumeLivenessClassificationRef::Retained(record) => {
                        (manifest_retention_reason(record.reason), None)
                    }
                    ResumeLivenessClassificationRef::Recomputable(record) => (
                        manifest_retention_reason(record.reason),
                        recompute_program_id(&slot.existing_slot, model),
                    ),
                    ResumeLivenessClassificationRef::Excluded(_)
                    | ResumeLivenessClassificationRef::Blocked(_) => return None,
                };
                let assignment = restore_assignments.get(&slot.existing_slot)?;
                Some(ResumeManifestSlotSchemaRecord {
                    slot_id: slot.resume_slot_id.clone(),
                    existing_storage_slot_id: slot.existing_slot.text(),
                    owner_boundary_id: schema.boundary.clone(),
                    semantic_type: semantic_type_text(&slot.semantic_type),
                    value_codec: slot.codec.clone(),
                    retention_reason: reason,
                    restore_phase: restore_phase_label(assignment.phase).to_string(),
                    recompute_program_id,
                    provenance: manifest_provenance(&slot.provenance),
                })
            })
        })
        .collect();
    let capture_programs = capture
        .programs
        .iter()
        .map(|program| ResumeManifestCaptureProgram {
            program_id: program.program_id.clone(),
            boundary_id: program.boundary_id.clone(),
            instructions: program
                .instructions
                .iter()
                .map(manifest_capture_instruction)
                .collect(),
        })
        .collect();
    let restore_programs = restore
        .programs
        .iter()
        .map(|program| ResumeManifestRestoreProgram {
            program_id: program.program_id.clone(),
            boundary_id: program.boundary_id.clone(),
            instructions: program
                .instructions
                .iter()
                .map(|record| ResumeManifestRestoreInstructionRecord {
                    phase: restore_phase_label(record.phase).to_string(),
                    instruction: manifest_restore_instruction(&record.instruction),
                })
                .collect(),
        })
        .collect();
    let chunk_records = chunks
        .chunks
        .iter()
        .map(|chunk| ResumeManifestChunkRecord {
            chunk_id: chunk.id.clone(),
            root_kind: manifest_chunk_kind(chunk.root_kind),
            root_id: chunk
                .root_boundary
                .as_ref()
                .map_or_else(|| "application".to_string(), ToString::to_string),
            module_path: chunk.module.module_path.clone(),
            content_hash: chunk.module.content_hash.clone(),
            required_boundary_ids: chunk.required_boundaries.clone(),
            provided_program_ids: chunk.programs.iter().map(chunk_program_id).collect(),
            dependency_chunk_ids: chunk.dependency_chunks.clone(),
        })
        .collect();
    let activation_records = activation
        .decisions
        .iter()
        .filter_map(|decision| {
            let chunk_id = match decision.policy {
                ResumeActivationPolicy::Eager => Some(chunks.eager_chunk.clone()),
                ResumeActivationPolicy::Visible
                | ResumeActivationPolicy::Interaction
                | ResumeActivationPolicy::Manual => chunks
                    .chunk_for_root(&decision.boundary)
                    .map(|chunk| chunk.id.clone()),
                ResumeActivationPolicy::None => None,
            }?;
            Some(ResumeManifestActivationRecord {
                activation_id: ResumeActivationId::for_boundary(&decision.boundary),
                boundary_id: decision.boundary.clone(),
                policy: manifest_activation_policy(decision.policy),
                event_id: event_by_interaction.get(&decision.boundary).cloned(),
                chunk_id,
                prerequisite_boundary_ids: decision
                    .prerequisites
                    .iter()
                    .filter_map(|prerequisite| match prerequisite {
                        ResumeActivationPrerequisite::ExactInteraction(boundary)
                        | ResumeActivationPrerequisite::RequiredBoundary(boundary) => {
                            Some(boundary.clone())
                        }
                        _ => None,
                    })
                    .collect(),
            })
        })
        .collect();
    let phase_i_component_resume_records = phase_i_component_records(&phase_i);
    let anchor_records = anchor_plan
        .anchors
        .iter()
        .map(|anchor| ResumeManifestAnchorRecord {
            anchor_id: anchor.anchor_id.clone(),
            kind: anchor.kind.label().to_string(),
            boundary_id: anchor.boundary_id.clone(),
            exact_target_id: anchor.exact_target_id.clone(),
            required: anchor.required,
        })
        .collect();
    let event_records = anchor_plan
        .events
        .iter()
        .map(|event| ResumeManifestEventRecord {
            resume_event_id: event.resume_event_id.clone(),
            existing_dom_event_id: event.existing_dom_event_id.clone(),
            event_type: event.event_type.clone(),
            event_phase: event.event_phase.clone(),
            exact_target_anchor_id: event.exact_target_anchor_id.clone(),
            owner_boundary_id: event.owner_boundary_id.clone(),
            action_or_submit_program_id: event.action_or_submit_program_id.clone(),
            chunk_id: event.chunk_id.clone(),
            native_default_policy: event.native_default_policy.clone(),
            propagation_policy: event.propagation_policy.clone(),
        })
        .collect();
    let mut manifest = ResumeManifest {
        schema_version: RESUME_MANIFEST_SCHEMA_VERSION,
        build_id: ResumeBuildId::zero_sentinel(),
        snapshot_schema_version: crate::RESUME_SNAPSHOT_SCHEMA_VERSION,
        runtime_protocol_version: RESUME_RUNTIME_PROTOCOL_VERSION,
        application_root_boundary_id,
        boundaries: boundary_records,
        slot_schemas,
        capture_programs,
        restore_programs,
        chunks: chunk_records,
        activations: activation_records,
        anchors: anchor_records,
        events: event_records,
        phase_i_component_resume_records,
        phase_i_form_resume_records: phase_i.form_instances,
    };
    manifest.build_id = compute_resume_build_id(model, &manifest);
    manifest
}

#[must_use]
pub fn compute_resume_build_id(
    model: &ApplicationSemanticModel,
    manifest: &ResumeManifest,
) -> ResumeBuildId {
    ResumeBuildId::for_public_inputs(&resume_build_fingerprint(model, manifest))
}

fn resume_build_fingerprint(model: &ApplicationSemanticModel, manifest: &ResumeManifest) -> String {
    let mut fingerprint = manifest.clone();
    fingerprint.build_id = ResumeBuildId::zero_sentinel();
    for boundary in &mut fingerprint.boundaries {
        boundary.provenance = ResumeManifestProvenance {
            path: String::new(),
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        };
    }
    for slot in &mut fingerprint.slot_schemas {
        slot.provenance = ResumeManifestProvenance {
            path: String::new(),
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        };
    }
    let ir = lower_components_to_ir(model);
    let computed = runtime_computed_artifact_json(&build_runtime_computed_artifact(model, &ir));
    let context_ir = optimize_context_ir(&ir);
    let context =
        runtime_context_artifact_json(&build_runtime_context_artifact(model, &context_ir));
    let effect_ir = optimize_effect_ir(&ir).output;
    let effects = runtime_effect_artifact_json(&build_runtime_effect_artifact(model, &effect_ir));
    let components = runtime_component_artifact_json(&build_runtime_component_artifact(
        model,
        &model.component_ir_optimization,
    ));
    let forms = runtime_forms_artifact_json(&build_runtime_forms_artifact(model));
    let template = template_manifest_json(&build_template_manifest_from_asm(model));
    let chunks = build_resume_chunk_graph(model);
    for (record, chunk) in fingerprint.chunks.iter_mut().zip(&chunks.chunks) {
        let normalized_bytes =
            normalize_absolute_source_root(model, &chunk.module.canonical_module_bytes);
        let normalized_hash = ResumeBuildId::for_public_inputs(&normalized_bytes)
            .as_str()
            .strip_prefix("resume-build:")
            .expect("build hash prefix")
            .to_string();
        record.content_hash.clone_from(&normalized_hash);
        record.module_path = normalized_chunk_module_path(model, record, &normalized_hash);
    }
    normalize_marker_identities(&mut fingerprint);
    let runtime = generate_runtime_stub();
    let manifest_bytes = canonical_manifest_json(&fingerprint);
    let marker_plan = serde_json::to_string(&(&fingerprint.anchors, &fingerprint.events))
        .expect("resume marker plan should serialize");
    let mut input = String::new();
    for (label, bytes) in [
        ("template", template.as_str()),
        ("computed", computed.as_str()),
        ("context", context.as_str()),
        ("effects", effects.as_str()),
        ("components", components.as_str()),
        ("forms", forms.as_str()),
        ("runtime", runtime.as_str()),
        ("manifest", manifest_bytes.as_str()),
        ("markers", marker_plan.as_str()),
        ("runtime_protocol", "1"),
        ("snapshot_schema", "1"),
    ] {
        append_framed_input(
            &mut input,
            label,
            &normalize_absolute_source_root(model, bytes),
        );
    }
    for chunk in &chunks.chunks {
        let chunk_label = normalize_absolute_source_root(model, &format!("chunk:{}", chunk.id));
        append_framed_input(
            &mut input,
            &chunk_label,
            &normalize_absolute_source_root(model, &chunk.module.canonical_module_bytes),
        );
    }
    input
}

fn normalize_marker_identities(manifest: &mut ResumeManifest) {
    let anchor_ids = manifest
        .anchors
        .iter()
        .enumerate()
        .map(|(index, anchor)| {
            (
                anchor.anchor_id.clone(),
                format!("presolve-r:canonical-{index}")
                    .parse::<ResumeAnchorId>()
                    .expect("canonical anchor placeholder"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let event_ids = manifest
        .events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            (
                event.resume_event_id.clone(),
                format!("resume-event:canonical-{index}")
                    .parse::<ResumeEventId>()
                    .expect("canonical event placeholder"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for anchor in &mut manifest.anchors {
        anchor.anchor_id = anchor_ids[&anchor.anchor_id].clone();
    }
    for event in &mut manifest.events {
        event.resume_event_id = event_ids[&event.resume_event_id].clone();
        event.exact_target_anchor_id = anchor_ids[&event.exact_target_anchor_id].clone();
    }
    for boundary in &mut manifest.boundaries {
        for anchor in &mut boundary.anchor_ids {
            *anchor = anchor_ids[anchor].clone();
        }
        for event in &mut boundary.event_ids {
            *event = event_ids[event].clone();
        }
    }
    for activation in &mut manifest.activations {
        if let Some(event) = &mut activation.event_id {
            *event = event_ids[event].clone();
        }
    }
}

fn normalized_chunk_module_path(
    model: &ApplicationSemanticModel,
    record: &ResumeManifestChunkRecord,
    content_hash: &str,
) -> String {
    let kind = match record.root_kind {
        ResumeManifestChunkRootKind::Eager => "boot",
        ResumeManifestChunkRootKind::Interaction => "event",
        ResumeManifestChunkRootKind::Visible => "visible",
        ResumeManifestChunkRootKind::Manual => "manual",
    };
    let root = normalize_absolute_source_root(model, &record.root_id);
    let safe_root = root
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("{kind}.{safe_root}.{content_hash}.js")
}

fn normalize_absolute_source_root(model: &ApplicationSemanticModel, input: &str) -> String {
    let paths = model
        .provenance
        .values()
        .map(|provenance| provenance.path.as_path())
        .filter(|path| path.is_absolute())
        .collect::<BTreeSet<_>>();
    let Some(root) = common_source_root(paths.iter().copied()) else {
        return input.to_string();
    };
    let root = root.to_string_lossy();
    input.replace(root.as_ref(), "__SOURCE_ROOT__").replace(
        &percent_encode_build_path(root.as_ref()),
        "__SOURCE_ROOT_ENCODED__",
    )
}

fn common_source_root<'a>(mut paths: impl Iterator<Item = &'a Path>) -> Option<PathBuf> {
    let first = paths.next()?;
    let mut root = first.parent()?.to_path_buf();
    for path in paths {
        while !path.starts_with(&root) {
            if !root.pop() {
                return None;
            }
        }
    }
    Some(root)
}

fn percent_encode_build_path(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(char::from(byte));
        } else {
            let _ = write!(output, "%{byte:02X}");
        }
    }
    output
}

fn append_framed_input(output: &mut String, label: &str, value: &str) {
    let _ = write!(output, "{label}:{}:", value.len());
    output.push_str(value);
    output.push('\n');
}

#[must_use]
pub fn resume_manifest_json(manifest: &ResumeManifest) -> String {
    canonical_manifest_json(manifest) + "\n"
}

fn canonical_manifest_json(manifest: &ResumeManifest) -> String {
    serde_json::to_string(manifest)
        .expect("resume manifest should serialize")
        .replace('<', "\\u003c")
}

/// # Errors
///
/// Rejects non-v7 documents before deserialization and reports malformed v7
/// JSON through the same runtime-boundary diagnostic envelope.
pub fn parse_resume_manifest_v7(
    json: &str,
) -> Result<ResumeManifest, ResumeManifestValidationDiagnostic> {
    let value = serde_json::from_str::<serde_json::Value>(json).map_err(|error| {
        diagnostic(
            "PSRSM1201",
            &format!("resume manifest is not valid JSON: {error}"),
        )
    })?;
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(u64::from(RESUME_MANIFEST_SCHEMA_VERSION))
    {
        return Err(diagnostic(
            "PSRSM1201",
            "resume runtime accepts only executable manifest schema v7",
        ));
    }
    let manifest = serde_json::from_value(value).map_err(|error| {
        diagnostic(
            "PSRSM1201",
            &format!("resume manifest v7 shape is malformed: {error}"),
        )
    })?;
    if let Some(diagnostic) = validate_resume_manifest(&manifest).into_iter().next() {
        return Err(diagnostic);
    }
    Ok(manifest)
}

/// Compatibility entrypoint retained while callers migrate their symbol name.
/// It accepts only the current schema-v7 product.
pub fn parse_resume_manifest_v6(
    json: &str,
) -> Result<ResumeManifest, ResumeManifestValidationDiagnostic> {
    parse_resume_manifest_v7(json)
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_resume_manifest(
    manifest: &ResumeManifest,
) -> Vec<ResumeManifestValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != RESUME_MANIFEST_SCHEMA_VERSION
        || manifest.snapshot_schema_version != crate::RESUME_SNAPSHOT_SCHEMA_VERSION
        || manifest.runtime_protocol_version != RESUME_RUNTIME_PROTOCOL_VERSION
    {
        diagnostics.push(diagnostic(
            "PSRSM1201",
            "resume manifest version fields do not match the v7/v2/v1 contract",
        ));
        return diagnostics;
    }
    let boundary_ids = unique_ids(
        manifest.boundaries.iter().map(|record| &record.boundary_id),
        "PSRSM1202",
        "resume boundary ID is duplicated",
        &mut diagnostics,
    );
    if !boundary_ids.contains(&manifest.application_root_boundary_id) {
        diagnostics.push(diagnostic(
            "PSRSM1203",
            "application root boundary does not resolve",
        ));
    }
    let schema_ids = unique_ids(
        manifest.slot_schemas.iter().map(|record| &record.slot_id),
        "PSRSM1204",
        "resume slot ID is duplicated",
        &mut diagnostics,
    );
    let capture_ids = unique_ids(
        manifest
            .capture_programs
            .iter()
            .map(|record| &record.program_id),
        "PSRSM1205",
        "capture program ID is duplicated",
        &mut diagnostics,
    );
    let restore_ids = unique_ids(
        manifest
            .restore_programs
            .iter()
            .map(|record| &record.program_id),
        "PSRSM1206",
        "restore program ID is duplicated",
        &mut diagnostics,
    );
    let chunk_ids = unique_ids(
        manifest.chunks.iter().map(|record| &record.chunk_id),
        "PSRSM1207",
        "chunk ID is duplicated",
        &mut diagnostics,
    );
    let anchor_ids = unique_ids(
        manifest.anchors.iter().map(|record| &record.anchor_id),
        "PSRSM1208",
        "anchor ID is duplicated",
        &mut diagnostics,
    );
    let event_ids = unique_ids(
        manifest.events.iter().map(|record| &record.resume_event_id),
        "PSRSM1209",
        "event ID is duplicated",
        &mut diagnostics,
    );
    for boundary in &manifest.boundaries {
        if boundary
            .parent_boundary_id
            .as_ref()
            .is_some_and(|parent| !boundary_ids.contains(parent))
            || boundary
                .child_boundary_ids
                .iter()
                .any(|child| !boundary_ids.contains(child))
            || boundary.schema_id != ResumeSchemaId::for_boundary(&boundary.boundary_id)
            || !capture_ids.contains(&boundary.capture_program_id)
            || !restore_ids.contains(&boundary.restore_program_id)
            || boundary
                .anchor_ids
                .iter()
                .any(|anchor| !anchor_ids.contains(anchor))
            || boundary
                .event_ids
                .iter()
                .any(|event| !event_ids.contains(event))
        {
            diagnostics.push(diagnostic(
                "PSRSM1210",
                "boundary record contains an unresolved endpoint",
            ));
        }
    }
    for slot in &manifest.slot_schemas {
        if !boundary_ids.contains(&slot.owner_boundary_id) {
            diagnostics.push(diagnostic(
                "PSRSM1211",
                "slot schema owner boundary does not resolve",
            ));
        }
    }
    for program in &manifest.capture_programs {
        if !boundary_ids.contains(&program.boundary_id)
            || program
                .instructions
                .iter()
                .any(|instruction| !schema_ids.contains(capture_instruction_slot(instruction)))
        {
            diagnostics.push(diagnostic(
                "PSRSM1212",
                "capture program contains an unresolved endpoint",
            ));
        }
    }
    for program in &manifest.restore_programs {
        if !boundary_ids.contains(&program.boundary_id)
            || program.instructions.iter().any(|record| {
                restore_instruction_slot(&record.instruction)
                    .is_some_and(|slot| !schema_ids.contains(slot))
            })
            || !matches!(
                program.instructions.last(),
                Some(ResumeManifestRestoreInstructionRecord {
                    instruction: ResumeManifestRestoreInstruction::MarkBoundaryRestored,
                    ..
                })
            )
        {
            diagnostics.push(diagnostic(
                "PSRSM1213",
                "restore program contains an unresolved endpoint or lacks completion",
            ));
        }
    }
    for chunk in &manifest.chunks {
        if chunk
            .required_boundary_ids
            .iter()
            .any(|boundary| !boundary_ids.contains(boundary))
            || chunk
                .dependency_chunk_ids
                .iter()
                .any(|dependency| !chunk_ids.contains(dependency))
        {
            diagnostics.push(diagnostic(
                "PSRSM1214",
                "chunk record contains an unresolved endpoint",
            ));
        }
    }
    for activation in &manifest.activations {
        if !boundary_ids.contains(&activation.boundary_id)
            || !chunk_ids.contains(&activation.chunk_id)
            || activation
                .event_id
                .as_ref()
                .is_some_and(|event| !event_ids.contains(event))
            || activation
                .prerequisite_boundary_ids
                .iter()
                .any(|boundary| !boundary_ids.contains(boundary))
        {
            diagnostics.push(diagnostic(
                "PSRSM1215",
                "activation record contains an unresolved endpoint",
            ));
        }
    }
    for anchor in &manifest.anchors {
        if !boundary_ids.contains(&anchor.boundary_id) {
            diagnostics.push(diagnostic(
                "PSRSM1216",
                "anchor record contains an unresolved boundary",
            ));
        }
    }
    for event in &manifest.events {
        if !anchor_ids.contains(&event.exact_target_anchor_id)
            || !boundary_ids.contains(&event.owner_boundary_id)
            || !chunk_ids.contains(&event.chunk_id)
        {
            diagnostics.push(diagnostic(
                "PSRSM1217",
                "event record contains an unresolved endpoint",
            ));
        }
    }
    if manifest.build_id == ResumeBuildId::zero_sentinel() {
        diagnostics.push(diagnostic(
            "PSRSM1218",
            "resume build ID retained the zero sentinel",
        ));
    }
    diagnostics
}

fn unique_ids<'a, T: Ord + Clone + 'a>(
    values: impl Iterator<Item = &'a T>,
    code: &str,
    message: &str,
    diagnostics: &mut Vec<ResumeManifestValidationDiagnostic>,
) -> BTreeSet<T> {
    let mut set = BTreeSet::new();
    for value in values {
        if !set.insert(value.clone()) {
            diagnostics.push(diagnostic(code, message));
        }
    }
    set
}

fn phase_i_component_records(
    plan: &crate::ResumePlan,
) -> Vec<ResumeManifestPhaseIComponentResumeRecord> {
    plan.components
        .iter()
        .cloned()
        .map(|component| ResumeManifestPhaseIComponentResumeRecord::Component { component })
        .chain(plan.effects.records.iter().map(|record| {
            ResumeManifestPhaseIComponentResumeRecord::Effect {
                effect: manifest_effect_record(record),
            }
        }))
        .chain(plan.contexts.records.iter().map(|record| {
            ResumeManifestPhaseIComponentResumeRecord::ContextSlot {
                context_slot: manifest_context_record(record),
            }
        }))
        .chain(
            plan.component_instances
                .iter()
                .cloned()
                .map(|component_instance| {
                    ResumeManifestPhaseIComponentResumeRecord::ComponentInstance {
                        component_instance,
                    }
                }),
        )
        .chain(
            plan.structural_regions
                .iter()
                .cloned()
                .map(|structural_region| {
                    ResumeManifestPhaseIComponentResumeRecord::StructuralRegion {
                        structural_region,
                    }
                }),
        )
        .chain(plan.slot_bindings.iter().cloned().map(|slot_binding| {
            ResumeManifestPhaseIComponentResumeRecord::SlotBinding { slot_binding }
        }))
        .collect()
}

fn manifest_effect_record(record: &crate::EffectResumeRecord) -> ResumeManifestEffectRecord {
    ResumeManifestEffectRecord {
        effect_id: record.effect.to_string(),
        activation_slot_id: record.activation_slot.clone(),
        initial_status: record.initial_status,
        execution_policy: execution_policy(record.execution_policy),
        runtime_function_id: record.runtime_function.to_string(),
        initial_plan: record.initial_plan_membership.as_ref().map(|membership| {
            ResumeManifestInitialEffectPlan {
                render_boundary: render_boundary(membership.render_boundary),
                batch_index: membership.batch_index,
            }
        }),
        action_batch_ids: record
            .action_batches
            .iter()
            .map(ToString::to_string)
            .collect(),
        execution_boundary: execution_boundary(record.boundary),
    }
}

fn manifest_context_record(record: &crate::ContextResumeRecord) -> ResumeManifestContextSlotRecord {
    ResumeManifestContextSlotRecord {
        source: match &record.source {
            crate::ContextValueSourceId::Provider(provider) => provider.as_str().to_string(),
            crate::ContextValueSourceId::ContextDefault(context) => {
                format!("{}/default", context.as_str())
            }
        },
        context_id: record.context.as_str().to_string(),
        runtime_slot_id: record.runtime_slot.as_str().to_string(),
        resume_slot_id: record.resume_slot.clone(),
        semantic_type: record.semantic_type.to_string(),
        source_kind: match record.source_kind {
            crate::RuntimeContextSourceKind::Provider => ResumeManifestContextSourceKind::Provider,
            crate::RuntimeContextSourceKind::ContextDefault => {
                ResumeManifestContextSourceKind::ContextDefault
            }
        },
        initial_status: record.initial_status,
        action_batch_ids: record
            .action_batches
            .iter()
            .map(ToString::to_string)
            .collect(),
        consumer_ids: record.consumers.iter().map(ToString::to_string).collect(),
        execution_boundary: execution_boundary(record.boundary),
    }
}

fn manifest_boundary_kind(kind: ResumeBoundaryKind) -> ResumeManifestBoundaryKind {
    match kind {
        ResumeBoundaryKind::ApplicationRoot => ResumeManifestBoundaryKind::ApplicationRoot,
        ResumeBoundaryKind::ComponentInstance => ResumeManifestBoundaryKind::ComponentInstance,
        ResumeBoundaryKind::StructuralRegion => ResumeManifestBoundaryKind::StructuralRegion,
        ResumeBoundaryKind::FormInstance => ResumeManifestBoundaryKind::FormInstance,
        ResumeBoundaryKind::Interaction => ResumeManifestBoundaryKind::Interaction,
    }
}

fn boundary_owner_text(owner: &ResumeBoundaryOwner) -> String {
    match owner {
        ResumeBoundaryOwner::ApplicationRoot(root) => root.to_string(),
        ResumeBoundaryOwner::ComponentInstance(instance) => instance.to_string(),
        ResumeBoundaryOwner::StructuralRegion { region, .. } => region.to_string(),
        ResumeBoundaryOwner::FormInstance(instance) => instance.to_string(),
        ResumeBoundaryOwner::Interaction(identity) => format!("{identity:?}"),
    }
}

fn manifest_activation_policy(policy: ResumeActivationPolicy) -> ResumeManifestActivationPolicy {
    match policy {
        ResumeActivationPolicy::Eager => ResumeManifestActivationPolicy::Eager,
        ResumeActivationPolicy::Visible => ResumeManifestActivationPolicy::Visible,
        ResumeActivationPolicy::Interaction => ResumeManifestActivationPolicy::Interaction,
        ResumeActivationPolicy::Manual => ResumeManifestActivationPolicy::Manual,
        ResumeActivationPolicy::None => ResumeManifestActivationPolicy::None,
    }
}

fn manifest_provenance(provenance: &SourceProvenance) -> ResumeManifestProvenance {
    ResumeManifestProvenance {
        path: provenance.path.display().to_string(),
        start: provenance.span.start,
        end: provenance.span.end,
        line: provenance.span.line,
        column: provenance.span.column,
    }
}

fn manifest_retention_reason(reason: ResumeRetentionReason) -> ResumeManifestRetentionReason {
    match reason {
        ResumeRetentionReason::MutableState => ResumeManifestRetentionReason::MutableState,
        ResumeRetentionReason::NonRecomputableComputedCache
        | ResumeRetentionReason::ComputedDirtyState
        | ResumeRetentionReason::PureEagerComputedCache
        | ResumeRetentionReason::ComputedDirtyAfterRecompute
        | ResumeRetentionReason::EffectSchedulerMetadata => {
            ResumeManifestRetentionReason::RequiredComputedCache
        }
        ResumeRetentionReason::ContextConsumerDependency => {
            ResumeManifestRetentionReason::RequiredContextValue
        }
        ResumeRetentionReason::FormFieldValue => ResumeManifestRetentionReason::FormValue,
        ResumeRetentionReason::FormFieldDirty => ResumeManifestRetentionReason::FormDirty,
        ResumeRetentionReason::FormFieldTouched => ResumeManifestRetentionReason::FormTouched,
        ResumeRetentionReason::FormRuleResult | ResumeRetentionReason::FormAggregateValidation => {
            ResumeManifestRetentionReason::FormRuleResult
        }
        ResumeRetentionReason::StableFormSubmission => {
            ResumeManifestRetentionReason::FormSubmissionState
        }
    }
}

fn recompute_program_id(
    slot: &ResumeExistingSlot,
    model: &ApplicationSemanticModel,
) -> Option<String> {
    let ResumeExistingSlot::ComputedCache(cache) = slot else {
        return None;
    };
    let ir = lower_components_to_ir(model);
    crate::build_computed_instance_slot_registry(model, &ir)
        .records
        .into_iter()
        .find(|record| &record.cache_slot_id == cache)
        .map(|record| record.computed_id.to_string())
}

fn manifest_capture_instruction(
    instruction: &ResumeCaptureInstruction,
) -> ResumeManifestCaptureInstruction {
    match instruction {
        ResumeCaptureInstruction::ReadSlot { slot_id } => {
            ResumeManifestCaptureInstruction::ReadSlot {
                slot_id: slot_id.clone(),
            }
        }
        ResumeCaptureInstruction::EncodeSlot { slot_id, codec } => {
            ResumeManifestCaptureInstruction::EncodeSlot {
                slot_id: slot_id.clone(),
                codec: codec.clone(),
            }
        }
        ResumeCaptureInstruction::AppendValueRecord {
            value_record_id,
            slot_id,
        } => ResumeManifestCaptureInstruction::AppendValueRecord {
            value_record_id: value_record_id.clone(),
            slot_id: slot_id.clone(),
        },
    }
}

fn manifest_restore_instruction(
    instruction: &ResumeRestoreInstruction,
) -> ResumeManifestRestoreInstruction {
    match instruction {
        ResumeRestoreInstruction::AllocateBoundaryRuntimeRecord => {
            ResumeManifestRestoreInstruction::AllocateBoundaryRuntimeRecord
        }
        ResumeRestoreInstruction::DecodeValue { slot_id, codec } => {
            ResumeManifestRestoreInstruction::DecodeValue {
                slot_id: slot_id.clone(),
                codec: codec.clone(),
            }
        }
        ResumeRestoreInstruction::WriteSlot { slot_id } => {
            ResumeManifestRestoreInstruction::WriteSlot {
                slot_id: slot_id.clone(),
            }
        }
        ResumeRestoreInstruction::RestoreStructuralSelection { region_id, slot_id } => {
            ResumeManifestRestoreInstruction::RestoreStructuralSelection {
                region_id: region_id.to_string(),
                slot_id: slot_id.clone(),
            }
        }
        ResumeRestoreInstruction::RestoreFormSlot {
            form_instance_id,
            field_id,
            slot_id,
        } => ResumeManifestRestoreInstruction::RestoreFormSlot {
            form_instance_id: form_instance_id.to_string(),
            field_id: field_id.as_ref().map(ToString::to_string),
            slot_id: slot_id.clone(),
        },
        ResumeRestoreInstruction::RecomputeComputed {
            component_instance_id,
            computed_id,
        } => ResumeManifestRestoreInstruction::RecomputeComputed {
            component_instance_id: component_instance_id.to_string(),
            computed_id: computed_id.to_string(),
        },
        ResumeRestoreInstruction::BindContextConsumer {
            consumer_instance_id,
            selected_source,
            provider_instance_id,
            value_slot_id,
        } => ResumeManifestRestoreInstruction::BindContextConsumer {
            consumer_instance_id: consumer_instance_id.to_string(),
            selected_source: selected_source.to_string(),
            provider_instance_id: provider_instance_id.as_ref().map(ToString::to_string),
            value_slot_id: value_slot_id.to_string(),
        },
        ResumeRestoreInstruction::InstallDomBinding {
            binding_id,
            anchor_id,
        } => ResumeManifestRestoreInstruction::InstallDomBinding {
            binding_id: binding_id.to_string(),
            anchor_id: anchor_id.clone(),
        },
        ResumeRestoreInstruction::InstallEffectSubscription { effect_id } => {
            ResumeManifestRestoreInstruction::InstallEffectSubscription {
                effect_id: effect_id.to_string(),
            }
        }
        ResumeRestoreInstruction::MarkBoundaryRestored => {
            ResumeManifestRestoreInstruction::MarkBoundaryRestored
        }
    }
}

fn manifest_chunk_kind(kind: ResumeChunkRootKind) -> ResumeManifestChunkRootKind {
    match kind {
        ResumeChunkRootKind::Eager => ResumeManifestChunkRootKind::Eager,
        ResumeChunkRootKind::Interaction => ResumeManifestChunkRootKind::Interaction,
        ResumeChunkRootKind::Visible => ResumeManifestChunkRootKind::Visible,
        ResumeChunkRootKind::Manual => ResumeManifestChunkRootKind::Manual,
    }
}

fn chunk_program_id(program: &ResumeChunkProgram) -> String {
    match program {
        ResumeChunkProgram::RuntimeBootstrap => "runtime-bootstrap".to_string(),
        ResumeChunkProgram::RuntimeRegistries => "runtime-registries".to_string(),
        ResumeChunkProgram::EventDelegation => "event-delegation".to_string(),
        ResumeChunkProgram::PostRestoreRecomputation(slot) => {
            format!("post-restore-recompute:{}", slot.text())
        }
        ResumeChunkProgram::ImmediateFormRuntime(form) => {
            format!("immediate-form-runtime:{form}")
        }
        ResumeChunkProgram::OrdinaryEvent { program, .. } => program.to_string(),
        ResumeChunkProgram::FormSubmit { submit_action, .. } => submit_action.to_string(),
    }
}

fn restore_phase_label(phase: ResumeRestorePhase) -> &'static str {
    match phase {
        ResumeRestorePhase::R0ValidateAllArtifactsAndSnapshot => "R0",
        ResumeRestorePhase::R1AllocateEmptyClosedRuntimeRegistries => "R1",
        ResumeRestorePhase::R2AllocateBoundaryRuntimeRecords => "R2",
        ResumeRestorePhase::R3RestoreMutableStateAndResources => "R3",
        ResumeRestorePhase::R4RestoreRetainedComputedCaches => "R4",
        ResumeRestorePhase::R5RecomputeComputed => "R5",
        ResumeRestorePhase::R6RestoreContextProviderValues => "R6",
        ResumeRestorePhase::R7BindContextConsumers => "R7",
        ResumeRestorePhase::R8RestoreComponentRuntimeAndSlots => "R8",
        ResumeRestorePhase::R9RestoreStructuralSelection => "R9",
        ResumeRestorePhase::R10VerifyDomAnchors => "R10",
        ResumeRestorePhase::R11RestoreFormFieldValues => "R11",
        ResumeRestorePhase::R12RestoreFormDirtyAndTouched => "R12",
        ResumeRestorePhase::R13RestoreFormValidation => "R13",
        ResumeRestorePhase::R14RestoreStableFormSubmission => "R14",
        ResumeRestorePhase::R15SynchronizeFormControls => "R15",
        ResumeRestorePhase::R16InstallDomBindings => "R16",
        ResumeRestorePhase::R17InstallEffectSubscriptions => "R17",
        ResumeRestorePhase::R18InstallEventDelegationAndActivation => "R18",
        ResumeRestorePhase::R19MarkBoundariesReady => "R19",
        ResumeRestorePhase::R20PublishApplicationReady => "R20",
    }
}

fn capture_instruction_slot(instruction: &ResumeManifestCaptureInstruction) -> &ResumeSlotId {
    match instruction {
        ResumeManifestCaptureInstruction::ReadSlot { slot_id }
        | ResumeManifestCaptureInstruction::EncodeSlot { slot_id, .. }
        | ResumeManifestCaptureInstruction::AppendValueRecord { slot_id, .. } => slot_id,
    }
}

fn restore_instruction_slot(
    instruction: &ResumeManifestRestoreInstruction,
) -> Option<&ResumeSlotId> {
    match instruction {
        ResumeManifestRestoreInstruction::DecodeValue { slot_id, .. }
        | ResumeManifestRestoreInstruction::WriteSlot { slot_id }
        | ResumeManifestRestoreInstruction::RestoreStructuralSelection { slot_id, .. }
        | ResumeManifestRestoreInstruction::RestoreFormSlot { slot_id, .. } => Some(slot_id),
        ResumeManifestRestoreInstruction::AllocateBoundaryRuntimeRecord
        | ResumeManifestRestoreInstruction::RecomputeComputed { .. }
        | ResumeManifestRestoreInstruction::BindContextConsumer { .. }
        | ResumeManifestRestoreInstruction::InstallDomBinding { .. }
        | ResumeManifestRestoreInstruction::InstallEffectSubscription { .. }
        | ResumeManifestRestoreInstruction::MarkBoundaryRestored => None,
    }
}

fn execution_policy(policy: EffectExecutionPolicy) -> ResumeManifestEffectExecutionPolicy {
    match policy {
        EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch => {
            ResumeManifestEffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch
        }
    }
}

fn render_boundary(boundary: EffectRenderBoundary) -> ResumeManifestEffectRenderBoundary {
    match boundary {
        EffectRenderBoundary::AfterInitialRender => {
            ResumeManifestEffectRenderBoundary::AfterInitialRender
        }
    }
}

fn execution_boundary(boundary: ExecutionBoundary) -> ResumeManifestEffectExecutionBoundary {
    match boundary {
        ExecutionBoundary::Client => ResumeManifestEffectExecutionBoundary::Client,
        ExecutionBoundary::Server => ResumeManifestEffectExecutionBoundary::Server,
    }
}

fn diagnostic(code: &str, message: &str) -> ResumeManifestValidationDiagnostic {
    ResumeManifestValidationDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(source: &str) -> ApplicationSemanticModel {
        crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeManifest.tsx",
            source,
        ))
    }

    #[test]
    fn emits_canonical_v7_manifest_and_public_snapshot_protocol() {
        let model = model(
            r#"@component("x-counter") class Counter {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  @action() increment() { this.count += 1; }
  render() { return <button onClick={() => this.increment()}>{this.doubled}</button>; }
}"#,
        );
        let manifest = build_resume_manifest(&model);
        let json = resume_manifest_json(&manifest);
        let value: serde_json::Value = serde_json::from_str(&json).expect("manifest JSON");
        assert_eq!(value["schema_version"], 7);
        assert_eq!(value["snapshot_schema_version"], 2);
        assert_eq!(value["runtime_protocol_version"], 1);
        assert!(!manifest.boundaries.is_empty());
        assert!(!manifest.slot_schemas.is_empty());
        assert!(!manifest.capture_programs.is_empty());
        assert!(!manifest.restore_programs.is_empty());
        assert!(!manifest.chunks.is_empty());
        assert!(!manifest.activations.is_empty());
        assert!(!manifest.anchors.is_empty());
        assert!(!manifest.events.is_empty());
        assert!(validate_resume_manifest(&manifest).is_empty());
        assert_eq!(parse_resume_manifest_v7(&json), Ok(manifest));
        assert!(json.ends_with('\n'));
        assert!(!json[..json.len() - 1].contains('\n'));
    }

    #[test]
    fn rejects_v5_without_a_compatibility_adapter() {
        let error = parse_resume_manifest_v7(r#"{"schema_version":5,"components":[]}"#)
            .expect_err("v5 rejected");
        assert_eq!(error.code, "PSRSM1201");
    }

    #[test]
    fn rejects_unknown_fields_and_unresolved_v7_endpoints_at_parse_time() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let manifest = build_resume_manifest(&model);
        let mut value =
            serde_json::to_value(&manifest).expect("resume manifest should serialize to JSON");
        value
            .as_object_mut()
            .expect("manifest object")
            .insert("unknown".to_string(), serde_json::Value::Bool(true));
        assert!(parse_resume_manifest_v7(&value.to_string()).is_err());

        let mut dangling = manifest;
        dangling.boundaries[0].capture_program_id = "resume-capture:missing".parse().expect("ID");
        assert!(parse_resume_manifest_v7(&resume_manifest_json(&dangling)).is_err());
    }

    #[test]
    fn validates_every_executable_endpoint_family() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); @action() go() {} render() { return <button onClick={() => this.go()}>{this.count}</button>; } }"#,
        );
        let canonical = build_resume_manifest(&model);
        let mutations = [
            {
                let mut manifest = canonical.clone();
                manifest.boundaries[0].schema_id = "resume-schema:missing".parse().expect("ID");
                manifest
            },
            {
                let mut manifest = canonical.clone();
                manifest.slot_schemas[0].owner_boundary_id =
                    "resume-boundary:missing".parse().expect("ID");
                manifest
            },
            {
                let mut manifest = canonical.clone();
                if let Some(instruction) = manifest
                    .capture_programs
                    .iter_mut()
                    .flat_map(|program| &mut program.instructions)
                    .next()
                {
                    *instruction = ResumeManifestCaptureInstruction::ReadSlot {
                        slot_id: "resume-slot:missing".parse().expect("ID"),
                    };
                }
                manifest
            },
            {
                let mut manifest = canonical.clone();
                manifest.restore_programs[0].boundary_id =
                    "resume-boundary:missing".parse().expect("ID");
                manifest
            },
            {
                let mut manifest = canonical.clone();
                manifest.chunks[0]
                    .dependency_chunk_ids
                    .push("resume-chunk:missing".parse().expect("ID"));
                manifest
            },
            {
                let mut manifest = canonical.clone();
                manifest.activations[0].chunk_id = "resume-chunk:missing".parse().expect("ID");
                manifest
            },
        ];
        for mutation in mutations {
            assert!(!validate_resume_manifest(&mutation).is_empty());
        }
    }

    #[test]
    fn build_id_is_stable_sensitive_to_execution_and_ignores_provenance() {
        let base = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let manifest = build_resume_manifest(&base);
        assert_eq!(manifest.build_id, build_resume_manifest(&base).build_id);
        let mut provenance_changed = manifest.clone();
        provenance_changed.boundaries[0].provenance.path =
            "/different/output/source.tsx".to_string();
        provenance_changed.boundaries[0].provenance.start += 10;
        assert_eq!(
            compute_resume_build_id(&base, &manifest),
            compute_resume_build_id(&base, &provenance_changed)
        );
        let changed = model(
            r#"@component("x-counter") class Counter { count = state(2); render() { return <button>{this.count}</button>; } }"#,
        );
        assert_ne!(
            build_resume_manifest(&base).build_id,
            build_resume_manifest(&changed).build_id
        );
    }

    #[test]
    fn build_id_ignores_absolute_source_and_output_paths() {
        let source = r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#;
        let first = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::parse_sources([("/tmp/presolve-a/src/Counter.tsx", source)]),
        );
        let second = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::parse_sources([(
                "/var/tmp/presolve-b/src/Counter.tsx",
                source,
            )]),
        );
        assert_eq!(
            build_resume_manifest(&first).build_id,
            build_resume_manifest(&second).build_id
        );
    }

    #[test]
    fn repeated_and_reversed_builds_are_byte_identical() {
        let first = presolve_parser::parse_file(
            "src/A.tsx",
            r#"@component("x-a") @route("/a") class A { value = state(1); render() { return <button>{this.value}</button>; } }"#,
        );
        let second = presolve_parser::parse_file(
            "src/B.tsx",
            r#"@component("x-b") @route("/b") class B { enabled = state(true); render() { return <button>{this.enabled}</button>; } }"#,
        );
        let forward = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![first.clone(), second.clone()]),
        );
        let reverse = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![second, first]),
        );
        assert_eq!(
            resume_manifest_json(&build_resume_manifest(&forward)),
            resume_manifest_json(&build_resume_manifest(&reverse))
        );
    }

    #[test]
    fn snapshot_example_uses_the_manifest_build_identity() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let manifest = build_resume_manifest(&model);
        let snapshot = crate::ResumeSnapshotV1 {
            schema_version: crate::RESUME_SNAPSHOT_SCHEMA_VERSION,
            snapshot_id: crate::ResumeSnapshotId::for_build(&manifest.build_id),
            build_id: manifest.build_id.clone(),
            manifest_version: RESUME_MANIFEST_SCHEMA_VERSION,
            captured_at: None,
            boundaries: Vec::new(),
            structural_occurrences: Vec::new(),
        };
        let json = crate::resume_snapshot_json(&snapshot);
        assert!(json.contains(manifest.build_id.as_str()));
        assert!(json.contains("\"capturedAt\":null"));
    }
}
