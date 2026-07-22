//! L11-F and L12-C canonical, source-free tooling products.

#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::platform::{SnapshotUnit, WorkspaceSnapshot};
use crate::{
    validate_production_chunk_graph, validate_production_runtime_artifact,
    ApplicationSemanticModel, ComponentDiagnosticSeverity, OptimizationReportV1,
    ProductionChunkGraph, ProductionChunkKind, ProductionRuntimeArtifactV1, RuntimeCostReportV1,
    SemanticEntityKind, SemanticReferenceKind, SourceProvenance,
};

pub const BUILD_TRACE_TOOLING_SCHEMA_V1: &str = "presolve.build-trace";
pub const COMPILE_COST_TOOLING_SCHEMA_V1: &str = "presolve.compile-cost-report";
pub const ARTIFACT_GRAPH_TOOLING_SCHEMA_V1: &str = "presolve.artifact-graph";
pub const QUERY_SNAPSHOT_TOOLING_SCHEMA_V1: &str = "presolve.query-snapshot";

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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQuerySnapshotSourceUnitV1 {
    pub source_unit_id: String,
    pub source_revision_id: String,
    pub source_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQueryRangeV1 {
    pub source_unit_id: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolingQuerySemanticKindV1 {
    Component,
    StateField,
    Method,
    Context,
    Provider,
    Consumer,
    Form,
    FormField,
    FormFieldBinding,
    ValidationRule,
    Slot,
    ComponentInvocation,
    ComponentInstance,
    BlockedComponentInstance,
    SlotContentFragment,
    SlotOutlet,
    Computed,
    Effect,
    Parameter,
    LocalVariable,
    Action,
    EventHandler,
    Template,
    TemplateEntity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolingQueryReferenceKindV1 {
    ActionState,
    ComputedState,
    ComputedComputed,
    EffectState,
    EffectComputed,
    ProvidesContext,
    ConsumesContext,
    ResolvesToProvider,
    EventMethod,
    TemplateState,
    TemplateComputed,
    TemplateLocal,
    FieldBindingField,
    FieldBindingForm,
    ValidationRuleField,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolingQueryDiagnosticSeverityV1 {
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQuerySemanticRecordV1 {
    pub query_semantic_id: String,
    pub kind: ToolingQuerySemanticKindV1,
    pub range: ToolingQueryRangeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQueryReferenceV1 {
    pub kind: ToolingQueryReferenceKindV1,
    pub source_query_semantic_id: String,
    pub target_query_semantic_id: String,
    pub range: ToolingQueryRangeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQueryDiagnosticSecondaryV1 {
    pub range: ToolingQueryRangeV1,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQueryDiagnosticV1 {
    pub code: String,
    pub severity: ToolingQueryDiagnosticSeverityV1,
    pub message: String,
    pub primary_range: Option<ToolingQueryRangeV1>,
    pub secondary: Vec<ToolingQueryDiagnosticSecondaryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolingQuerySnapshotV1 {
    pub schema: String,
    pub version: u32,
    pub query_snapshot_id: String,
    pub workspace_id: String,
    pub snapshot_id: String,
    pub source_units: Vec<ToolingQuerySnapshotSourceUnitV1>,
    pub semantic_records: Vec<ToolingQuerySemanticRecordV1>,
    pub references: Vec<ToolingQueryReferenceV1>,
    pub diagnostics: Vec<ToolingQueryDiagnosticV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolingProductValidationErrorV1 {
    InvalidTraceProvenance,
    InvalidCostProvenance,
    InvalidSourceReport,
    InvalidArtifactGraphProvenance,
    ArtifactGraphTopologyDisagreement,
    InvalidQuerySnapshotProvenance,
    InvalidQuerySnapshotBinding,
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

pub(crate) fn build_tooling_query_snapshot_v1(
    snapshot: &WorkspaceSnapshot,
    model: &ApplicationSemanticModel,
) -> Result<ToolingQuerySnapshotV1, ToolingProductValidationErrorV1> {
    snapshot
        .validate()
        .map_err(|_| ToolingProductValidationErrorV1::InvalidQuerySnapshotBinding)?;
    let source_lengths = snapshot
        .units
        .iter()
        .map(|unit| (unit.path.as_str(), unit))
        .collect::<std::collections::BTreeMap<_, _>>();
    let make_range =
        |provenance: &SourceProvenance| query_range_from_provenance(provenance, &source_lengths);
    let mut query_ids = std::collections::BTreeMap::new();
    let mut semantic_records = Vec::new();
    for semantic_id in model.ownership.keys() {
        let Some(provenance) = model.provenance(semantic_id) else {
            continue;
        };
        let entity = model
            .entity(semantic_id)
            .ok_or(ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance)?;
        let query_semantic_id = query_semantic_id(semantic_id.as_str());
        query_ids.insert(semantic_id.clone(), query_semantic_id.clone());
        semantic_records.push(ToolingQuerySemanticRecordV1 {
            query_semantic_id,
            kind: query_semantic_kind(entity.kind()),
            range: make_range(provenance)?,
        });
    }
    semantic_records.sort_by_key(query_semantic_record_key);

    let mut references = Vec::new();
    for reference in &model.references {
        let (Some(source_query_semantic_id), Some(target_query_semantic_id)) = (
            query_ids.get(&reference.source),
            query_ids.get(&reference.target),
        ) else {
            continue;
        };
        references.push(ToolingQueryReferenceV1 {
            kind: query_reference_kind(reference.kind),
            source_query_semantic_id: source_query_semantic_id.clone(),
            target_query_semantic_id: target_query_semantic_id.clone(),
            range: make_range(&reference.provenance)?,
        });
    }
    references.sort_by_key(query_reference_key);
    references.dedup();

    let mut diagnostics = model
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let mut secondary = diagnostic
                .secondary_labels
                .iter()
                .map(|label| {
                    Ok(ToolingQueryDiagnosticSecondaryV1 {
                        range: make_range(&label.provenance)?,
                        message: label.message.clone(),
                    })
                })
                .collect::<Result<Vec<_>, ToolingProductValidationErrorV1>>()?;
            secondary.sort_by_key(query_diagnostic_secondary_key);
            secondary.dedup();
            Ok(ToolingQueryDiagnosticV1 {
                code: diagnostic.code.clone(),
                severity: query_diagnostic_severity(diagnostic.severity),
                message: diagnostic.message.clone(),
                primary_range: diagnostic.provenance.as_ref().map(make_range).transpose()?,
                secondary,
            })
        })
        .collect::<Result<Vec<_>, ToolingProductValidationErrorV1>>()?;
    diagnostics.sort_by_key(query_diagnostic_key);

    let mut product = ToolingQuerySnapshotV1 {
        schema: QUERY_SNAPSHOT_TOOLING_SCHEMA_V1.into(),
        version: 1,
        query_snapshot_id: String::new(),
        workspace_id: snapshot.workspace_id.as_str().into(),
        snapshot_id: snapshot.snapshot_id.as_str().into(),
        source_units: snapshot
            .units
            .iter()
            .map(|unit| ToolingQuerySnapshotSourceUnitV1 {
                source_unit_id: unit.source_unit_id.as_str().into(),
                source_revision_id: unit.source_revision_id.as_str().into(),
                source_length: unit.source_length,
            })
            .collect(),
        semantic_records,
        references,
        diagnostics,
    };
    product.source_units.sort();
    validate_query_snapshot(&product)?;
    product.query_snapshot_id = identity_without_field(&product, "querySnapshotId");
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
#[must_use]
pub fn tooling_query_snapshot_json_v1(value: &ToolingQuerySnapshotV1) -> String {
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
pub fn decode_tooling_query_snapshot_v1(
    bytes: &[u8],
) -> Result<ToolingQuerySnapshotV1, ToolingProductValidationErrorV1> {
    let value: ToolingQuerySnapshotV1 =
        serde_json::from_slice(bytes).map_err(|_| ToolingProductValidationErrorV1::Noncanonical)?;
    validate_query_snapshot(&value)?;
    (value.query_snapshot_id == identity_without_field(&value, "querySnapshotId")
        && tooling_query_snapshot_json_v1(&value).as_bytes() == bytes)
        .then_some(value)
        .ok_or(ToolingProductValidationErrorV1::Noncanonical)
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
fn validate_query_snapshot(
    value: &ToolingQuerySnapshotV1,
) -> Result<(), ToolingProductValidationErrorV1> {
    if value.schema != QUERY_SNAPSHOT_TOOLING_SCHEMA_V1
        || value.version != 1
        || value.workspace_id.is_empty()
        || value.snapshot_id.is_empty()
        || value.source_units.is_empty()
        || value
            .source_units
            .windows(2)
            .any(|pair| pair[0].source_unit_id >= pair[1].source_unit_id)
        || value
            .source_units
            .iter()
            .any(|unit| unit.source_unit_id.is_empty() || unit.source_revision_id.is_empty())
    {
        return Err(ToolingProductValidationErrorV1::InvalidQuerySnapshotBinding);
    }
    let lengths = value
        .source_units
        .iter()
        .map(|unit| (unit.source_unit_id.as_str(), unit.source_length))
        .collect::<std::collections::BTreeMap<_, _>>();
    let valid_range = |range: &ToolingQueryRangeV1| {
        lengths
            .get(range.source_unit_id.as_str())
            .is_some_and(|length| range.start <= range.end && range.end <= *length)
    };
    if value
        .semantic_records
        .iter()
        .any(|record| record.query_semantic_id.is_empty() || !valid_range(&record.range))
        || value
            .semantic_records
            .windows(2)
            .any(|pair| query_semantic_record_key(&pair[0]) >= query_semantic_record_key(&pair[1]))
    {
        return Err(ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance);
    }
    let query_semantic_ids = value
        .semantic_records
        .iter()
        .map(|record| record.query_semantic_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if query_semantic_ids.len() != value.semantic_records.len()
        || value.references.iter().any(|reference| {
            !valid_range(&reference.range)
                || !query_semantic_ids.contains(reference.source_query_semantic_id.as_str())
                || !query_semantic_ids.contains(reference.target_query_semantic_id.as_str())
        })
        || value
            .references
            .windows(2)
            .any(|pair| query_reference_key(&pair[0]) >= query_reference_key(&pair[1]))
        || value.diagnostics.iter().any(|diagnostic| {
            diagnostic.code.is_empty()
                || diagnostic.message.is_empty()
                || diagnostic
                    .primary_range
                    .as_ref()
                    .is_some_and(|range| !valid_range(range))
                || diagnostic
                    .secondary
                    .iter()
                    .any(|secondary| secondary.message.is_empty() || !valid_range(&secondary.range))
                || diagnostic.secondary.windows(2).any(|pair| {
                    query_diagnostic_secondary_key(&pair[0])
                        >= query_diagnostic_secondary_key(&pair[1])
                })
        })
        || value
            .diagnostics
            .windows(2)
            .any(|pair| query_diagnostic_key(&pair[0]) > query_diagnostic_key(&pair[1]))
    {
        return Err(ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance);
    }
    Ok(())
}
fn query_range_from_provenance(
    provenance: &SourceProvenance,
    source_units: &std::collections::BTreeMap<&str, &SnapshotUnit>,
) -> Result<ToolingQueryRangeV1, ToolingProductValidationErrorV1> {
    let path = provenance.path.to_string_lossy();
    let source_unit = source_units
        .get(path.as_ref())
        .ok_or(ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance)?;
    let start = u64::try_from(provenance.span.start)
        .map_err(|_| ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance)?;
    let end = u64::try_from(provenance.span.end)
        .map_err(|_| ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance)?;
    (start <= end && end <= source_unit.source_length)
        .then_some(ToolingQueryRangeV1 {
            source_unit_id: source_unit.source_unit_id.as_str().into(),
            start,
            end,
        })
        .ok_or(ToolingProductValidationErrorV1::InvalidQuerySnapshotProvenance)
}
fn query_semantic_id(semantic_id: &str) -> String {
    let mut bytes = b"query-semantic-v1\0".to_vec();
    bytes.extend_from_slice(semantic_id.as_bytes());
    format!("query-semantic:sha256:{:x}", Sha256::digest(bytes))
}
fn query_semantic_kind(kind: SemanticEntityKind) -> ToolingQuerySemanticKindV1 {
    match kind {
        SemanticEntityKind::Component => ToolingQuerySemanticKindV1::Component,
        SemanticEntityKind::StateField => ToolingQuerySemanticKindV1::StateField,
        SemanticEntityKind::Method => ToolingQuerySemanticKindV1::Method,
        SemanticEntityKind::Context => ToolingQuerySemanticKindV1::Context,
        SemanticEntityKind::Provider => ToolingQuerySemanticKindV1::Provider,
        SemanticEntityKind::Consumer => ToolingQuerySemanticKindV1::Consumer,
        SemanticEntityKind::Form => ToolingQuerySemanticKindV1::Form,
        SemanticEntityKind::FormField => ToolingQuerySemanticKindV1::FormField,
        SemanticEntityKind::FormFieldBinding => ToolingQuerySemanticKindV1::FormFieldBinding,
        SemanticEntityKind::ValidationRule => ToolingQuerySemanticKindV1::ValidationRule,
        SemanticEntityKind::Slot => ToolingQuerySemanticKindV1::Slot,
        SemanticEntityKind::ComponentInvocation => ToolingQuerySemanticKindV1::ComponentInvocation,
        SemanticEntityKind::ComponentInstance => ToolingQuerySemanticKindV1::ComponentInstance,
        SemanticEntityKind::BlockedComponentInstance => {
            ToolingQuerySemanticKindV1::BlockedComponentInstance
        }
        SemanticEntityKind::SlotContentFragment => ToolingQuerySemanticKindV1::SlotContentFragment,
        SemanticEntityKind::SlotOutlet => ToolingQuerySemanticKindV1::SlotOutlet,
        SemanticEntityKind::Computed => ToolingQuerySemanticKindV1::Computed,
        SemanticEntityKind::Effect => ToolingQuerySemanticKindV1::Effect,
        SemanticEntityKind::Parameter => ToolingQuerySemanticKindV1::Parameter,
        SemanticEntityKind::LocalVariable => ToolingQuerySemanticKindV1::LocalVariable,
        SemanticEntityKind::Action => ToolingQuerySemanticKindV1::Action,
        SemanticEntityKind::EventHandler => ToolingQuerySemanticKindV1::EventHandler,
        SemanticEntityKind::Template => ToolingQuerySemanticKindV1::Template,
        SemanticEntityKind::TemplateEntity => ToolingQuerySemanticKindV1::TemplateEntity,
    }
}
fn query_reference_kind(kind: SemanticReferenceKind) -> ToolingQueryReferenceKindV1 {
    match kind {
        SemanticReferenceKind::ActionState => ToolingQueryReferenceKindV1::ActionState,
        SemanticReferenceKind::ComputedState => ToolingQueryReferenceKindV1::ComputedState,
        SemanticReferenceKind::ComputedComputed => ToolingQueryReferenceKindV1::ComputedComputed,
        SemanticReferenceKind::EffectState => ToolingQueryReferenceKindV1::EffectState,
        SemanticReferenceKind::EffectComputed => ToolingQueryReferenceKindV1::EffectComputed,
        SemanticReferenceKind::ProvidesContext => ToolingQueryReferenceKindV1::ProvidesContext,
        SemanticReferenceKind::ConsumesContext => ToolingQueryReferenceKindV1::ConsumesContext,
        SemanticReferenceKind::ResolvesToProvider => {
            ToolingQueryReferenceKindV1::ResolvesToProvider
        }
        SemanticReferenceKind::EventMethod => ToolingQueryReferenceKindV1::EventMethod,
        SemanticReferenceKind::TemplateState => ToolingQueryReferenceKindV1::TemplateState,
        SemanticReferenceKind::TemplateComputed => ToolingQueryReferenceKindV1::TemplateComputed,
        SemanticReferenceKind::TemplateLocal => ToolingQueryReferenceKindV1::TemplateLocal,
        SemanticReferenceKind::FieldBindingField => ToolingQueryReferenceKindV1::FieldBindingField,
        SemanticReferenceKind::FieldBindingForm => ToolingQueryReferenceKindV1::FieldBindingForm,
        SemanticReferenceKind::ValidationRuleField => {
            ToolingQueryReferenceKindV1::ValidationRuleField
        }
    }
}
fn query_diagnostic_severity(
    severity: ComponentDiagnosticSeverity,
) -> ToolingQueryDiagnosticSeverityV1 {
    match severity {
        ComponentDiagnosticSeverity::Error => ToolingQueryDiagnosticSeverityV1::Error,
    }
}
fn query_semantic_record_key(record: &ToolingQuerySemanticRecordV1) -> (String, u64, u64, String) {
    (
        record.range.source_unit_id.clone(),
        record.range.start,
        record.range.end,
        record.query_semantic_id.clone(),
    )
}
fn query_reference_key(
    reference: &ToolingQueryReferenceV1,
) -> (String, u64, u64, String, String, String) {
    (
        reference.range.source_unit_id.clone(),
        reference.range.start,
        reference.range.end,
        reference.source_query_semantic_id.clone(),
        reference.target_query_semantic_id.clone(),
        format!("{:?}", reference.kind),
    )
}
fn query_diagnostic_secondary_key(
    secondary: &ToolingQueryDiagnosticSecondaryV1,
) -> (String, u64, u64, String) {
    (
        secondary.range.source_unit_id.clone(),
        secondary.range.start,
        secondary.range.end,
        secondary.message.clone(),
    )
}
fn query_diagnostic_key(
    diagnostic: &ToolingQueryDiagnosticV1,
) -> (String, u64, u64, String, String) {
    let range = diagnostic.primary_range.as_ref();
    (
        range.map_or_else(String::new, |range| range.source_unit_id.clone()),
        range.map_or(0, |range| range.start),
        range.map_or(0, |range| range.end),
        diagnostic.code.clone(),
        diagnostic.message.clone(),
    )
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
    use crate::platform::{
        derive_workspace_id_v1, CacheLimits, CancellationToken, CompilationOutcome,
        CompileWorkspaceRequest, CompilerSessionState, RequestedCompilationMode, WorkspaceInput,
        WorkspaceSource,
    };
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

    #[test]
    fn l12c_query_snapshot_is_compiler_produced_source_free_and_strict() {
        let source = r#"@component("x-query-fixture")
class QueryFixture extends Component {
  value = state(1)
  render() { return <main>{this.value}</main>; }
}
"#;
        let workspace = WorkspaceInput::new(vec![WorkspaceSource {
            path: "src/QueryFixture.tsx".into(),
            source: source.into(),
            language: None,
        }]);
        let workspace_id = derive_workspace_id_v1(&workspace.configuration).unwrap();
        let mut session = CompilerSessionState::new(
            workspace_id,
            workspace.compiler_contract.clone(),
            CacheLimits::default(),
        );
        let CompilationOutcome::Committed(result) =
            session.compile_workspace(CompileWorkspaceRequest {
                workspace,
                mode: RequestedCompilationMode::Full,
                cancellation: CancellationToken::new(),
            })
        else {
            panic!("query snapshot fixture compilation must commit");
        };
        let snapshot = result.query_snapshot.as_ref().clone();
        let bytes = tooling_query_snapshot_json_v1(&snapshot);
        assert_eq!(
            decode_tooling_query_snapshot_v1(bytes.as_bytes()).unwrap(),
            snapshot
        );
        assert_eq!(
            bytes,
            include_str!("../fixtures/tooling/query-snapshot-v1.json")
        );
        assert_eq!(snapshot.workspace_id, result.snapshot.workspace_id.as_str());
        assert_eq!(snapshot.snapshot_id, result.snapshot.snapshot_id.as_str());
        assert!(!bytes.contains("QueryFixture.tsx"), "{bytes}");
        assert!(!bytes.contains("x-query-fixture"));
        assert!(decode_tooling_query_snapshot_v1(bytes.trim_end().as_bytes()).is_err());
        assert!(decode_tooling_query_snapshot_v1(
            bytes
                .replacen("query-semantic:", "query-semantiX:", 1)
                .as_bytes()
        )
        .is_err());
    }

    #[test]
    fn l12c_query_snapshot_is_input_enumeration_independent() {
        let alpha = WorkspaceSource {
            path: "src/Alpha.tsx".into(),
            source: "@component(\"x-alpha\") class Alpha extends Component { render() { return <main />; } }\n".into(),
            language: None,
        };
        let beta = WorkspaceSource {
            path: "src/Beta.tsx".into(),
            source: "@component(\"x-beta\") class Beta extends Component { render() { return <aside />; } }\n".into(),
            language: None,
        };
        let compile = |sources| {
            let workspace = WorkspaceInput::new(sources);
            let workspace_id = derive_workspace_id_v1(&workspace.configuration).unwrap();
            let mut session = CompilerSessionState::new(
                workspace_id,
                workspace.compiler_contract.clone(),
                CacheLimits::default(),
            );
            let CompilationOutcome::Committed(result) =
                session.compile_workspace(CompileWorkspaceRequest {
                    workspace,
                    mode: RequestedCompilationMode::Full,
                    cancellation: CancellationToken::new(),
                })
            else {
                panic!("query enumeration fixture compilation must commit");
            };
            tooling_query_snapshot_json_v1(result.query_snapshot.as_ref())
        };
        assert_eq!(
            compile(vec![alpha.clone(), beta.clone()]),
            compile(vec![beta, alpha])
        );
    }
}
