//! K15 immutable optimization and static runtime-cost reports.

use serde::{Deserialize, Serialize};

use crate::{
    OptimizationPolicyId, ProductionChunkGraph, ProductionChunkKind, ProductionRuntimeArtifactV1,
    ResumeBuildId,
};

pub const OPTIMIZATION_REPORT_SCHEMA_VERSION: u32 = 1;
pub const RUNTIME_COST_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionReportInputs {
    pub dead_products_removed: u32,
    pub constants_pooled: u32,
    pub programs_deduplicated: u32,
    pub shared_candidates_rejected: u32,
    pub binding_writes_coalesced: u32,
    pub development_bytes: u64,
    pub production_bytes: u64,
    pub cold_init_operation_count: u32,
    pub resume_restore_operation_count: u32,
    pub max_action_batch_operation_count: u32,
    pub max_scheduler_batch_width: u32,
    pub max_dom_patch_count_per_action: u32,
    pub retained_slot_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OptimizationReportV1 {
    pub schema_version: u32,
    pub build_id: ResumeBuildId,
    pub optimization_policy: OptimizationPolicyId,
    pub dead_products_removed: u32,
    pub constants_pooled: u32,
    pub programs_deduplicated: u32,
    pub shared_chunks_extracted: u32,
    pub shared_candidates_rejected: u32,
    pub binding_writes_coalesced: u32,
    pub runtime_table_count: u32,
    pub development_bytes: u64,
    pub production_bytes: u64,
    pub retained_exclusions: Vec<String>,
    pub validation_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeCostReportV1 {
    pub schema_version: u32,
    pub build_id: ResumeBuildId,
    pub bootstrap_module_bytes: u64,
    pub production_artifact_bytes: u64,
    pub eager_program_count: u32,
    pub lazy_root_chunk_count: u32,
    pub shared_chunk_count: u32,
    pub max_lazy_dependency_depth: u32,
    pub runtime_table_count: u32,
    pub runtime_record_count: u32,
    pub estimated_boot_decode_units: u32,
    pub estimated_boot_validation_units: u32,
    pub estimated_cold_init_operation_count: u32,
    pub estimated_resume_restore_operation_count: u32,
    pub max_action_batch_operation_count: u32,
    pub max_scheduler_batch_width: u32,
    pub max_dom_patch_count_per_action: u32,
    pub retained_slot_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationInspectionQuery {
    pub report: OptimizationReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCostInspectionQuery {
    pub report: RuntimeCostReportV1,
}

#[must_use]
pub fn build_production_reports(
    artifact: &ProductionRuntimeArtifactV1,
    graph: &ProductionChunkGraph,
    inputs: &ProductionReportInputs,
) -> (OptimizationReportV1, RuntimeCostReportV1) {
    let table_count = count_u32(artifact.tables.tables.len());
    let table_records = count_u32(
        artifact
            .tables
            .tables
            .iter()
            .map(|table| table.mappings.len())
            .sum::<usize>(),
    );
    let shared_count = count_u32(
        graph
            .chunks
            .iter()
            .filter(|chunk| chunk.kind == ProductionChunkKind::Shared)
            .count(),
    );
    let root_count = count_u32(graph.activation_plans.len());
    let eager_program_count = graph
        .chunks
        .iter()
        .find(|chunk| chunk.kind == ProductionChunkKind::Eager)
        .map_or(0, |chunk| count_u32(chunk.programs.len()));
    (
        OptimizationReportV1 {
            schema_version: OPTIMIZATION_REPORT_SCHEMA_VERSION,
            build_id: artifact.build_id.clone(),
            optimization_policy: artifact.optimization_policy.clone(),
            dead_products_removed: inputs.dead_products_removed,
            constants_pooled: inputs.constants_pooled,
            programs_deduplicated: inputs.programs_deduplicated,
            shared_chunks_extracted: shared_count,
            shared_candidates_rejected: inputs.shared_candidates_rejected,
            binding_writes_coalesced: inputs.binding_writes_coalesced,
            runtime_table_count: table_count,
            development_bytes: inputs.development_bytes,
            production_bytes: inputs.production_bytes,
            retained_exclusions: vec![
                "cryptographic-signing".to_string(),
                "wall-clock-timing".to_string(),
            ],
            validation_status: "valid".to_string(),
        },
        RuntimeCostReportV1 {
            schema_version: RUNTIME_COST_REPORT_SCHEMA_VERSION,
            build_id: artifact.build_id.clone(),
            bootstrap_module_bytes: graph
                .chunks
                .iter()
                .find(|chunk| chunk.kind == ProductionChunkKind::Eager)
                .map_or(0, |chunk| {
                    count_u64(chunk.provisional_module_filename.len())
                }),
            production_artifact_bytes: inputs.production_bytes,
            eager_program_count,
            lazy_root_chunk_count: root_count,
            shared_chunk_count: shared_count,
            max_lazy_dependency_depth: u32::from(shared_count > 0),
            runtime_table_count: table_count,
            runtime_record_count: table_records + count_u32(graph.chunks.len()),
            estimated_boot_decode_units: table_records,
            estimated_boot_validation_units: table_records + count_u32(graph.dependencies.len()),
            estimated_cold_init_operation_count: inputs.cold_init_operation_count,
            estimated_resume_restore_operation_count: inputs.resume_restore_operation_count,
            max_action_batch_operation_count: inputs.max_action_batch_operation_count,
            max_scheduler_batch_width: inputs.max_scheduler_batch_width,
            max_dom_patch_count_per_action: inputs.max_dom_patch_count_per_action,
            retained_slot_count: inputs.retained_slot_count,
        },
    )
}

#[must_use]
///
/// # Panics
///
/// Panics only if an in-memory compiler report cannot serialize.
pub fn optimization_report_json(report: &OptimizationReportV1) -> String {
    serde_json::to_string(report).expect("optimization report should serialize") + "\n"
}

#[must_use]
///
/// # Panics
///
/// Panics only if an in-memory compiler report cannot serialize.
pub fn runtime_cost_report_json(report: &RuntimeCostReportV1) -> String {
    serde_json::to_string(report).expect("runtime cost report should serialize") + "\n"
}

fn count_u32(value: usize) -> u32 {
    u32::try_from(value).expect("production report count exceeds u32")
}

fn count_u64(value: usize) -> u64 {
    u64::try_from(value).expect("production report byte count exceeds u64")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_production_runtime_artifact, extract_production_chunk_graph,
        ExecutableProgramFingerprint, ProductionRootChunkInput, ResumeBoundaryId, ResumeManifest,
        SharedChunkCandidatePlan,
    };
    use std::str::FromStr;

    #[test]
    fn k15_reports_recompute_exact_static_products_without_timing() {
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
        let inputs = ProductionReportInputs {
            dead_products_removed: 1,
            constants_pooled: 2,
            programs_deduplicated: 3,
            shared_candidates_rejected: 4,
            binding_writes_coalesced: 5,
            development_bytes: 100,
            production_bytes: 80,
            cold_init_operation_count: 6,
            resume_restore_operation_count: 7,
            max_action_batch_operation_count: 8,
            max_scheduler_batch_width: 9,
            max_dom_patch_count_per_action: 10,
            retained_slot_count: 11,
        };
        let (optimization, cost) = build_production_reports(&artifact, &graph, &inputs);
        assert_eq!(optimization.production_bytes, 80);
        assert_eq!(
            cost.runtime_table_count,
            count_u32(artifact.tables.tables.len())
        );
        let bytes = optimization_report_json(&optimization) + &runtime_cost_report_json(&cost);
        assert!(!bytes.contains("millisecond"));
        assert!(!bytes.contains("timestamp"));
    }
}
