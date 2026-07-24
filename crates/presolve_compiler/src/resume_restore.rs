//! J8 closed restore programs and the fixed application R0-R20 schedule.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_computed_instance_slot_registry, build_ordinary_template_instance_registry,
    build_resume_boundary_graph, build_resume_liveness_plan, build_resume_schema_registry,
    build_runtime_component_registry, build_state_instance_storage_registry,
    lower_components_to_ir, ApplicationSemanticModel, ComponentInstanceId,
    ComponentStructuralRegionId, ConsumerInstanceId, ContextSourceInstanceId, FieldId,
    FormInstanceId, InstanceContextValueSlotId, OrdinaryTemplateBindingKind, ProviderInstanceId,
    ResumeAnchorId, ResumeBoundaryId, ResumeBoundaryOwner, ResumeExistingSlot,
    ResumeLivenessClassificationRef, ResumeRestoreProgramId, ResumeSlotId, ResumeValueCodec,
    SemanticId, SourceProvenance, TemplateInstanceBindingId,
};

pub const RESUME_RESTORE_PLAN_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeRestorePhase {
    R0ValidateAllArtifactsAndSnapshot,
    R1AllocateEmptyClosedRuntimeRegistries,
    R2AllocateBoundaryRuntimeRecords,
    R3RestoreMutableStateAndResources,
    R4RestoreRetainedComputedCaches,
    R5RecomputeComputed,
    R6RestoreContextProviderValues,
    R7BindContextConsumers,
    R8RestoreComponentRuntimeAndSlots,
    R9RestoreStructuralSelection,
    R10VerifyDomAnchors,
    R11RestoreFormFieldValues,
    R12RestoreFormDirtyAndTouched,
    R13RestoreFormValidation,
    R14RestoreStableFormSubmission,
    R15SynchronizeFormControls,
    R16InstallDomBindings,
    R17InstallEffectSubscriptions,
    R18InstallEventDelegationAndActivation,
    R19MarkBoundariesReady,
    R20PublishApplicationReady,
}

