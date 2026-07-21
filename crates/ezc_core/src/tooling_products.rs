//! L11-F canonical, source-free tooling products approved by L11-D and L11-E.

#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    validate_production_chunk_graph, validate_production_runtime_artifact, OptimizationReportV1,
    ProductionChunkGraph, ProductionChunkKind, ProductionRuntimeArtifactV1, RuntimeCostReportV1,
};

pub const BUILD_TRACE_TOOLING_SCHEMA_V1: &str = "presolve.build-trace";
pub const COMPILE_COST_TOOLING_SCHEMA_V1: &str = "presolve.compile-cost-report";
pub const ARTIFACT_GRAPH_TOOLING_SCHEMA_V1: &str = "presolve.artifact-graph";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingTraceIdentityV1 {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolingTraceStageKindV1 {
    L3Snapshot,
    L5IncrementalPlan,
    L6Cache,
    L7Workspace,
    L8Watch,
    L4Publication,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolingTraceOutcomeV1 {
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingBuildTraceStageV1 {
    pub ordinal: u32,
    pub kind: ToolingTraceStageKindV1,
    pub outcome: ToolingTraceOutcomeV1,
    pub identities: Vec<ToolingTraceIdentityV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingBuildTraceV1 {
    pub schema: String,
    pub version: u32,
    pub trace_id: String,
    pub workspace_id: String,
    pub compiler_contract: String,
    pub snapshot_id: Option<String>,
    pub outcome: ToolingTraceOutcomeV1,
    pub stages: Vec<ToolingBuildTraceStageV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingCompileCostReportV1 {
    pub schema: String,
    pub version: u32,
    pub report_id: String,
    pub build_id: String,
    pub optimization_policy: String,
    pub optimization_report_id: String,
    pub runtime_cost_report_id: String,
    pub optimization_report: OptimizationReportV1,
    pub runtime_cost_report: RuntimeCostReportV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingArtifactGraphChunkV1 {
    pub chunk_id: String,
    pub kind: String,
    pub module_filename: String,
    pub activation_roots: Vec<String>,
    pub root_kind: Option<String>,
    pub program_fingerprints: Vec<String>,
    pub registration_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingArtifactGraphDependencyV1 {
    pub dependent_chunk_id: String,
    pub dependency_chunk_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingArtifactGraphActivationV1 {
    pub activation_root_id: String,
    pub root_chunk_id: String,
    pub shared_chunk_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingArtifactGraphV1 {
    pub schema: String,
    pub version: u32,
    pub graph_id: String,
    pub build_id: String,
    pub runtime_protocol_version: u32,
    pub optimization_policy: String,
    pub artifact_checksum: String,
    pub chunks: Vec<ToolingArtifactGraphChunkV1>,
    pub dependencies: Vec<ToolingArtifactGraphDependencyV1>,
    pub activations: Vec<ToolingArtifactGraphActivationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolingProductValidationErrorV1 {
    InvalidTraceProvenance,
    InvalidCostProvenance,
    InvalidSourceReport,
    InvalidArtifactGraphProvenance,
    ArtifactGraphTopologyDisagreement,
    Noncanonical,
}

pub fn build_tooling_build_trace_v1(
    workspace_id: String,
    compiler_contract: String,
    snapshot_id: Option<String>,
    outcome: ToolingTraceOutcomeV1,
    stages: Vec<ToolingBuildTraceStageV1>,
) -> Result<ToolingBuildTraceV1, ToolingProductValidationErrorV1> {
    let mut product = ToolingBuildTraceV1 {
        schema: BUILD_TRACE_TOOLING_SCHEMA_V1.into(),
        version: 1,
        trace_id: String::new(),
        workspace_id,
        compiler_contract,
        snapshot_id,
        outcome,
        stages,
    };
    validate_trace(&product)?;
    product.trace_id = identity_without_field(&product, "traceId");
    Ok(product)
}

pub fn build_tooling_compile_cost_report_v1(
    optimization_report: OptimizationReportV1,
    runtime_cost_report: RuntimeCostReportV1,
) -> Result<ToolingCompileCostReportV1, ToolingProductValidationErrorV1> {
    if optimization_report.build_id != runtime_cost_report.build_id {
        return Err(ToolingProductValidationErrorV1::InvalidCostProvenance);
    }
    if optimization_report.schema_version != 1
        || runtime_cost_report.schema_version != 1
        || optimization_report.validation_status != "valid"
    {
        return Err(ToolingProductValidationErrorV1::InvalidSourceReport);
    }
    let mut product = ToolingCompileCostReportV1 {
        schema: COMPILE_COST_TOOLING_SCHEMA_V1.into(),
        version: 1,
        report_id: String::new(),
        build_id: optimization_report.build_id.to_string(),
        optimization_policy: optimization_report.optimization_policy.to_string(),
        optimization_report_id: sha256_json(&optimization_report),
        runtime_cost_report_id: sha256_json(&runtime_cost_report),
        optimization_report,
        runtime_cost_report,
    };
    product.report_id = identity_without_field(&product, "reportId");
    Ok(product)
}

pub fn build_tooling_artifact_graph_v1(
    graph: &ProductionChunkGraph,
    artifact: &ProductionRuntimeArtifactV1,
) -> Result<ToolingArtifactGraphV1, ToolingProductValidationErrorV1> {
    validate_production_chunk_graph(graph)
        .map_err(|_| ToolingProductValidationErrorV1::ArtifactGraphTopologyDisagreement)?;
    if !validate_production_runtime_artifact(artifact, &artifact.build_id).is_empty()
        || artifact.entry.eager_chunk_id != graph.eager_chunk_id
    {
        return Err(ToolingProductValidationErrorV1::InvalidArtifactGraphProvenance);
    }
    let mut product = ToolingArtifactGraphV1 {
        schema: ARTIFACT_GRAPH_TOOLING_SCHEMA_V1.into(),
        version: 1,
        graph_id: String::new(),
        build_id: artifact.build_id.to_string(),
        runtime_protocol_version: artifact.runtime_protocol_version,
        optimization_policy: artifact.optimization_policy.to_string(),
        artifact_checksum: artifact.integrity.artifact_checksum.clone(),
        chunks: graph
            .chunks
            .iter()
            .map(|chunk| ToolingArtifactGraphChunkV1 {
                chunk_id: chunk.id.to_string(),
                kind: chunk_kind(chunk.kind).into(),
                module_filename: chunk.provisional_module_filename.clone(),
                activation_roots: sorted(chunk.activation_roots.clone()),
                root_kind: chunk.root_kind.clone(),
                program_fingerprints: sorted(
                    chunk.programs.iter().map(ToString::to_string).collect(),
                ),
                registration_only: chunk.registration_only,
            })
            .collect(),
        dependencies: graph
            .dependencies
            .iter()
            .map(|edge| ToolingArtifactGraphDependencyV1 {
                dependent_chunk_id: edge.dependent_chunk_id.to_string(),
                dependency_chunk_id: edge.dependency_chunk_id.to_string(),
            })
            .collect(),
        activations: graph
            .activation_plans
            .iter()
            .map(|plan| ToolingArtifactGraphActivationV1 {
                activation_root_id: plan.activation_root_id.clone(),
                root_chunk_id: plan.root_chunk_id.to_string(),
                shared_chunk_ids: sorted(
                    plan.shared_chunk_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                ),
            })
            .collect(),
    };
    product.chunks.sort_by(|a, b| a.chunk_id.cmp(&b.chunk_id));
    product.dependencies.sort();
    product.dependencies.dedup();
    product
        .activations
        .sort_by(|a, b| a.activation_root_id.cmp(&b.activation_root_id));
    product.graph_id = identity_without_field(&product, "graphId");
    Ok(product)
}

#[must_use]
pub fn tooling_build_trace_json_v1(value: &ToolingBuildTraceV1) -> String {
    canonical_json(value)
}
#[must_use]
pub fn tooling_compile_cost_report_json_v1(value: &ToolingCompileCostReportV1) -> String {
    canonical_json(value)
}
#[must_use]
pub fn tooling_artifact_graph_json_v1(value: &ToolingArtifactGraphV1) -> String {
    canonical_json(value)
}

pub fn decode_tooling_build_trace_v1(
    bytes: &[u8],
) -> Result<ToolingBuildTraceV1, ToolingProductValidationErrorV1> {
    let value: ToolingBuildTraceV1 =
        serde_json::from_slice(bytes).map_err(|_| ToolingProductValidationErrorV1::Noncanonical)?;
    validate_trace(&value)?;
    (tooling_build_trace_json_v1(&value).as_bytes() == bytes
        && value.trace_id == identity_without_field(&value, "traceId"))
    .then_some(value)
    .ok_or(ToolingProductValidationErrorV1::Noncanonical)
}
pub fn decode_tooling_compile_cost_report_v1(
    bytes: &[u8],
) -> Result<ToolingCompileCostReportV1, ToolingProductValidationErrorV1> {
    let value: ToolingCompileCostReportV1 =
        serde_json::from_slice(bytes).map_err(|_| ToolingProductValidationErrorV1::Noncanonical)?;
    let rebuilt = build_tooling_compile_cost_report_v1(
        value.optimization_report.clone(),
        value.runtime_cost_report.clone(),
    )?;
    (rebuilt == value && tooling_compile_cost_report_json_v1(&value).as_bytes() == bytes)
        .then_some(value)
        .ok_or(ToolingProductValidationErrorV1::Noncanonical)
}
pub fn decode_tooling_artifact_graph_v1(
    bytes: &[u8],
) -> Result<ToolingArtifactGraphV1, ToolingProductValidationErrorV1> {
    let value: ToolingArtifactGraphV1 =
        serde_json::from_slice(bytes).map_err(|_| ToolingProductValidationErrorV1::Noncanonical)?;
    if value.schema != ARTIFACT_GRAPH_TOOLING_SCHEMA_V1
        || value.version != 1
        || value.graph_id != identity_without_field(&value, "graphId")
        || tooling_artifact_graph_json_v1(&value).as_bytes() != bytes
    {
        return Err(ToolingProductValidationErrorV1::Noncanonical);
    }
    let canonical = value
        .chunks
        .windows(2)
        .all(|p| p[0].chunk_id < p[1].chunk_id)
        && value.dependencies.windows(2).all(|p| p[0] < p[1])
        && value
            .activations
            .windows(2)
            .all(|p| p[0].activation_root_id < p[1].activation_root_id);
    canonical
        .then_some(value)
        .ok_or(ToolingProductValidationErrorV1::ArtifactGraphTopologyDisagreement)
}

fn validate_trace(value: &ToolingBuildTraceV1) -> Result<(), ToolingProductValidationErrorV1> {
    if value.schema != BUILD_TRACE_TOOLING_SCHEMA_V1
        || value.version != 1
        || value.workspace_id.is_empty()
        || value.compiler_contract.is_empty()
        || value
            .stages
            .windows(2)
            .any(|p| p[0].ordinal >= p[1].ordinal)
        || value.stages.iter().any(|stage| {
            stage.identities.windows(2).any(|p| p[0].name >= p[1].name)
                || stage
                    .identities
                    .iter()
                    .any(|id| id.value.is_empty() || forbidden(&id.value))
        })
    {
        return Err(ToolingProductValidationErrorV1::InvalidTraceProvenance);
    }
    Ok(())
}
fn forbidden(value: &str) -> bool {
    ["/", "\\", "timestamp", "duration", "millisecond"]
        .iter()
        .any(|needle| value.contains(needle))
}
fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
fn chunk_kind(kind: ProductionChunkKind) -> &'static str {
    match kind {
        ProductionChunkKind::Eager => "eager",
        ProductionChunkKind::Root => "root",
        ProductionChunkKind::Shared => "shared",
    }
}
fn canonical_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("tooling product serializes") + "\n"
}
fn sha256_json<T: Serialize>(value: &T) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("tooling product serializes"))
    )
}
fn identity_without_field<T: Serialize>(value: &T, field: &str) -> String {
    let mut object = serde_json::to_value(value).expect("tooling product serializes");
    object
        .as_object_mut()
        .expect("tooling product is object")
        .remove(field);
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&object).expect("tooling product serializes"))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_production_reports, build_production_runtime_artifact,
        extract_production_chunk_graph, ExecutableProgramFingerprint, ProductionReportInputs,
        ProductionRootChunkInput, ResumeBoundaryId, ResumeBuildId, ResumeManifest,
        SharedChunkCandidatePlan,
    };
    use std::str::FromStr;

    fn phase_k_products() -> (
        ProductionChunkGraph,
        ProductionRuntimeArtifactV1,
        OptimizationReportV1,
        RuntimeCostReportV1,
    ) {
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
                activation_root_id: "root".into(),
                root_kind: "interaction".into(),
                programs: vec![ExecutableProgramFingerprint::for_canonical_opcode_stream(
                    b"a",
                )],
            }],
        )
        .expect("graph")
        .0;
        let artifact = build_production_runtime_artifact(&manifest, &graph).expect("artifact");
        let (optimization, cost) = build_production_reports(
            &artifact,
            &graph,
            &ProductionReportInputs {
                dead_products_removed: 1,
                constants_pooled: 2,
                programs_deduplicated: 3,
                shared_candidates_rejected: 0,
                binding_writes_coalesced: 4,
                development_bytes: 100,
                production_bytes: 80,
                cold_init_operation_count: 0,
                resume_restore_operation_count: 0,
                max_action_batch_operation_count: 0,
                max_scheduler_batch_width: 0,
                max_dom_patch_count_per_action: 0,
                retained_slot_count: 0,
            },
        );
        (graph, artifact, optimization, cost)
    }

    #[test]
    fn l11f_trace_is_canonical_source_free_and_strict() {
        let trace = build_tooling_build_trace_v1(
            "workspace:fixture".into(),
            "compiler-contract:v1".into(),
            Some("snapshot:fixture".into()),
            ToolingTraceOutcomeV1::Succeeded,
            vec![ToolingBuildTraceStageV1 {
                ordinal: 0,
                kind: ToolingTraceStageKindV1::L3Snapshot,
                outcome: ToolingTraceOutcomeV1::Succeeded,
                identities: vec![ToolingTraceIdentityV1 {
                    name: "snapshot_id".into(),
                    value: "snapshot:fixture".into(),
                }],
            }],
        )
        .unwrap();
        let bytes = tooling_build_trace_json_v1(&trace);
        assert_eq!(
            decode_tooling_build_trace_v1(bytes.as_bytes()).unwrap(),
            trace
        );
        assert!(!bytes.contains("timestamp"));
        assert!(decode_tooling_build_trace_v1(bytes.trim_end().as_bytes()).is_err());
    }

    #[test]
    fn l11f_cost_and_artifact_graph_are_canonical_source_free_products() {
        let (graph, artifact, optimization, cost) = phase_k_products();
        let report = build_tooling_compile_cost_report_v1(optimization, cost).unwrap();
        let report_bytes = tooling_compile_cost_report_json_v1(&report);
        assert_eq!(
            decode_tooling_compile_cost_report_v1(report_bytes.as_bytes()).unwrap(),
            report
        );
        assert!(!report_bytes.contains("timestamp"));

        let artifact_graph = build_tooling_artifact_graph_v1(&graph, &artifact).unwrap();
        let graph_bytes = tooling_artifact_graph_json_v1(&artifact_graph);
        assert_eq!(
            decode_tooling_artifact_graph_v1(graph_bytes.as_bytes()).unwrap(),
            artifact_graph
        );
        assert!(!graph_bytes.contains("production/"));
    }
}
