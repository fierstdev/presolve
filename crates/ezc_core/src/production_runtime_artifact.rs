//! K8 packed, validated production runtime artifact v1.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    validate_production_chunk_graph, OptimizationPolicyId, ProductionChunkGraph, ProductionChunkId,
    ProductionChunkKind, ResumeBuildId, ResumeManifest, RuntimeTableId,
    RESUME_RUNTIME_PROTOCOL_VERSION,
};

pub const PRODUCTION_RUNTIME_ARTIFACT_SCHEMA_VERSION: u32 = 1;
pub const PRODUCTION_RUNTIME_TABLE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionOrdinalWidth {
    U8,
    U16,
    U32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionOrdinalMapping {
    pub canonical_id: String,
    pub ordinal: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRuntimeTable {
    pub table_id: RuntimeTableId,
    pub table_kind: String,
    pub schema_version: u32,
    pub count: u32,
    pub ordinal_width: ProductionOrdinalWidth,
    pub checksum: String,
    pub mappings: Vec<ProductionOrdinalMapping>,
    pub referenced_table_ids: Vec<RuntimeTableId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRuntimeTableRegistry {
    pub tables: Vec<ProductionRuntimeTable>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionArtifactChunkRecord {
    pub chunk_id: ProductionChunkId,
    pub kind: String,
    pub module_filename: String,
    pub dependency_chunk_ids: Vec<ProductionChunkId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionArtifactActivationEntry {
    pub activation_root_id: String,
    pub root_chunk_id: ProductionChunkId,
    pub shared_chunk_ids: Vec<ProductionChunkId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionArtifactEntry {
    pub eager_chunk_id: ProductionChunkId,
    pub activations: Vec<ProductionArtifactActivationEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionArtifactIntegrity {
    pub artifact_checksum: String,
    pub table_checksums: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductionRuntimeArtifactV1 {
    pub schema_version: u32,
    pub build_id: ResumeBuildId,
    pub runtime_protocol_version: u32,
    pub optimization_policy: OptimizationPolicyId,
    pub tables: ProductionRuntimeTableRegistry,
    pub programs: Vec<String>,
    pub chunks: Vec<ProductionArtifactChunkRecord>,
    pub entry: ProductionArtifactEntry,
    pub integrity: ProductionArtifactIntegrity,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionArtifactIntegrityViolation {
    ArtifactChecksumMismatch,
    BuildIdMismatch,
    ChunkReferenceMismatch,
    GraphTopologyMismatch,
    InvalidOrdinalWidth,
    RuntimeProtocolMismatch,
    SchemaVersionMismatch,
    TableChecksumMismatch(RuntimeTableId),
    TableMappingMismatch(RuntimeTableId),
    UnknownTableReference(RuntimeTableId),
}

/// Assigns dense compiler-owned ordinals in canonical ID order.
#[must_use]
pub fn build_production_runtime_table(
    table_kind: &str,
    canonical_ids: &[String],
    referenced_table_ids: &[RuntimeTableId],
) -> Option<ProductionRuntimeTable> {
    let table_id = RuntimeTableId::for_artifact_table("production_runtime", table_kind)?;
    let mut ids = canonical_ids.to_vec();
    ids.sort();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }
    let mut references = referenced_table_ids.to_vec();
    references.sort();
    if references.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }
    let mut mappings = Vec::with_capacity(ids.len());
    for canonical_id in ids {
        mappings.push(ProductionOrdinalMapping {
            canonical_id,
            ordinal: u32::try_from(mappings.len()).ok()?,
        });
    }
    let mut table = ProductionRuntimeTable {
        table_id,
        table_kind: table_kind.to_string(),
        schema_version: PRODUCTION_RUNTIME_TABLE_SCHEMA_VERSION,
        count: u32::try_from(mappings.len()).ok()?,
        ordinal_width: ordinal_width(mappings.len()),
        checksum: String::new(),
        mappings,
        referenced_table_ids: references,
    };
    table.checksum = table_checksum(&table);
    Some(table)
}

/// Builds the production-only pack from frozen resume data and the validated K7 graph.
///
/// # Errors
///
/// Returns graph-integrity evidence rather than packing an invalid chunk topology.
///
/// # Panics
///
/// Panics only if compiler-derived canonical IDs cannot form one of the fixed
/// K8 tables, which indicates an earlier compiler invariant failure.
pub fn build_production_runtime_artifact(
    resume: &ResumeManifest,
    chunk_graph: &ProductionChunkGraph,
) -> Result<ProductionRuntimeArtifactV1, Vec<ProductionArtifactIntegrityViolation>> {
    if validate_production_chunk_graph(chunk_graph).is_err() {
        return Err(vec![
            ProductionArtifactIntegrityViolation::GraphTopologyMismatch,
        ]);
    }
    let programs = production_programs(resume);
    let activation_roots = chunk_graph
        .activation_plans
        .iter()
        .map(|plan| plan.activation_root_id.clone())
        .collect::<Vec<_>>();
    let chunk_ids = chunk_graph
        .chunks
        .iter()
        .map(|chunk| chunk.id.to_string())
        .collect::<Vec<_>>();
    let program_table = build_production_runtime_table("programs", &programs, &[])
        .expect("compiler-derived program IDs are canonical and unique");
    let chunk_table = build_production_runtime_table("chunks", &chunk_ids, &[])
        .expect("validated graph chunk IDs are canonical and unique");
    let activation_table = build_production_runtime_table(
        "activation_roots",
        &activation_roots,
        std::slice::from_ref(&chunk_table.table_id),
    )
    .expect("graph activation roots are canonical and unique");
    let tables = ProductionRuntimeTableRegistry {
        tables: vec![activation_table, chunk_table, program_table],
    };
    let dependencies = chunk_graph.dependencies.iter().fold(
        BTreeMap::<ProductionChunkId, Vec<ProductionChunkId>>::new(),
        |mut map, edge| {
            map.entry(edge.dependent_chunk_id.clone())
                .or_default()
                .push(edge.dependency_chunk_id.clone());
            map
        },
    );
    let mut chunks = chunk_graph
        .chunks
        .iter()
        .map(|chunk| ProductionArtifactChunkRecord {
            chunk_id: chunk.id.clone(),
            kind: chunk_kind_name(chunk.kind).to_string(),
            module_filename: chunk.provisional_module_filename.clone(),
            dependency_chunk_ids: dependencies.get(&chunk.id).cloned().unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    chunks.sort_by(|left, right| left.chunk_id.cmp(&right.chunk_id));
    for chunk in &mut chunks {
        chunk.dependency_chunk_ids.sort();
        chunk.dependency_chunk_ids.dedup();
    }
    let mut activations = chunk_graph
        .activation_plans
        .iter()
        .map(|plan| ProductionArtifactActivationEntry {
            activation_root_id: plan.activation_root_id.clone(),
            root_chunk_id: plan.root_chunk_id.clone(),
            shared_chunk_ids: plan.shared_chunk_ids.clone(),
        })
        .collect::<Vec<_>>();
    activations.sort_by(|left, right| left.activation_root_id.cmp(&right.activation_root_id));
    let mut artifact = ProductionRuntimeArtifactV1 {
        schema_version: PRODUCTION_RUNTIME_ARTIFACT_SCHEMA_VERSION,
        build_id: resume.build_id.clone(),
        runtime_protocol_version: resume.runtime_protocol_version,
        optimization_policy: OptimizationPolicyId::production_v1(),
        tables,
        programs,
        chunks,
        entry: ProductionArtifactEntry {
            eager_chunk_id: chunk_graph.eager_chunk_id.clone(),
            activations,
        },
        integrity: ProductionArtifactIntegrity {
            artifact_checksum: String::new(),
            table_checksums: Vec::new(),
        },
    };
    artifact.integrity.table_checksums = artifact
        .tables
        .tables
        .iter()
        .map(|table| table.checksum.clone())
        .collect();
    artifact.integrity.artifact_checksum = artifact_checksum(&artifact);
    Ok(artifact)
}

#[must_use]
///
/// # Panics
///
/// Panics only if an in-memory compiler-produced artifact cannot serialize.
pub fn production_runtime_artifact_json(artifact: &ProductionRuntimeArtifactV1) -> String {
    serde_json::to_string(artifact).expect("production runtime artifact should serialize") + "\n"
}

/// Parses and validates the closed v1 production artifact before runtime use.
///
/// # Errors
///
/// Returns all deterministic integrity violations; unknown fields are rejected
/// by the v1 deserializer before this validation begins.
pub fn parse_production_runtime_artifact_v1(
    json: &str,
    expected_build_id: &ResumeBuildId,
) -> Result<ProductionRuntimeArtifactV1, Vec<ProductionArtifactIntegrityViolation>> {
    let artifact = serde_json::from_str::<ProductionRuntimeArtifactV1>(json)
        .map_err(|_| vec![ProductionArtifactIntegrityViolation::SchemaVersionMismatch])?;
    let errors = validate_production_runtime_artifact(&artifact, expected_build_id);
    errors.is_empty().then_some(artifact).ok_or(errors)
}

#[must_use]
pub fn validate_production_runtime_artifact(
    artifact: &ProductionRuntimeArtifactV1,
    expected_build_id: &ResumeBuildId,
) -> Vec<ProductionArtifactIntegrityViolation> {
    let mut violations = Vec::new();
    if artifact.schema_version != PRODUCTION_RUNTIME_ARTIFACT_SCHEMA_VERSION {
        violations.push(ProductionArtifactIntegrityViolation::SchemaVersionMismatch);
    }
    if artifact.runtime_protocol_version != RESUME_RUNTIME_PROTOCOL_VERSION {
        violations.push(ProductionArtifactIntegrityViolation::RuntimeProtocolMismatch);
    }
    if &artifact.build_id != expected_build_id {
        violations.push(ProductionArtifactIntegrityViolation::BuildIdMismatch);
    }
    let table_ids = artifact
        .tables
        .tables
        .iter()
        .map(|table| table.table_id.clone())
        .collect::<BTreeSet<_>>();
    for table in &artifact.tables.tables {
        if table.checksum != table_checksum(table) {
            violations.push(ProductionArtifactIntegrityViolation::TableChecksumMismatch(
                table.table_id.clone(),
            ));
        }
        let mappings_are_canonical = u32::try_from(table.mappings.len()) == Ok(table.count)
            && table.ordinal_width == ordinal_width(table.mappings.len())
            && table
                .mappings
                .iter()
                .enumerate()
                .all(|(ordinal, mapping)| u32::try_from(ordinal) == Ok(mapping.ordinal))
            && table
                .mappings
                .windows(2)
                .all(|pair| pair[0].canonical_id < pair[1].canonical_id);
        if !mappings_are_canonical
            || table.schema_version != PRODUCTION_RUNTIME_TABLE_SCHEMA_VERSION
        {
            violations.push(ProductionArtifactIntegrityViolation::TableMappingMismatch(
                table.table_id.clone(),
            ));
        }
        if table.ordinal_width != ordinal_width(table.count as usize) {
            violations.push(ProductionArtifactIntegrityViolation::InvalidOrdinalWidth);
        }
        for reference in &table.referenced_table_ids {
            if !table_ids.contains(reference) {
                violations.push(ProductionArtifactIntegrityViolation::UnknownTableReference(
                    reference.clone(),
                ));
            }
        }
    }
    let chunk_ids = artifact
        .chunks
        .iter()
        .map(|chunk| chunk.chunk_id.clone())
        .collect::<BTreeSet<_>>();
    let chunk_references_valid = chunk_ids.contains(&artifact.entry.eager_chunk_id)
        && artifact.chunks.iter().all(|chunk| {
            chunk
                .dependency_chunk_ids
                .iter()
                .all(|dependency| chunk_ids.contains(dependency))
        })
        && artifact.entry.activations.iter().all(|activation| {
            chunk_ids.contains(&activation.root_chunk_id)
                && activation
                    .shared_chunk_ids
                    .iter()
                    .all(|shared| chunk_ids.contains(shared))
        });
    if !chunk_references_valid {
        violations.push(ProductionArtifactIntegrityViolation::ChunkReferenceMismatch);
    }
    if artifact.integrity.artifact_checksum != artifact_checksum(artifact) {
        violations.push(ProductionArtifactIntegrityViolation::ArtifactChecksumMismatch);
    }
    violations.sort();
    violations.dedup();
    violations
}

fn production_programs(resume: &ResumeManifest) -> Vec<String> {
    let mut programs = BTreeSet::new();
    for chunk in &resume.chunks {
        programs.extend(chunk.provided_program_ids.iter().cloned());
    }
    programs.extend(
        resume
            .capture_programs
            .iter()
            .map(|program| program.program_id.to_string()),
    );
    programs.extend(
        resume
            .restore_programs
            .iter()
            .map(|program| program.program_id.to_string()),
    );
    programs.extend(
        resume
            .events
            .iter()
            .map(|event| event.action_or_submit_program_id.clone()),
    );
    programs.into_iter().collect()
}

fn ordinal_width(count: usize) -> ProductionOrdinalWidth {
    if count <= 255 {
        ProductionOrdinalWidth::U8
    } else if count <= 65_535 {
        ProductionOrdinalWidth::U16
    } else {
        ProductionOrdinalWidth::U32
    }
}

fn table_checksum(table: &ProductionRuntimeTable) -> String {
    let mut bytes = format!(
        "table:{}\nschema:{}\ncount:{}\nwidth:{:?}\n",
        table.table_id, table.schema_version, table.count, table.ordinal_width
    )
    .into_bytes();
    for mapping in &table.mappings {
        bytes.extend_from_slice(mapping.canonical_id.as_bytes());
        bytes.extend_from_slice(format!("\0{}\n", mapping.ordinal).as_bytes());
    }
    for reference in &table.referenced_table_ids {
        bytes.extend_from_slice(reference.to_string().as_bytes());
        bytes.push(b'\n');
    }
    format!("{:x}", Sha256::digest(bytes))
}

fn artifact_checksum(artifact: &ProductionRuntimeArtifactV1) -> String {
    let mut canonical = artifact.clone();
    canonical.integrity.artifact_checksum.clear();
    format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&canonical).expect("production artifact should serialize")
        )
    )
}

const fn chunk_kind_name(kind: ProductionChunkKind) -> &'static str {
    match kind {
        ProductionChunkKind::Eager => "eager",
        ProductionChunkKind::Root => "root",
        ProductionChunkKind::Shared => "shared",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        extract_production_chunk_graph, ExecutableProgramFingerprint, ProductionRootChunkInput,
        ResumeBoundaryId, ResumeManifest, SharedChunkCandidatePlan,
    };
    use std::str::FromStr;

    fn manifest() -> ResumeManifest {
        ResumeManifest {
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
        }
    }

    fn graph() -> ProductionChunkGraph {
        extract_production_chunk_graph(
            &SharedChunkCandidatePlan {
                candidates: Vec::new(),
                rejections: Vec::new(),
            },
            &[ProductionRootChunkInput {
                activation_root_id: "root-a".to_string(),
                root_kind: "interaction".to_string(),
                programs: vec![ExecutableProgramFingerprint::for_canonical_opcode_stream(
                    b"a",
                )],
            }],
        )
        .expect("graph")
        .0
    }

    #[test]
    fn k8_assigns_canonical_dense_ordinals_and_width_boundaries() {
        let ids = (0..256)
            .map(|index| format!("id-{index:03}"))
            .collect::<Vec<_>>();
        let table = build_production_runtime_table("programs", &ids, &[]).expect("table");
        assert_eq!(table.mappings[0].canonical_id, "id-000");
        assert_eq!(table.mappings[255].ordinal, 255);
        assert_eq!(table.ordinal_width, ProductionOrdinalWidth::U16);
        assert!(build_production_runtime_table(
            "programs",
            &["same".to_string(), "same".to_string()],
            &[]
        )
        .is_none());
    }

    #[test]
    fn k8_round_trips_canonical_artifact_and_rejects_integrity_drift() {
        let manifest = manifest();
        let artifact = build_production_runtime_artifact(&manifest, &graph()).expect("artifact");
        let json = production_runtime_artifact_json(&artifact);
        assert!(json.ends_with('\n'));
        assert!(!json.contains("src/"));
        assert_eq!(
            parse_production_runtime_artifact_v1(&json, &manifest.build_id),
            Ok(artifact.clone())
        );
        let mut checksum_drift = artifact.clone();
        checksum_drift.tables.tables[0].checksum = "bad".to_string();
        assert!(
            validate_production_runtime_artifact(&checksum_drift, &manifest.build_id)
                .iter()
                .any(|violation| matches!(
                    violation,
                    ProductionArtifactIntegrityViolation::TableChecksumMismatch(_)
                ))
        );
    }

    #[test]
    fn k8_rejects_unknown_chunk_reference_and_build_mismatch() {
        let manifest = manifest();
        let mut artifact =
            build_production_runtime_artifact(&manifest, &graph()).expect("artifact");
        artifact.chunks[0]
            .dependency_chunk_ids
            .push(ProductionChunkId::from_str("production-chunk:unknown").expect("chunk ID"));
        let errors = validate_production_runtime_artifact(
            &artifact,
            &ResumeBuildId::from_str("resume-build:other").expect("build ID"),
        );
        assert!(errors.contains(&ProductionArtifactIntegrityViolation::BuildIdMismatch));
        assert!(errors.contains(&ProductionArtifactIntegrityViolation::ChunkReferenceMismatch));
        assert!(errors.contains(&ProductionArtifactIntegrityViolation::ArtifactChecksumMismatch));
    }
}