impl ResumeRestorePhase {
    pub const ALL: [Self; 21] = [
        Self::R0ValidateAllArtifactsAndSnapshot,
        Self::R1AllocateEmptyClosedRuntimeRegistries,
        Self::R2AllocateBoundaryRuntimeRecords,
        Self::R3RestoreMutableStateAndResources,
        Self::R4RestoreRetainedComputedCaches,
        Self::R5RecomputeComputed,
        Self::R6RestoreContextProviderValues,
        Self::R7BindContextConsumers,
        Self::R8RestoreComponentRuntimeAndSlots,
        Self::R9RestoreStructuralSelection,
        Self::R10VerifyDomAnchors,
        Self::R11RestoreFormFieldValues,
        Self::R12RestoreFormDirtyAndTouched,
        Self::R13RestoreFormValidation,
        Self::R14RestoreStableFormSubmission,
        Self::R15SynchronizeFormControls,
        Self::R16InstallDomBindings,
        Self::R17InstallEffectSubscriptions,
        Self::R18InstallEventDelegationAndActivation,
        Self::R19MarkBoundariesReady,
        Self::R20PublishApplicationReady,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeRestoreInstruction {
    AllocateBoundaryRuntimeRecord,
    DecodeValue {
        slot_id: ResumeSlotId,
        codec: ResumeValueCodec,
    },
    WriteSlot {
        slot_id: ResumeSlotId,
    },
    RestoreStructuralSelection {
        region_id: crate::ComponentStructuralRegionId,
        slot_id: ResumeSlotId,
    },
    RestoreFormSlot {
        form_instance_id: FormInstanceId,
        field_id: Option<FieldId>,
        slot_id: ResumeSlotId,
    },
    RecomputeComputed {
        component_instance_id: ComponentInstanceId,
        computed_id: SemanticId,
    },
    BindContextConsumer {
        consumer_instance_id: ConsumerInstanceId,
        selected_source: ContextSourceInstanceId,
        provider_instance_id: Option<ProviderInstanceId>,
        value_slot_id: InstanceContextValueSlotId,
    },
    InstallDomBinding {
        binding_id: TemplateInstanceBindingId,
        anchor_id: ResumeAnchorId,
    },
    InstallEffectSubscription {
        effect_id: SemanticId,
    },
    MarkBoundaryRestored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreInstructionRecord {
    pub phase: ResumeRestorePhase,
    pub instruction: ResumeRestoreInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreSlotAssignment {
    pub slot: ResumeExistingSlot,
    pub slot_id: ResumeSlotId,
    pub phase: ResumeRestorePhase,
    pub retained: bool,
    pub recomputable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreProgram {
    pub program_id: ResumeRestoreProgramId,
    pub boundary_id: ResumeBoundaryId,
    pub slot_assignments: Vec<ResumeRestoreSlotAssignment>,
    pub instructions: Vec<ResumeRestoreInstructionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreSchedulePhase {
    pub phase: ResumeRestorePhase,
    pub programs: Vec<ResumeRestoreProgramId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreApplicationSchedule {
    pub phases: Vec<ResumeRestoreSchedulePhase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeRestoreBlockReason {
    MissingSlotSchema,
    UnsupportedRecomputableSlot,
    MissingComputedProgram,
    MissingFormSlotOwner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreBlock {
    pub boundary: Option<ResumeBoundaryId>,
    pub slot: ResumeExistingSlot,
    pub reason: ResumeRestoreBlockReason,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestorePlan {
    pub version: u32,
    pub programs: Vec<ResumeRestoreProgram>,
    pub schedule: ResumeRestoreApplicationSchedule,
    pub blocks: Vec<ResumeRestoreBlock>,
    pub program_index: BTreeMap<ResumeBoundaryId, usize>,
}

impl ResumeRestorePlan {
    #[must_use]
    pub fn program(&self, boundary: &ResumeBoundaryId) -> Option<&ResumeRestoreProgram> {
        self.program_index
            .get(boundary)
            .and_then(|index| self.programs.get(*index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeRestoreIntegrityCode {
    ProgramReference,
    PhaseOrDuplicateWrite,
    MissingCompletion,
    OrderingOrOutputDrift,
}

impl ResumeRestoreIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProgramReference => "PSASM1359",
            Self::PhaseOrDuplicateWrite => "PSASM1360",
            Self::MissingCompletion => "PSASM1361",
            Self::OrderingOrOutputDrift => "PSASM1362",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRestoreIntegrityDiagnostic {
    pub code: ResumeRestoreIntegrityCode,
    pub boundary: Option<ResumeBoundaryId>,
    pub message: String,
}

#[derive(Debug, Clone)]
struct RestoreAuthorities {
    computed: BTreeMap<ResumeExistingSlot, (ComponentInstanceId, SemanticId)>,
    form: BTreeMap<ResumeExistingSlot, (FormInstanceId, Option<FieldId>)>,
    context_bindings:
        BTreeMap<ResumeExistingSlot, Vec<crate::RuntimeComponentContextBindingRecord>>,
    structural: BTreeMap<ComponentStructuralRegionId, ResumeExistingSlot>,
}

impl RestoreAuthorities {
    fn new(model: &ApplicationSemanticModel) -> Self {
        let ir = lower_components_to_ir(model);
        let mut computed = BTreeMap::new();
        for record in build_computed_instance_slot_registry(model, &ir).records {
            let value = (
                record.component_instance_id.clone(),
                record.computed_id.clone(),
            );
            computed.insert(
                ResumeExistingSlot::ComputedCache(record.cache_slot_id),
                value.clone(),
            );
            computed.insert(
                ResumeExistingSlot::ComputedDirty(record.dirty_slot_id),
                value,
            );
        }
        let mut form = BTreeMap::new();
        for instance in model.optimized_form_ir.optimized.instances.values() {
            for (field, slot) in &instance.storage.value {
                form.insert(
                    ResumeExistingSlot::FormFieldValue(slot.clone()),
                    (instance.id.clone(), Some(field.clone())),
                );
            }
            for (field, slot) in &instance.storage.dirty {
                form.insert(
                    ResumeExistingSlot::FormFieldDirty(slot.clone()),
                    (instance.id.clone(), Some(field.clone())),
                );
            }
            for (field, slot) in &instance.storage.touched {
                form.insert(
                    ResumeExistingSlot::FormFieldTouched(slot.clone()),
                    (instance.id.clone(), Some(field.clone())),
                );
            }
            for (field, slot) in &instance.storage.validation {
                form.insert(
                    ResumeExistingSlot::FormFieldValidation(slot.clone()),
                    (instance.id.clone(), Some(field.clone())),
                );
            }
            form.insert(
                ResumeExistingSlot::FormValidationAggregate(instance.storage.aggregate.clone()),
                (instance.id.clone(), None),
            );
            form.insert(
                ResumeExistingSlot::FormSubmission(instance.storage.submission.clone()),
                (instance.id.clone(), None),
            );
        }
        let mut context_bindings =
            BTreeMap::<ResumeExistingSlot, Vec<crate::RuntimeComponentContextBindingRecord>>::new();
        for binding in build_runtime_component_registry(model, &model.component_ir_optimization)
            .instance_context_bindings
        {
            context_bindings
                .entry(ResumeExistingSlot::Context(binding.runtime_slot.clone()))
                .or_default()
                .push(binding);
        }
        for bindings in context_bindings.values_mut() {
            bindings.sort_by(|left, right| left.consumer_instance.cmp(&right.consumer_instance));
        }
        let state_slots = build_state_instance_storage_registry(model, &ir);
        let structural = build_ordinary_template_instance_registry(model)
            .bindings
            .iter()
            .filter_map(|binding| {
                let region_kind = match binding.binding_kind {
                    OrdinaryTemplateBindingKind::Conditional => "conditional",
                    OrdinaryTemplateBindingKind::List => "keyed-list",
                    _ => return None,
                };
                let [storage] = binding.state_storage_ids.as_slice() else {
                    return None;
                };
                let state_slot = state_slots.records.iter().find(|record| {
                    record.component_instance_id == binding.component_instance_id
                        && record.storage_id == *storage
                })?;
                Some((
                    ComponentStructuralRegionId::for_template_entity(
                        &binding.declaration_binding_id,
                        region_kind,
                    ),
                    ResumeExistingSlot::State(state_slot.slot_id.clone()),
                ))
            })
            .collect();
        Self {
            computed,
            form,
            context_bindings,
            structural,
        }
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_resume_restore_plan(model: &ApplicationSemanticModel) -> ResumeRestorePlan {
    let graph = build_resume_boundary_graph(model);
    let liveness = build_resume_liveness_plan(model);
    let schemas = build_resume_schema_registry(model);
    let authorities = RestoreAuthorities::new(model);
    let mut programs = Vec::new();
    let mut blocks = Vec::new();

    for boundary in &graph.boundaries {
        let mut by_phase = BTreeMap::<ResumeRestorePhase, Vec<ResumeRestoreInstruction>>::new();
        by_phase
            .entry(ResumeRestorePhase::R2AllocateBoundaryRuntimeRecords)
            .or_default()
            .push(ResumeRestoreInstruction::AllocateBoundaryRuntimeRecord);
        let mut slot_assignments = Vec::new();
        if let Some(schema) = schemas.schema(&boundary.id) {
            for slot_schema in &schema.slots {
                let Some(classification) = liveness.classification(&slot_schema.existing_slot)
                else {
                    blocks.push(ResumeRestoreBlock {
                        boundary: Some(boundary.id.clone()),
                        slot: slot_schema.existing_slot.clone(),
                        reason: ResumeRestoreBlockReason::MissingSlotSchema,
                        provenance: slot_schema.provenance.clone(),
                    });
                    continue;
                };
                match classification {
                    ResumeLivenessClassificationRef::Retained(_) => {
                        let phase = restore_phase(&slot_schema.existing_slot);
                        slot_assignments.push(ResumeRestoreSlotAssignment {
                            slot: slot_schema.existing_slot.clone(),
                            slot_id: slot_schema.resume_slot_id.clone(),
                            phase,
                            retained: true,
                            recomputable: false,
                        });
                        let instructions = by_phase.entry(phase).or_default();
                        instructions.push(ResumeRestoreInstruction::DecodeValue {
                            slot_id: slot_schema.resume_slot_id.clone(),
                            codec: slot_schema.codec.clone(),
                        });
                        if is_form_slot(&slot_schema.existing_slot) {
                            if let Some((form_instance_id, field_id)) =
                                authorities.form.get(&slot_schema.existing_slot)
                            {
                                instructions.push(ResumeRestoreInstruction::RestoreFormSlot {
                                    form_instance_id: form_instance_id.clone(),
                                    field_id: field_id.clone(),
                                    slot_id: slot_schema.resume_slot_id.clone(),
                                });
                            } else {
                                blocks.push(ResumeRestoreBlock {
                                    boundary: Some(boundary.id.clone()),
                                    slot: slot_schema.existing_slot.clone(),
                                    reason: ResumeRestoreBlockReason::MissingFormSlotOwner,
                                    provenance: slot_schema.provenance.clone(),
                                });
                            }
                        } else {
                            instructions.push(ResumeRestoreInstruction::WriteSlot {
                                slot_id: slot_schema.resume_slot_id.clone(),
                            });
                        }
                    }
                    ResumeLivenessClassificationRef::Recomputable(_) => {
                        slot_assignments.push(ResumeRestoreSlotAssignment {
                            slot: slot_schema.existing_slot.clone(),
                            slot_id: slot_schema.resume_slot_id.clone(),
                            phase: ResumeRestorePhase::R5RecomputeComputed,
                            retained: false,
                            recomputable: true,
                        });
                        if matches!(
                            slot_schema.existing_slot,
                            ResumeExistingSlot::ComputedCache(_)
                        ) {
                            if let Some((component_instance_id, computed_id)) =
                                authorities.computed.get(&slot_schema.existing_slot)
                            {
                                by_phase
                                    .entry(ResumeRestorePhase::R5RecomputeComputed)
                                    .or_default()
                                    .push(ResumeRestoreInstruction::RecomputeComputed {
                                        component_instance_id: component_instance_id.clone(),
                                        computed_id: computed_id.clone(),
                                    });
                            } else {
                                blocks.push(ResumeRestoreBlock {
                                    boundary: Some(boundary.id.clone()),
                                    slot: slot_schema.existing_slot.clone(),
                                    reason: ResumeRestoreBlockReason::MissingComputedProgram,
                                    provenance: slot_schema.provenance.clone(),
                                });
                            }
                        } else if !matches!(
                            slot_schema.existing_slot,
                            ResumeExistingSlot::ComputedDirty(_)
                        ) {
                            blocks.push(ResumeRestoreBlock {
                                boundary: Some(boundary.id.clone()),
                                slot: slot_schema.existing_slot.clone(),
                                reason: ResumeRestoreBlockReason::UnsupportedRecomputableSlot,
                                provenance: slot_schema.provenance.clone(),
                            });
                        }
                    }
                    ResumeLivenessClassificationRef::Excluded(_)
                    | ResumeLivenessClassificationRef::Blocked(_) => {
                        blocks.push(ResumeRestoreBlock {
                            boundary: Some(boundary.id.clone()),
                            slot: slot_schema.existing_slot.clone(),
                            reason: ResumeRestoreBlockReason::MissingSlotSchema,
                            provenance: slot_schema.provenance.clone(),
                        });
                    }
                }
            }
        }
        for slot in &boundary.owned_slots {
            if let Some(bindings) = authorities.context_bindings.get(slot) {
                for binding in bindings {
                    by_phase
                        .entry(ResumeRestorePhase::R7BindContextConsumers)
                        .or_default()
                        .push(ResumeRestoreInstruction::BindContextConsumer {
                            consumer_instance_id: binding.consumer_instance.clone(),
                            selected_source: binding.selected_source.clone(),
                            provider_instance_id: binding.provider_source.clone(),
                            value_slot_id: binding.runtime_slot.clone(),
                        });
                }
            }
        }
        if let ResumeBoundaryOwner::StructuralRegion { region, .. } = &boundary.owner {
            if let Some(slot) = authorities.structural.get(region) {
                by_phase
                    .entry(ResumeRestorePhase::R9RestoreStructuralSelection)
                    .or_default()
                    .push(ResumeRestoreInstruction::RestoreStructuralSelection {
                        region_id: region.clone(),
                        slot_id: slot.resume_slot_id(),
                    });
            }
        }
        by_phase
            .entry(ResumeRestorePhase::R19MarkBoundariesReady)
            .or_default()
            .push(ResumeRestoreInstruction::MarkBoundaryRestored);
        slot_assignments.sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
        let instructions = ResumeRestorePhase::ALL
            .iter()
            .flat_map(|phase| {
                by_phase
                    .remove(phase)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|instruction| ResumeRestoreInstructionRecord {
                        phase: *phase,
                        instruction,
                    })
            })
            .collect();
        programs.push(ResumeRestoreProgram {
            program_id: ResumeRestoreProgramId::for_boundary(&boundary.id),
            boundary_id: boundary.id.clone(),
            slot_assignments,
            instructions,
        });
    }

    blocks.sort_by(|left, right| {
        (&left.boundary, &left.slot, left.reason).cmp(&(&right.boundary, &right.slot, right.reason))
    });
    let schedule = ResumeRestoreApplicationSchedule {
        phases: ResumeRestorePhase::ALL
            .iter()
            .map(|phase| ResumeRestoreSchedulePhase {
                phase: *phase,
                programs: programs
                    .iter()
                    .filter(|program| {
                        program
                            .instructions
                            .iter()
                            .any(|instruction| instruction.phase == *phase)
                    })
                    .map(|program| program.program_id.clone())
                    .collect(),
            })
            .collect(),
    };
    let program_index = programs
        .iter()
        .enumerate()
        .map(|(index, program)| (program.boundary_id.clone(), index))
        .collect();
    ResumeRestorePlan {
        version: RESUME_RESTORE_PLAN_VERSION,
        programs,
        schedule,
        blocks,
        program_index,
    }
}

fn restore_phase(slot: &ResumeExistingSlot) -> ResumeRestorePhase {
    match slot {
        ResumeExistingSlot::State(_) => ResumeRestorePhase::R3RestoreMutableStateAndResources,
        ResumeExistingSlot::ComputedCache(_) | ResumeExistingSlot::ComputedDirty(_) => {
            ResumeRestorePhase::R4RestoreRetainedComputedCaches
        }
        ResumeExistingSlot::Context(_) => ResumeRestorePhase::R6RestoreContextProviderValues,
        ResumeExistingSlot::FormFieldValue(_) => ResumeRestorePhase::R11RestoreFormFieldValues,
        ResumeExistingSlot::FormFieldDirty(_) | ResumeExistingSlot::FormFieldTouched(_) => {
            ResumeRestorePhase::R12RestoreFormDirtyAndTouched
        }
        ResumeExistingSlot::FormFieldValidation(_)
        | ResumeExistingSlot::FormValidationAggregate(_) => {
            ResumeRestorePhase::R13RestoreFormValidation
        }
        ResumeExistingSlot::FormSubmission(_) => ResumeRestorePhase::R14RestoreStableFormSubmission,
        ResumeExistingSlot::EffectActivation(_) => {
            ResumeRestorePhase::R17InstallEffectSubscriptions
        }
    }
}

fn is_form_slot(slot: &ResumeExistingSlot) -> bool {
    matches!(
        slot,
        ResumeExistingSlot::FormFieldValue(_)
            | ResumeExistingSlot::FormFieldDirty(_)
            | ResumeExistingSlot::FormFieldTouched(_)
            | ResumeExistingSlot::FormFieldValidation(_)
            | ResumeExistingSlot::FormValidationAggregate(_)
            | ResumeExistingSlot::FormSubmission(_)
    )
}

/// # Errors
///
/// Returns J8 integrity diagnostics for malformed program references, wrong
/// phases, duplicate writes, missing completion, parent-order violations, or
/// deterministic construction drift.
#[allow(clippy::too_many_lines)]
pub fn validate_resume_restore_plan(
    model: &ApplicationSemanticModel,
    plan: &ResumeRestorePlan,
) -> Result<(), Vec<ResumeRestoreIntegrityDiagnostic>> {
    let mut diagnostics = Vec::new();
    let graph = build_resume_boundary_graph(model);
    let expected_boundaries = graph
        .boundaries
        .iter()
        .map(|boundary| boundary.id.clone())
        .collect::<Vec<_>>();
    let actual_boundaries = plan
        .programs
        .iter()
        .map(|program| program.boundary_id.clone())
        .collect::<Vec<_>>();
    if expected_boundaries != actual_boundaries
        || plan.schedule.phases.len() != ResumeRestorePhase::ALL.len()
        || plan
            .schedule
            .phases
            .iter()
            .map(|phase| phase.phase)
            .collect::<Vec<_>>()
            != ResumeRestorePhase::ALL
    {
        diagnostics.push(restore_diagnostic(
            ResumeRestoreIntegrityCode::ProgramReference,
            None,
            "restore programs or fixed phase schedule do not correspond",
        ));
    }
    let known_programs = plan
        .programs
        .iter()
        .map(|program| program.program_id.clone())
        .collect::<BTreeSet<_>>();
    if plan
        .schedule
        .phases
        .iter()
        .flat_map(|phase| &phase.programs)
        .any(|program| !known_programs.contains(program))
    {
        diagnostics.push(restore_diagnostic(
            ResumeRestoreIntegrityCode::ProgramReference,
            None,
            "restore schedule references an unknown program",
        ));
    }
    for program in &plan.programs {
        let mut writes = BTreeSet::new();
        let mut previous_phase = None;
        for record in &program.instructions {
            if previous_phase.is_some_and(|phase| phase > record.phase) {
                diagnostics.push(restore_diagnostic(
                    ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite,
                    Some(program.boundary_id.clone()),
                    "restore instruction phases are out of fixed order",
                ));
            }
            previous_phase = Some(record.phase);
            if !instruction_phase_is_valid(program, record) {
                diagnostics.push(restore_diagnostic(
                    ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite,
                    Some(program.boundary_id.clone()),
                    "restore instruction is assigned to the wrong fixed phase",
                ));
            }
            if let ResumeRestoreInstruction::WriteSlot { slot_id }
            | ResumeRestoreInstruction::RestoreFormSlot { slot_id, .. } = &record.instruction
            {
                if !writes.insert(slot_id.clone()) {
                    diagnostics.push(restore_diagnostic(
                        ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite,
                        Some(program.boundary_id.clone()),
                        "restore program writes one slot more than once",
                    ));
                }
            }
        }
        if !matches!(
            program.instructions.last(),
            Some(ResumeRestoreInstructionRecord {
                phase: ResumeRestorePhase::R19MarkBoundariesReady,
                instruction: ResumeRestoreInstruction::MarkBoundaryRestored,
            })
        ) {
            diagnostics.push(restore_diagnostic(
                ResumeRestoreIntegrityCode::MissingCompletion,
                Some(program.boundary_id.clone()),
                "restore program lacks its final MarkBoundaryRestored",
            ));
        }
    }
    if plan.version != RESUME_RESTORE_PLAN_VERSION || plan != &build_resume_restore_plan(model) {
        diagnostics.push(restore_diagnostic(
            ResumeRestoreIntegrityCode::OrderingOrOutputDrift,
            None,
            "restore plan drifted from canonical J2/J3/J6 construction",
        ));
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn instruction_phase_is_valid(
    program: &ResumeRestoreProgram,
    record: &ResumeRestoreInstructionRecord,
) -> bool {
    match &record.instruction {
        ResumeRestoreInstruction::AllocateBoundaryRuntimeRecord => {
            record.phase == ResumeRestorePhase::R2AllocateBoundaryRuntimeRecords
        }
        ResumeRestoreInstruction::DecodeValue { slot_id, .. }
        | ResumeRestoreInstruction::WriteSlot { slot_id }
        | ResumeRestoreInstruction::RestoreFormSlot { slot_id, .. } => program
            .slot_assignments
            .iter()
            .find(|assignment| &assignment.slot_id == slot_id)
            .is_some_and(|assignment| assignment.retained && assignment.phase == record.phase),
        ResumeRestoreInstruction::RestoreStructuralSelection { .. } => {
            record.phase == ResumeRestorePhase::R9RestoreStructuralSelection
        }
        ResumeRestoreInstruction::RecomputeComputed { .. } => {
            record.phase == ResumeRestorePhase::R5RecomputeComputed
        }
        ResumeRestoreInstruction::BindContextConsumer { .. } => {
            record.phase == ResumeRestorePhase::R7BindContextConsumers
        }
        ResumeRestoreInstruction::InstallDomBinding { .. } => {
            record.phase == ResumeRestorePhase::R16InstallDomBindings
        }
        ResumeRestoreInstruction::InstallEffectSubscription { .. } => {
            record.phase == ResumeRestorePhase::R17InstallEffectSubscriptions
        }
        ResumeRestoreInstruction::MarkBoundaryRestored => {
            record.phase == ResumeRestorePhase::R19MarkBoundariesReady
        }
    }
}

fn restore_diagnostic(
    code: ResumeRestoreIntegrityCode,
    boundary: Option<ResumeBoundaryId>,
    message: &str,
) -> ResumeRestoreIntegrityDiagnostic {
    ResumeRestoreIntegrityDiagnostic {
        code,
        boundary,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(source: &str) -> ApplicationSemanticModel {
        crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeRestore.tsx",
            source,
        ))
    }

    #[test]
    fn builds_all_fixed_phases_and_assigns_retained_and_recomputable_slots() {
        let model = model(
            r#"@component("x-counter") class Counter {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  render() { return <button>{this.doubled}</button>; }
}"#,
        );
        let plan = build_resume_restore_plan(&model);
        let liveness = build_resume_liveness_plan(&model);
        assert_eq!(
            plan.schedule
                .phases
                .iter()
                .map(|phase| phase.phase)
                .collect::<Vec<_>>(),
            ResumeRestorePhase::ALL
        );
        let assignments = plan
            .programs
            .iter()
            .flat_map(|program| &program.slot_assignments)
            .map(|assignment| assignment.slot.clone())
            .collect::<BTreeSet<_>>();
        assert!(liveness
            .retained
            .iter()
            .all(|record| assignments.contains(&record.slot.existing_slot)));
        assert!(liveness
            .recomputable
            .iter()
            .all(|record| assignments.contains(&record.slot.existing_slot)));
        assert!(plan.programs.iter().all(|program| matches!(
            program.instructions.last(),
            Some(ResumeRestoreInstructionRecord {
                phase: ResumeRestorePhase::R19MarkBoundariesReady,
                instruction: ResumeRestoreInstruction::MarkBoundaryRestored,
            })
        )));
        assert_eq!(validate_resume_restore_plan(&model, &plan), Ok(()));
    }

    #[test]
    fn restores_form_slot_classes_in_their_exact_fixed_phases() {
        let model = model(
            r#"@component("x-profile") class Profile {
  @form() profile!: Form;
  @field(this.profile) name = "";
  render() { return <input field={this.name} />; }
}"#,
        );
        let plan = build_resume_restore_plan(&model);
        let assignments = plan
            .programs
            .iter()
            .flat_map(|program| &program.slot_assignments)
            .map(|assignment| (&assignment.slot, assignment.phase))
            .collect::<BTreeMap<_, _>>();
        assert!(assignments.iter().any(|(slot, phase)| matches!(
            (slot, phase),
            (
                ResumeExistingSlot::FormFieldValue(_),
                ResumeRestorePhase::R11RestoreFormFieldValues
            )
        )));
        assert!(assignments.iter().any(|(slot, phase)| matches!(
            (slot, phase),
            (
                ResumeExistingSlot::FormFieldDirty(_) | ResumeExistingSlot::FormFieldTouched(_),
                ResumeRestorePhase::R12RestoreFormDirtyAndTouched
            )
        )));
        assert!(assignments.iter().any(|(slot, phase)| matches!(
            (slot, phase),
            (
                ResumeExistingSlot::FormFieldValidation(_)
                    | ResumeExistingSlot::FormValidationAggregate(_),
                ResumeRestorePhase::R13RestoreFormValidation
            )
        )));
        assert!(assignments.iter().any(|(slot, phase)| matches!(
            (slot, phase),
            (
                ResumeExistingSlot::FormSubmission(_),
                ResumeRestorePhase::R14RestoreStableFormSubmission
            )
        )));
    }

    #[test]
    fn structural_regions_restore_from_their_exact_retained_state_slots() {
        let model = model(
            r#"@component("x-leaf") class Leaf extends Component { render() { return <span />; } }
@component("x-page") @route("/") class Page extends Component {
  visible = state(true);
  items = state([{ id: "a" }, { id: "b" }]);
  render() {
    return <main>{this.visible ? <div><Leaf /></div> : <aside>Hidden</aside>}<ul>{this.items.map(item => <li key={item.id}><Leaf /></li>)}</ul></main>;
  }
  hide() { this.visible = false; }
  trim() { this.items = [{ id: "b" }]; }
}"#,
        );
        let plan = build_resume_restore_plan(&model);
        let structural = plan
            .programs
            .iter()
            .flat_map(|program| &program.instructions)
            .filter_map(|record| match &record.instruction {
                ResumeRestoreInstruction::RestoreStructuralSelection { region_id, slot_id } => {
                    Some((record.phase, region_id.to_string(), slot_id.to_string()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(structural.len(), 2);
        assert!(structural
            .iter()
            .all(|(phase, _, _)| { *phase == ResumeRestorePhase::R9RestoreStructuralSelection }));
        assert!(structural
            .iter()
            .any(|(_, region, slot)| region.ends_with(":conditional")
                && slot.contains("state%3Avisible")));
        assert!(structural
            .iter()
            .any(|(_, region, slot)| region.ends_with(":keyed-list")
                && slot.contains("state%3Aitems")));
        assert_eq!(validate_resume_restore_plan(&model, &plan), Ok(()));
    }

    #[test]
    fn malformed_reference_phase_write_completion_and_parent_order_are_rejected() {
        let model = model(
            r#"@component("x-child") class Child { value = state(1); render() { return <span>{this.value}</span>; } }
@component("x-parent") @route("/") class Parent { render() { return <Child />; } }"#,
        );
        let canonical = build_resume_restore_plan(&model);

        let mut dangling = canonical.clone();
        dangling.programs.pop();
        let diagnostics =
            validate_resume_restore_plan(&model, &dangling).expect_err("dangling reference");
        assert!(diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == ResumeRestoreIntegrityCode::ProgramReference }));

        let mut missing = canonical.clone();
        missing.programs[0].instructions.pop();
        let diagnostics = validate_resume_restore_plan(&model, &missing).expect_err("missing mark");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeRestoreIntegrityCode::MissingCompletion
        }));

        let mut wrong_phase = canonical.clone();
        wrong_phase.programs[0].instructions[0].phase = ResumeRestorePhase::R19MarkBoundariesReady;
        let diagnostics =
            validate_resume_restore_plan(&model, &wrong_phase).expect_err("wrong phase");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite
                || diagnostic.code == ResumeRestoreIntegrityCode::OrderingOrOutputDrift
        }));

        let mut duplicate = canonical.clone();
        let duplicate_write = duplicate
            .programs
            .iter()
            .enumerate()
            .find_map(|(index, program)| {
                program
                    .instructions
                    .iter()
                    .find(|record| {
                        matches!(
                            record.instruction,
                            ResumeRestoreInstruction::WriteSlot { .. }
                        )
                    })
                    .cloned()
                    .map(|record| (index, record))
            });
        if let Some((program_index, write)) = duplicate_write {
            duplicate.programs[program_index]
                .instructions
                .insert(1, write);
        }
        let diagnostics =
            validate_resume_restore_plan(&model, &duplicate).expect_err("duplicate write");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite
                || diagnostic.code == ResumeRestoreIntegrityCode::OrderingOrOutputDrift
        }));

        let mut reversed = canonical;
        reversed.programs.reverse();
        let diagnostics =
            validate_resume_restore_plan(&model, &reversed).expect_err("parent order");
        assert!(diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == ResumeRestoreIntegrityCode::ProgramReference }));
    }

    #[test]
    fn construction_is_deterministic_under_source_reversal() {
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
            build_resume_restore_plan(&forward),
            build_resume_restore_plan(&reverse)
        );
    }

    #[test]
    fn reserves_the_complete_j8_integrity_range() {
        assert_eq!(
            [
                ResumeRestoreIntegrityCode::ProgramReference,
                ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite,
                ResumeRestoreIntegrityCode::MissingCompletion,
                ResumeRestoreIntegrityCode::OrderingOrOutputDrift,
            ]
            .map(ResumeRestoreIntegrityCode::code),
            ["PSASM1359", "PSASM1360", "PSASM1361", "PSASM1362"]
        );
    }
}
