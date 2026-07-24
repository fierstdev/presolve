//! J7 generated exact-slot capture programs and snapshot model v1.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_resume_liveness_plan, build_resume_schema_registry, ApplicationSemanticModel,
    ResumeBoundaryId, ResumeBuildId, ResumeCaptureProgramId, ResumeExistingSlot, ResumeSchemaId,
    ResumeSchemaRegistry, ResumeSlotId, ResumeSnapshotId, ResumeValueCodec, ResumeValueRecordId,
    SerializableValue, SourceProvenance,
};

pub const RESUME_CAPTURE_PLAN_VERSION: u32 = 1;
pub const RESUME_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const RESUME_CAPTURE_MANIFEST_VERSION: u32 = 6;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeCaptureInstruction {
    ReadSlot {
        slot_id: ResumeSlotId,
    },
    EncodeSlot {
        slot_id: ResumeSlotId,
        codec: ResumeValueCodec,
    },
    AppendValueRecord {
        value_record_id: ResumeValueRecordId,
        slot_id: ResumeSlotId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeCaptureProgram {
    pub program_id: ResumeCaptureProgramId,
    pub boundary_id: ResumeBoundaryId,
    pub instructions: Vec<ResumeCaptureInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeEnvelopeWriterPlan {
    pub snapshot_schema_version: u32,
    pub manifest_version: u32,
    pub captured_at_is_null: bool,
    pub boundary_programs: Vec<ResumeCaptureProgramId>,
    pub standalone_trailing_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeCaptureBlockReason {
    MissingBoundarySchema,
    MissingRetainedSlotSchema,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeCaptureBlock {
    pub boundary: Option<ResumeBoundaryId>,
    pub slot: Option<ResumeExistingSlot>,
    pub reason: ResumeCaptureBlockReason,
    pub provenance: Option<SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeCapturePlan {
    pub version: u32,
    pub programs: Vec<ResumeCaptureProgram>,
    pub envelope_writer: ResumeEnvelopeWriterPlan,
    pub blocks: Vec<ResumeCaptureBlock>,
    pub program_index: BTreeMap<ResumeBoundaryId, usize>,
}

impl ResumeCapturePlan {
    #[must_use]
    pub fn program(&self, boundary: &ResumeBoundaryId) -> Option<&ResumeCaptureProgram> {
        self.program_index
            .get(boundary)
            .and_then(|index| self.programs.get(*index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct RuntimeQuiescenceState {
    pub action_batch_depth: usize,
    pub scheduler_pending_program_count: usize,
    pub dom_patch_queue_empty: bool,
    pub structural_program_active: bool,
    pub form_submission_active: bool,
    pub validation_execution_active: bool,
    pub effect_execution_active: bool,
    pub chunk_activation_active: bool,
}

impl RuntimeQuiescenceState {
    #[must_use]
    pub const fn quiescent() -> Self {
        Self {
            action_batch_depth: 0,
            scheduler_pending_program_count: 0,
            dom_patch_queue_empty: true,
            structural_program_active: false,
            form_submission_active: false,
            validation_execution_active: false,
            effect_execution_active: false,
            chunk_activation_active: false,
        }
    }

    #[must_use]
    pub const fn is_quiescent(self) -> bool {
        self.action_batch_depth == 0
            && self.scheduler_pending_program_count == 0
            && self.dom_patch_queue_empty
            && !self.structural_program_active
            && !self.form_submission_active
            && !self.validation_execution_active
            && !self.effect_execution_active
            && !self.chunk_activation_active
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeEncodedValue(String);

impl ResumeEncodedValue {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSnapshotValueRecordV1 {
    pub value_record_id: ResumeValueRecordId,
    pub slot_id: ResumeSlotId,
    pub value: ResumeEncodedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSnapshotBoundaryV1 {
    pub boundary_id: ResumeBoundaryId,
    pub schema_id: ResumeSchemaId,
    pub values: Vec<ResumeSnapshotValueRecordV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSnapshotV1 {
    pub schema_version: u32,
    pub build_id: ResumeBuildId,
    pub snapshot_id: ResumeSnapshotId,
    pub manifest_version: u32,
    pub captured_at: Option<()>,
    pub boundaries: Vec<ResumeSnapshotBoundaryV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeCaptureErrorKind {
    NotQuiescent,
    MissingExactSlot,
    InvalidRuntimeShape,
    InvalidNumber,
    InvalidFormStableState,
    ProgramIntegrity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeCaptureError {
    pub kind: ResumeCaptureErrorKind,
    pub boundary: Option<ResumeBoundaryId>,
    pub slot: Option<ResumeSlotId>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeCaptureIntegrityCode {
    ProgramCorrespondence,
    InvalidInstruction,
    InvalidCaptureState,
    OrderingOrOutputDrift,
}

impl ResumeCaptureIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProgramCorrespondence => "PSASM1355",
            Self::InvalidInstruction => "PSASM1356",
            Self::InvalidCaptureState => "PSASM1357",
            Self::OrderingOrOutputDrift => "PSASM1358",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeCaptureIntegrityDiagnostic {
    pub code: ResumeCaptureIntegrityCode,
    pub boundary: Option<ResumeBoundaryId>,
    pub message: String,
}

#[must_use]
pub fn build_resume_capture_plan(model: &ApplicationSemanticModel) -> ResumeCapturePlan {
    let schemas = build_resume_schema_registry(model);
    let liveness = build_resume_liveness_plan(model);
    let retained = liveness
        .retained
        .iter()
        .map(|record| record.slot.existing_slot.clone())
        .collect::<BTreeSet<_>>();
    let mut captured = BTreeSet::new();
    let mut programs = Vec::new();
    let mut blocks = Vec::new();

    for schema in &schemas.schemas {
        let mut instructions = Vec::new();
        for slot in &schema.slots {
            if !retained.contains(&slot.existing_slot) {
                continue;
            }
            captured.insert(slot.existing_slot.clone());
            instructions.extend([
                ResumeCaptureInstruction::ReadSlot {
                    slot_id: slot.resume_slot_id.clone(),
                },
                ResumeCaptureInstruction::EncodeSlot {
                    slot_id: slot.resume_slot_id.clone(),
                    codec: slot.codec.clone(),
                },
                ResumeCaptureInstruction::AppendValueRecord {
                    value_record_id: ResumeValueRecordId::for_slot(&slot.resume_slot_id),
                    slot_id: slot.resume_slot_id.clone(),
                },
            ]);
        }
        programs.push(ResumeCaptureProgram {
            program_id: ResumeCaptureProgramId::for_boundary(&schema.boundary),
            boundary_id: schema.boundary.clone(),
            instructions,
        });
    }

    for retained_slot in &liveness.retained {
        if !captured.contains(&retained_slot.slot.existing_slot) {
            blocks.push(ResumeCaptureBlock {
                boundary: retained_slot.slot.boundary_candidate.clone(),
                slot: Some(retained_slot.slot.existing_slot.clone()),
                reason: ResumeCaptureBlockReason::MissingRetainedSlotSchema,
                provenance: Some(retained_slot.slot.provenance.clone()),
            });
        }
    }
    for block in &schemas.blocks {
        if retained.contains(&block.slot) {
            blocks.push(ResumeCaptureBlock {
                boundary: block.boundary.clone(),
                slot: Some(block.slot.clone()),
                reason: ResumeCaptureBlockReason::MissingRetainedSlotSchema,
                provenance: Some(block.provenance.clone()),
            });
        }
    }
    blocks.sort_by(|left, right| {
        (&left.boundary, &left.slot, left.reason).cmp(&(&right.boundary, &right.slot, right.reason))
    });
    blocks.dedup_by(|left, right| {
        left.boundary == right.boundary && left.slot == right.slot && left.reason == right.reason
    });
    let program_index = programs
        .iter()
        .enumerate()
        .map(|(index, program)| (program.boundary_id.clone(), index))
        .collect();
    let envelope_writer = ResumeEnvelopeWriterPlan {
        snapshot_schema_version: RESUME_SNAPSHOT_SCHEMA_VERSION,
        manifest_version: RESUME_CAPTURE_MANIFEST_VERSION,
        captured_at_is_null: true,
        boundary_programs: programs
            .iter()
            .map(|program| program.program_id.clone())
            .collect(),
        standalone_trailing_newline: true,
    };

    ResumeCapturePlan {
        version: RESUME_CAPTURE_PLAN_VERSION,
        programs,
        envelope_writer,
        blocks,
        program_index,
    }
}

/// Executes the generated J7 program over exact runtime slot values.
///
/// # Errors
///
/// Returns an error when capture is not quiescent, a required exact slot is
/// missing, a runtime value does not match its compiler-generated codec, a
/// number is not finite/canonical, a Form submission state is pending, or the
/// supplied program/schema products are not reciprocal.
#[allow(clippy::too_many_lines)]
pub fn capture_resume_snapshot(
    plan: &ResumeCapturePlan,
    schemas: &ResumeSchemaRegistry,
    quiescence: RuntimeQuiescenceState,
    build_id: ResumeBuildId,
    runtime_values: &BTreeMap<ResumeSlotId, SerializableValue>,
) -> Result<ResumeSnapshotV1, ResumeCaptureError> {
    if !quiescence.is_quiescent() {
        return Err(capture_error(
            ResumeCaptureErrorKind::NotQuiescent,
            None,
            None,
            "capture requires an immediate compiler-defined quiescent point",
        ));
    }
    if plan.version != RESUME_CAPTURE_PLAN_VERSION
        || plan
            .blocks
            .iter()
            .any(|block| block.reason == ResumeCaptureBlockReason::MissingRetainedSlotSchema)
        || !capture_products_reciprocal(plan, schemas)
    {
        return Err(capture_error(
            ResumeCaptureErrorKind::ProgramIntegrity,
            None,
            None,
            "capture plan contains unresolved schema correspondence",
        ));
    }
    let slot_schemas = schemas
        .schemas
        .iter()
        .flat_map(|schema| {
            schema
                .slots
                .iter()
                .map(|slot| (slot.resume_slot_id.clone(), slot))
        })
        .collect::<BTreeMap<_, _>>();
    let mut boundaries = Vec::new();

    for program in &plan.programs {
        let schema = schemas.schema(&program.boundary_id).ok_or_else(|| {
            capture_error(
                ResumeCaptureErrorKind::ProgramIntegrity,
                Some(program.boundary_id.clone()),
                None,
                "capture program lacks its exact boundary schema",
            )
        })?;
        let mut values = Vec::new();
        for instruction_group in program.instructions.chunks_exact(3) {
            let (
                ResumeCaptureInstruction::ReadSlot { slot_id: read_slot },
                ResumeCaptureInstruction::EncodeSlot {
                    slot_id: encode_slot,
                    codec,
                },
                ResumeCaptureInstruction::AppendValueRecord {
                    value_record_id,
                    slot_id: append_slot,
                },
            ) = (
                &instruction_group[0],
                &instruction_group[1],
                &instruction_group[2],
            )
            else {
                return Err(capture_error(
                    ResumeCaptureErrorKind::ProgramIntegrity,
                    Some(program.boundary_id.clone()),
                    None,
                    "capture instructions are not closed read/encode/append triples",
                ));
            };
            if read_slot != encode_slot
                || read_slot != append_slot
                || value_record_id != &ResumeValueRecordId::for_slot(read_slot)
            {
                return Err(capture_error(
                    ResumeCaptureErrorKind::ProgramIntegrity,
                    Some(program.boundary_id.clone()),
                    Some(read_slot.clone()),
                    "capture instruction identities are not reciprocal",
                ));
            }
            let slot_schema = slot_schemas.get(read_slot).ok_or_else(|| {
                capture_error(
                    ResumeCaptureErrorKind::ProgramIntegrity,
                    Some(program.boundary_id.clone()),
                    Some(read_slot.clone()),
                    "capture slot lacks its generated schema",
                )
            })?;
            if &slot_schema.codec != codec {
                return Err(capture_error(
                    ResumeCaptureErrorKind::ProgramIntegrity,
                    Some(program.boundary_id.clone()),
                    Some(read_slot.clone()),
                    "capture codec drifted from the generated slot schema",
                ));
            }
            let value = runtime_values.get(read_slot).ok_or_else(|| {
                capture_error(
                    ResumeCaptureErrorKind::MissingExactSlot,
                    Some(program.boundary_id.clone()),
                    Some(read_slot.clone()),
                    "capture could not read the exact required runtime slot",
                )
            })?;
            validate_form_stable_state(slot_schema, value, &program.boundary_id)?;
            let encoded = encode_resume_value(value, codec).map_err(|mut error| {
                error.boundary = Some(program.boundary_id.clone());
                error.slot = Some(read_slot.clone());
                error
            })?;
            values.push(ResumeSnapshotValueRecordV1 {
                value_record_id: value_record_id.clone(),
                slot_id: read_slot.clone(),
                value: ResumeEncodedValue(encoded),
            });
        }
        if !program.instructions.chunks_exact(3).remainder().is_empty() {
            return Err(capture_error(
                ResumeCaptureErrorKind::ProgramIntegrity,
                Some(program.boundary_id.clone()),
                None,
                "capture program has an incomplete instruction group",
            ));
        }
        boundaries.push(ResumeSnapshotBoundaryV1 {
            boundary_id: program.boundary_id.clone(),
            schema_id: schema.id.clone(),
            values,
        });
    }

    Ok(ResumeSnapshotV1 {
        schema_version: RESUME_SNAPSHOT_SCHEMA_VERSION,
        snapshot_id: ResumeSnapshotId::for_build(&build_id),
        build_id,
        manifest_version: RESUME_CAPTURE_MANIFEST_VERSION,
        captured_at: None,
        boundaries,
    })
}

fn capture_products_reciprocal(plan: &ResumeCapturePlan, schemas: &ResumeSchemaRegistry) -> bool {
    let program_boundaries = plan
        .programs
        .iter()
        .map(|program| &program.boundary_id)
        .collect::<Vec<_>>();
    let schema_boundaries = schemas
        .schemas
        .iter()
        .map(|schema| &schema.boundary)
        .collect::<Vec<_>>();
    let program_ids = plan
        .programs
        .iter()
        .map(|program| program.program_id.clone())
        .collect::<Vec<_>>();
    program_boundaries == schema_boundaries
        && plan.envelope_writer.snapshot_schema_version == RESUME_SNAPSHOT_SCHEMA_VERSION
        && plan.envelope_writer.manifest_version == RESUME_CAPTURE_MANIFEST_VERSION
        && plan.envelope_writer.captured_at_is_null
        && plan.envelope_writer.standalone_trailing_newline
        && plan.envelope_writer.boundary_programs == program_ids
        && plan.program_index
            == plan
                .programs
                .iter()
                .enumerate()
                .map(|(index, program)| (program.boundary_id.clone(), index))
                .collect()
}

fn validate_form_stable_state(
    slot_schema: &crate::ResumeSlotSchema,
    value: &SerializableValue,
    boundary: &ResumeBoundaryId,
) -> Result<(), ResumeCaptureError> {
    if !matches!(
        slot_schema.existing_slot,
        ResumeExistingSlot::FormSubmission(_)
    ) {
        return Ok(());
    }
    let SerializableValue::String(state) = value else {
        return Err(capture_error(
            ResumeCaptureErrorKind::InvalidRuntimeShape,
            Some(boundary.clone()),
            Some(slot_schema.resume_slot_id.clone()),
            "Form submission state must be a compiler-owned string value",
        ));
    };
    if matches!(state.as_str(), "Idle" | "Invalid" | "Failed" | "Completed") {
        Ok(())
    } else {
        Err(capture_error(
            ResumeCaptureErrorKind::InvalidFormStableState,
            Some(boundary.clone()),
            Some(slot_schema.resume_slot_id.clone()),
            "pending or unknown Form submission state cannot enter a snapshot",
        ))
    }
}

/// Encodes one already-selected runtime value through its generated closed
/// codec without reflection or property enumeration as semantic authority.
///
/// # Errors
///
/// Returns an error for codec/value mismatch, unsupported object shape,
/// non-finite numbers, or negative zero.
pub fn encode_resume_value(
    value: &SerializableValue,
    codec: &ResumeValueCodec,
) -> Result<String, ResumeCaptureError> {
    match (value, codec) {
        (
            SerializableValue::Null,
            ResumeValueCodec::NullCodec | ResumeValueCodec::NullableCodec(_),
        ) => Ok("null".to_string()),
        (SerializableValue::Boolean(value), ResumeValueCodec::BooleanCodec) => {
            Ok(value.to_string())
        }
        (SerializableValue::Number(value), ResumeValueCodec::NumberCodec) => {
            canonical_number(value)
        }
        (SerializableValue::String(value), ResumeValueCodec::StringCodec) => {
            serde_json::to_string(value).map_err(|error| {
                capture_error(
                    ResumeCaptureErrorKind::InvalidRuntimeShape,
                    None,
                    None,
                    &format!("string could not be canonically escaped: {error}"),
                )
            })
        }
        (SerializableValue::Array(values), ResumeValueCodec::ArrayCodec(element_codec)) => values
            .iter()
            .map(|value| encode_resume_value(value, element_codec))
            .collect::<Result<Vec<_>, _>>()
            .map(|values| format!("[{}]", values.join(","))),
        (SerializableValue::Object(values), ResumeValueCodec::ObjectCodec(properties)) => {
            if values.len() != properties.len()
                || properties
                    .iter()
                    .any(|property| !values.contains_key(&property.name))
            {
                return Err(capture_error(
                    ResumeCaptureErrorKind::InvalidRuntimeShape,
                    None,
                    None,
                    "runtime object does not exactly match the generated property schema",
                ));
            }
            properties
                .iter()
                .map(|property| {
                    let Some(value) = values.get(&property.name) else {
                        return Err(capture_error(
                            ResumeCaptureErrorKind::InvalidRuntimeShape,
                            None,
                            None,
                            "runtime object lost a generated schema property",
                        ));
                    };
                    let name = serde_json::to_string(&property.name).map_err(|error| {
                        capture_error(
                            ResumeCaptureErrorKind::InvalidRuntimeShape,
                            None,
                            None,
                            &format!("object property could not be escaped: {error}"),
                        )
                    })?;
                    Ok(format!(
                        "{name}:{}",
                        encode_resume_value(value, &property.codec)?
                    ))
                })
                .collect::<Result<Vec<_>, ResumeCaptureError>>()
                .map(|properties| format!("{{{}}}", properties.join(",")))
        }
        (value, ResumeValueCodec::NullableCodec(inner)) => encode_resume_value(value, inner),
        _ => Err(capture_error(
            ResumeCaptureErrorKind::InvalidRuntimeShape,
            None,
            None,
            "runtime value shape does not match the generated resume codec",
        )),
    }
}

fn canonical_number(value: &str) -> Result<String, ResumeCaptureError> {
    let number = value.parse::<f64>().map_err(|_| {
        capture_error(
            ResumeCaptureErrorKind::InvalidNumber,
            None,
            None,
            "snapshot number is not a finite JSON number",
        )
    })?;
    if !number.is_finite() || (number == 0.0 && value.trim_start().starts_with('-')) {
        return Err(capture_error(
            ResumeCaptureErrorKind::InvalidNumber,
            None,
            None,
            "snapshot numbers reject non-finite values and negative zero",
        ));
    }
    Ok(number.to_string())
}

#[must_use]
pub fn resume_snapshot_json(snapshot: &ResumeSnapshotV1) -> String {
    let boundaries = snapshot
        .boundaries
        .iter()
        .map(|boundary| {
            let values = boundary
                .values
                .iter()
                .map(|record| {
                    format!(
                        "{{\"valueRecordId\":{},\"slotId\":{},\"value\":{}}}",
                        json_string(&record.value_record_id.to_string()),
                        json_string(&record.slot_id.to_string()),
                        record.value.as_str()
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"boundaryId\":{},\"schemaId\":{},\"values\":[{values}]}}",
                json_string(&boundary.boundary_id.to_string()),
                json_string(&boundary.schema_id.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"schemaVersion\":{},\"buildId\":{},\"snapshotId\":{},\"manifestVersion\":{},\"capturedAt\":null,\"boundaries\":[{boundaries}]}}",
        snapshot.schema_version,
        json_string(&snapshot.build_id.to_string()),
        json_string(&snapshot.snapshot_id.to_string()),
        snapshot.manifest_version,
    )
}

#[must_use]
pub fn resume_snapshot_artifact_json(snapshot: &ResumeSnapshotV1) -> String {
    resume_snapshot_json(snapshot) + "\n"
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("resume identity string serializes")
}

/// # Errors
///
/// Returns J7 integrity diagnostics when programs do not correspond exactly to
/// J2/J6, use malformed instruction triples, or drift from canonical order.
pub fn validate_resume_capture_plan(
    model: &ApplicationSemanticModel,
    plan: &ResumeCapturePlan,
) -> Result<(), Vec<ResumeCaptureIntegrityDiagnostic>> {
    let mut diagnostics = Vec::new();
    let schemas = build_resume_schema_registry(model);
    let expected_boundaries = schemas
        .schemas
        .iter()
        .map(|schema| schema.boundary.clone())
        .collect::<Vec<_>>();
    let actual_boundaries = plan
        .programs
        .iter()
        .map(|program| program.boundary_id.clone())
        .collect::<Vec<_>>();
    if expected_boundaries != actual_boundaries {
        diagnostics.push(integrity_diagnostic(
            ResumeCaptureIntegrityCode::ProgramCorrespondence,
            None,
            "capture programs do not cover schemas in parent-before-child order",
        ));
    }
    for program in &plan.programs {
        if program.program_id != ResumeCaptureProgramId::for_boundary(&program.boundary_id)
            || !program.instructions.chunks_exact(3).remainder().is_empty()
        {
            diagnostics.push(integrity_diagnostic(
                ResumeCaptureIntegrityCode::InvalidInstruction,
                Some(program.boundary_id.clone()),
                "capture program identity or instruction grouping is invalid",
            ));
            continue;
        }
        for group in program.instructions.chunks_exact(3) {
            let valid = matches!(
                (&group[0], &group[1], &group[2]),
                (
                    ResumeCaptureInstruction::ReadSlot { slot_id: read },
                    ResumeCaptureInstruction::EncodeSlot { slot_id: encode, .. },
                    ResumeCaptureInstruction::AppendValueRecord {
                        value_record_id,
                        slot_id: append
                    }
                ) if read == encode
                    && read == append
                    && value_record_id == &ResumeValueRecordId::for_slot(read)
            );
            if !valid {
                diagnostics.push(integrity_diagnostic(
                    ResumeCaptureIntegrityCode::InvalidInstruction,
                    Some(program.boundary_id.clone()),
                    "capture instructions are not exact read/encode/append triples",
                ));
            }
        }
    }
    if plan.envelope_writer.snapshot_schema_version != RESUME_SNAPSHOT_SCHEMA_VERSION
        || plan.envelope_writer.manifest_version != RESUME_CAPTURE_MANIFEST_VERSION
        || !plan.envelope_writer.captured_at_is_null
        || !plan.envelope_writer.standalone_trailing_newline
    {
        diagnostics.push(integrity_diagnostic(
            ResumeCaptureIntegrityCode::InvalidCaptureState,
            None,
            "capture envelope policy violates snapshot v1 determinism",
        ));
    }
    if plan != &build_resume_capture_plan(model) {
        diagnostics.push(integrity_diagnostic(
            ResumeCaptureIntegrityCode::OrderingOrOutputDrift,
            None,
            "capture plan drifted from canonical J2/J6 construction",
        ));
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn capture_error(
    kind: ResumeCaptureErrorKind,
    boundary: Option<ResumeBoundaryId>,
    slot: Option<ResumeSlotId>,
    message: &str,
) -> ResumeCaptureError {
    ResumeCaptureError {
        kind,
        boundary,
        slot,
        message: message.to_string(),
    }
}

fn integrity_diagnostic(
    code: ResumeCaptureIntegrityCode,
    boundary: Option<ResumeBoundaryId>,
    message: &str,
) -> ResumeCaptureIntegrityDiagnostic {
    ResumeCaptureIntegrityDiagnostic {
        code,
        boundary,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ObjectType, ResumeObjectPropertyCodec, SemanticType};

    fn model(source: &str) -> ApplicationSemanticModel {
        crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeCapture.tsx",
            source,
        ))
    }

    fn value_for_codec(codec: &ResumeValueCodec) -> SerializableValue {
        match codec {
            ResumeValueCodec::NullCodec | ResumeValueCodec::NullableCodec(_) => {
                SerializableValue::Null
            }
            ResumeValueCodec::BooleanCodec => SerializableValue::Boolean(false),
            ResumeValueCodec::NumberCodec => SerializableValue::Number("1".to_string()),
            ResumeValueCodec::StringCodec => SerializableValue::String(String::new()),
            ResumeValueCodec::ArrayCodec(_) => SerializableValue::Array(Vec::new()),
            ResumeValueCodec::ObjectCodec(properties) => SerializableValue::Object(
                properties
                    .iter()
                    .map(|property| (property.name.clone(), value_for_codec(&property.codec)))
                    .collect(),
            ),
        }
    }

    fn capture_values(
        plan: &ResumeCapturePlan,
        schemas: &ResumeSchemaRegistry,
    ) -> BTreeMap<ResumeSlotId, SerializableValue> {
        let captured = plan
            .programs
            .iter()
            .flat_map(|program| &program.instructions)
            .filter_map(|instruction| match instruction {
                ResumeCaptureInstruction::ReadSlot { slot_id } => Some(slot_id),
                ResumeCaptureInstruction::EncodeSlot { .. }
                | ResumeCaptureInstruction::AppendValueRecord { .. } => None,
            })
            .collect::<BTreeSet<_>>();
        schemas
            .schemas
            .iter()
            .flat_map(|schema| &schema.slots)
            .filter(|slot| captured.contains(&slot.resume_slot_id))
            .map(|slot| (slot.resume_slot_id.clone(), value_for_codec(&slot.codec)))
            .collect()
    }

    #[test]
    fn encodes_every_closed_value_variant_in_canonical_order() {
        let object_codec = ResumeValueCodec::ObjectCodec(vec![
            ResumeObjectPropertyCodec {
                name: "a".to_string(),
                codec: ResumeValueCodec::NumberCodec,
            },
            ResumeObjectPropertyCodec {
                name: "b".to_string(),
                codec: ResumeValueCodec::StringCodec,
            },
        ]);
        let fixtures = [
            (SerializableValue::Null, ResumeValueCodec::NullCodec, "null"),
            (
                SerializableValue::Boolean(true),
                ResumeValueCodec::BooleanCodec,
                "true",
            ),
            (
                SerializableValue::Number("1.50".to_string()),
                ResumeValueCodec::NumberCodec,
                "1.5",
            ),
            (
                SerializableValue::String("a\nb".to_string()),
                ResumeValueCodec::StringCodec,
                "\"a\\nb\"",
            ),
            (
                SerializableValue::Array(vec![
                    SerializableValue::Number("1".to_string()),
                    SerializableValue::Number("2".to_string()),
                ]),
                ResumeValueCodec::ArrayCodec(Box::new(ResumeValueCodec::NumberCodec)),
                "[1,2]",
            ),
            (
                SerializableValue::Object(BTreeMap::from([
                    (
                        "b".to_string(),
                        SerializableValue::String("two".to_string()),
                    ),
                    ("a".to_string(), SerializableValue::Number("1".to_string())),
                ])),
                object_codec,
                "{\"a\":1,\"b\":\"two\"}",
            ),
            (
                SerializableValue::Null,
                ResumeValueCodec::NullableCodec(Box::new(ResumeValueCodec::StringCodec)),
                "null",
            ),
        ];
        for (value, codec, expected) in fixtures {
            assert_eq!(encode_resume_value(&value, &codec).as_deref(), Ok(expected));
            let parsed: serde_json::Value =
                serde_json::from_str(expected).expect("canonical fixture JSON");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(
                    &encode_resume_value(&value, &codec).expect("encode")
                )
                .expect("encoded JSON"),
                parsed
            );
        }
    }

    #[test]
    fn rejects_negative_zero_non_finite_and_runtime_shape_guessing() {
        for number in ["-0", "-0.0", "NaN", "inf", "-inf", "1e9999"] {
            assert_eq!(
                encode_resume_value(
                    &SerializableValue::Number(number.to_string()),
                    &ResumeValueCodec::NumberCodec,
                )
                .expect_err("invalid number")
                .kind,
                ResumeCaptureErrorKind::InvalidNumber
            );
        }
        assert_eq!(
            encode_resume_value(
                &SerializableValue::Object(BTreeMap::new()),
                &ResumeValueCodec::NumberCodec,
            )
            .expect_err("shape mismatch")
            .kind,
            ResumeCaptureErrorKind::InvalidRuntimeShape
        );
    }

    #[test]
    fn generates_one_exact_program_per_boundary_and_omits_recomputable_slots() {
        let model = model(
            r#"@component("x-counter") class Counter {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  render() { return <button>{this.doubled}</button>; }
}"#,
        );
        let schemas = build_resume_schema_registry(&model);
        let liveness = build_resume_liveness_plan(&model);
        let plan = build_resume_capture_plan(&model);
        assert_eq!(plan.programs.len(), schemas.schemas.len());
        let captured = plan
            .programs
            .iter()
            .flat_map(|program| &program.instructions)
            .filter_map(|instruction| match instruction {
                ResumeCaptureInstruction::ReadSlot { slot_id } => Some(slot_id.clone()),
                ResumeCaptureInstruction::EncodeSlot { .. }
                | ResumeCaptureInstruction::AppendValueRecord { .. } => None,
            })
            .collect::<BTreeSet<_>>();
        assert!(liveness
            .retained
            .iter()
            .all(|record| captured.contains(&record.slot.resume_slot_id)));
        assert!(liveness
            .recomputable
            .iter()
            .all(|record| !captured.contains(&record.slot.resume_slot_id)));
        assert_eq!(validate_resume_capture_plan(&model, &plan), Ok(()));
    }

    #[test]
    fn equal_quiescent_state_produces_equal_bytes_without_mutation_or_timestamp() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let schemas = build_resume_schema_registry(&model);
        let plan = build_resume_capture_plan(&model);
        let values = capture_values(&plan, &schemas);
        let before = values.clone();
        let build = ResumeBuildId::for_public_inputs("capture-test");
        let first = capture_resume_snapshot(
            &plan,
            &schemas,
            RuntimeQuiescenceState::quiescent(),
            build.clone(),
            &values,
        )
        .expect("capture");
        let second = capture_resume_snapshot(
            &plan,
            &schemas,
            RuntimeQuiescenceState::quiescent(),
            build,
            &values,
        )
        .expect("capture");
        assert_eq!(values, before);
        assert_eq!(resume_snapshot_json(&first), resume_snapshot_json(&second));
        assert!(resume_snapshot_json(&first).contains("\"capturedAt\":null"));
        assert!(!resume_snapshot_json(&first).ends_with('\n'));
        assert!(resume_snapshot_artifact_json(&first).ends_with('\n'));
    }

    #[test]
    fn rejects_malformed_program_products_before_reading_slots() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let schemas = build_resume_schema_registry(&model);
        let mut plan = build_resume_capture_plan(&model);
        plan.programs.pop();
        let diagnostics = validate_resume_capture_plan(&model, &plan).expect_err("malformed plan");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeCaptureIntegrityCode::ProgramCorrespondence
        }));
        assert_eq!(
            capture_resume_snapshot(
                &plan,
                &schemas,
                RuntimeQuiescenceState::quiescent(),
                ResumeBuildId::for_public_inputs("malformed"),
                &BTreeMap::new(),
            )
            .expect_err("program integrity")
            .kind,
            ResumeCaptureErrorKind::ProgramIntegrity
        );
    }

    #[test]
    fn rejects_non_quiescent_and_pending_form_capture() {
        let model = model(
            r#"@component("x-profile") class Profile {
  @form() profile!: Form;
  @field(this.profile) name = "";
  render() { return <input field={this.name} />; }
}"#,
        );
        let schemas = build_resume_schema_registry(&model);
        let plan = build_resume_capture_plan(&model);
        let mut values = capture_values(&plan, &schemas);
        let submission = schemas
            .schemas
            .iter()
            .flat_map(|schema| &schema.slots)
            .find(|slot| matches!(slot.existing_slot, ResumeExistingSlot::FormSubmission(_)))
            .expect("submission slot");
        values.insert(
            submission.resume_slot_id.clone(),
            SerializableValue::String("Submitting".to_string()),
        );
        assert_eq!(
            capture_resume_snapshot(
                &plan,
                &schemas,
                RuntimeQuiescenceState::quiescent(),
                ResumeBuildId::for_public_inputs("pending-form"),
                &values,
            )
            .expect_err("pending submission")
            .kind,
            ResumeCaptureErrorKind::InvalidFormStableState
        );
        let mut active = RuntimeQuiescenceState::quiescent();
        active.validation_execution_active = true;
        assert_eq!(
            capture_resume_snapshot(
                &plan,
                &schemas,
                active,
                ResumeBuildId::for_public_inputs("active-validation"),
                &values,
            )
            .expect_err("not quiescent")
            .kind,
            ResumeCaptureErrorKind::NotQuiescent
        );
    }

    #[test]
    fn reserves_the_complete_j7_integrity_range() {
        assert_eq!(
            [
                ResumeCaptureIntegrityCode::ProgramCorrespondence,
                ResumeCaptureIntegrityCode::InvalidInstruction,
                ResumeCaptureIntegrityCode::InvalidCaptureState,
                ResumeCaptureIntegrityCode::OrderingOrOutputDrift,
            ]
            .map(ResumeCaptureIntegrityCode::code),
            ["PSASM1355", "PSASM1356", "PSASM1357", "PSASM1358"]
        );
    }

    #[test]
    fn capture_plan_is_deterministic_under_source_reversal() {
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
            build_resume_capture_plan(&forward),
            build_resume_capture_plan(&reverse)
        );
    }

    #[test]
    fn codec_fixture_type_remains_semantically_closed() {
        let semantic_type = SemanticType::Object(ObjectType {
            properties: BTreeMap::from([("value".to_string(), SemanticType::Number)]),
        });
        assert!(crate::resume_value_codec(&semantic_type).is_ok());
    }
}
