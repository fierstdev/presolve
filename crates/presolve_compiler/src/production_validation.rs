//! K14 ordered V0-V10 production validation and compact failure records.

use serde::{Deserialize, Serialize};

use crate::{
    parse_production_runtime_artifact_v1, ProductionArtifactIntegrityViolation,
    ProductionRuntimeArtifactV1, ResumeBuildId,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ProductionValidationPhase {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProductionValidationEvidence {
    pub endpoints_valid: bool,
    pub fingerprints_and_aliases_valid: bool,
    pub chunk_exports_valid: bool,
    pub boot_entry_closed: bool,
    pub resume_products_agree: bool,
    pub anchor_event_tables_agree: bool,
    pub cleanup_closure_valid: bool,
}

impl ProductionValidationEvidence {
    #[must_use]
    pub const fn all_valid() -> Self {
        Self {
            endpoints_valid: true,
            fingerprints_and_aliases_valid: true,
            chunk_exports_valid: true,
            boot_entry_closed: true,
            resume_products_agree: true,
            anchor_event_tables_agree: true,
            cleanup_closure_valid: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRuntimeFailure {
    pub class: String,
    pub code: String,
    pub build_id: ResumeBuildId,
    pub subject_kind: String,
    pub subject_id_or_ordinal: String,
    pub subject_trusted: bool,
    pub phase: ProductionValidationPhase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionValidationResult {
    pub artifact: ProductionRuntimeArtifactV1,
    pub completed_phases: Vec<ProductionValidationPhase>,
}

/// Runs V0-V10 in exact order before any authored execution is permitted.
///
/// # Errors
///
/// Returns one stable compact failure at the first invalid phase.
pub fn validate_production_runtime_pipeline(
    json: &str,
    expected_build_id: &ResumeBuildId,
    evidence: &ProductionValidationEvidence,
) -> Result<ProductionValidationResult, Box<ProductionRuntimeFailure>> {
    if contains_forbidden_generic_key(json) {
        return Err(Box::new(failure(
            expected_build_id,
            ProductionValidationPhase::V0,
            "PSPRD1400",
            "schema",
            "untrusted-field",
            false,
        )));
    }
    let artifact =
        parse_production_runtime_artifact_v1(json, expected_build_id).map_err(|violations| {
            let phase = integrity_phase(&violations);
            Box::new(failure(
                expected_build_id,
                phase,
                "PSPRD1401",
                "artifact",
                "untrusted",
                false,
            ))
        })?;
    let checks = [
        (ProductionValidationPhase::V4, evidence.endpoints_valid),
        (
            ProductionValidationPhase::V5,
            evidence.fingerprints_and_aliases_valid,
        ),
        (ProductionValidationPhase::V6, evidence.chunk_exports_valid),
        (ProductionValidationPhase::V7, evidence.boot_entry_closed),
        (
            ProductionValidationPhase::V8,
            evidence.resume_products_agree,
        ),
        (
            ProductionValidationPhase::V9,
            evidence.anchor_event_tables_agree,
        ),
        (
            ProductionValidationPhase::V10,
            evidence.cleanup_closure_valid,
        ),
    ];
    for (phase, valid) in checks {
        if !valid {
            return Err(Box::new(failure(
                &artifact.build_id,
                phase,
                "PSPRD1402",
                "validation",
                "closed-product",
                true,
            )));
        }
    }
    Ok(ProductionValidationResult {
        artifact,
        completed_phases: vec![
            ProductionValidationPhase::V0,
            ProductionValidationPhase::V1,
            ProductionValidationPhase::V2,
            ProductionValidationPhase::V3,
            ProductionValidationPhase::V4,
            ProductionValidationPhase::V5,
            ProductionValidationPhase::V6,
            ProductionValidationPhase::V7,
            ProductionValidationPhase::V8,
            ProductionValidationPhase::V9,
            ProductionValidationPhase::V10,
        ],
    })
}

fn integrity_phase(
    violations: &[ProductionArtifactIntegrityViolation],
) -> ProductionValidationPhase {
    if violations.iter().any(|violation| {
        matches!(
            violation,
            ProductionArtifactIntegrityViolation::SchemaVersionMismatch
        )
    }) {
        ProductionValidationPhase::V0
    } else if violations.iter().any(|violation| {
        matches!(
            violation,
            ProductionArtifactIntegrityViolation::BuildIdMismatch
                | ProductionArtifactIntegrityViolation::RuntimeProtocolMismatch
        )
    }) {
        ProductionValidationPhase::V1
    } else if violations.iter().any(|violation| {
        matches!(
            violation,
            ProductionArtifactIntegrityViolation::TableChecksumMismatch(_)
                | ProductionArtifactIntegrityViolation::ArtifactChecksumMismatch
        )
    }) {
        ProductionValidationPhase::V2
    } else {
        ProductionValidationPhase::V3
    }
}

fn contains_forbidden_generic_key(json: &str) -> bool {
    ["\"__proto__\"", "\"constructor\"", "\"prototype\""]
        .iter()
        .any(|key| json.contains(key))
}

fn failure(
    build_id: &ResumeBuildId,
    phase: ProductionValidationPhase,
    code: &str,
    subject_kind: &str,
    subject: &str,
    trusted: bool,
) -> ProductionRuntimeFailure {
    ProductionRuntimeFailure {
        class: "ProductionArtifactMismatch".to_string(),
        code: code.to_string(),
        build_id: build_id.clone(),
        subject_kind: subject_kind.to_string(),
        subject_id_or_ordinal: subject.to_string(),
        subject_trusted: trusted,
        phase,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_production_runtime_artifact, extract_production_chunk_graph,
        production_runtime_artifact_json, ExecutableProgramFingerprint, ProductionRootChunkInput,
        ResumeBoundaryId, ResumeManifest, SharedChunkCandidatePlan,
    };
    use std::str::FromStr;

    fn valid_json() -> (String, ResumeBuildId) {
        let manifest = ResumeManifest {
            schema_version: 6,
            build_id: ResumeBuildId::zero_sentinel(),
            snapshot_schema_version: 1,
            runtime_protocol_version: 1,
            application_root_boundary_id: ResumeBoundaryId::from_str("resume-boundary:root")
                .expect("boundary"),
            boundaries: Vec::new(),
            slot_schemas: Vec::new(),
            capture_programs: Vec::new(),
            restore_programs: Vec::new(),
            chunks: Vec::new(),
            activations: Vec::new(),
            anchors: Vec::new(),
            events: Vec::new(),
            phase_i_component_resume_records: Vec::new(),
            phase_i_form_resume_records: Vec::new(),
        };
        let graph = extract_production_chunk_graph(
            &SharedChunkCandidatePlan {
                candidates: Vec::new(),
                rejections: Vec::new(),
            },
            &[ProductionRootChunkInput {
                activation_root_id: "root".to_string(),
                root_kind: "interaction".to_string(),
                programs: vec![ExecutableProgramFingerprint::for_canonical_opcode_stream(
                    b"a",
                )],
            }],
        )
        .expect("graph")
        .0;
        let artifact = build_production_runtime_artifact(&manifest, &graph).expect("artifact");
        (
            production_runtime_artifact_json(&artifact),
            manifest.build_id,
        )
    }

    #[test]
    fn k14_runs_all_phases_and_stops_before_authored_execution() {
        let (json, build_id) = valid_json();
        let result = validate_production_runtime_pipeline(
            &json,
            &build_id,
            &ProductionValidationEvidence::all_valid(),
        )
        .expect("validated pipeline");
        assert_eq!(result.completed_phases.len(), 11);
        let mut invalid = ProductionValidationEvidence::all_valid();
        invalid.cleanup_closure_valid = false;
        let failure = validate_production_runtime_pipeline(&json, &build_id, &invalid)
            .expect_err("V10 failure");
        assert_eq!(failure.phase, ProductionValidationPhase::V10);
        assert!(!serde_json::to_string(&failure)
            .expect("failure JSON")
            .contains("src/"));
    }

    #[test]
    fn k14_rejects_prototype_pollution_as_untrusted_v0_input() {
        let build_id = ResumeBuildId::zero_sentinel();
        let failure = validate_production_runtime_pipeline(
            r#"{"__proto__":{}}"#,
            &build_id,
            &ProductionValidationEvidence::all_valid(),
        )
        .expect_err("prototype key");
        assert_eq!(failure.phase, ProductionValidationPhase::V0);
        assert!(!failure.subject_trusted);
    }
}
