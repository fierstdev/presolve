//! K10 validated canonical-ID to ordinal production bootstrap indexes.

use std::collections::{BTreeMap, BTreeSet};

use crate::{ProductionRuntimeArtifactV1, ProductionRuntimeTableRegistry};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionRuntimeIndexes {
    pub anchors: BTreeMap<String, u32>,
    pub events: BTreeMap<String, u32>,
    pub activations: BTreeMap<String, u32>,
    pub activation_roots: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionBootstrapPlan {
    pub indexes: ProductionRuntimeIndexes,
    pub listener_event_types: Vec<String>,
    pub ready: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionBootstrapBlock {
    MissingTable(String),
    UnknownEventAnchor(String),
    UnknownEventActivation(String),
}

/// Builds closed ordinal indexes only after the K8 artifact has been validated.
///
/// # Errors
///
/// Returns deterministic blocks when frozen manifest event references do not
/// resolve through the packed artifact tables.
pub fn build_production_bootstrap_plan(
    artifact: &ProductionRuntimeArtifactV1,
    manifest: &crate::ResumeManifest,
) -> Result<ProductionBootstrapPlan, Vec<ProductionBootstrapBlock>> {
    let indexes = ProductionRuntimeIndexes {
        anchors: table_index(&artifact.tables, "anchors")?,
        events: table_index(&artifact.tables, "events")?,
        activations: table_index(&artifact.tables, "activations")?,
        activation_roots: table_index(&artifact.tables, "activation_roots")?,
    };
    let activation_by_event = manifest
        .activations
        .iter()
        .filter_map(|activation| {
            activation
                .event_id
                .as_ref()
                .map(|event| (event.to_string(), activation.activation_id.to_string()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut blocks = Vec::new();
    let mut listener_event_types = BTreeSet::new();
    for event in &manifest.events {
        let event_id = event.resume_event_id.to_string();
        if !indexes.events.contains_key(&event_id)
            || !indexes
                .anchors
                .contains_key(&event.exact_target_anchor_id.to_string())
        {
            blocks.push(ProductionBootstrapBlock::UnknownEventAnchor(
                event_id.clone(),
            ));
        }
        if activation_by_event
            .get(&event_id)
            .is_none_or(|activation| !indexes.activations.contains_key(activation))
        {
            blocks.push(ProductionBootstrapBlock::UnknownEventActivation(event_id));
        }
        listener_event_types.insert(event.event_type.clone());
    }
    blocks.sort();
    blocks.dedup();
    if !blocks.is_empty() {
        return Err(blocks);
    }
    Ok(ProductionBootstrapPlan {
        indexes,
        listener_event_types: listener_event_types.into_iter().collect(),
        ready: true,
    })
}

fn table_index(
    registry: &ProductionRuntimeTableRegistry,
    table_kind: &str,
) -> Result<BTreeMap<String, u32>, Vec<ProductionBootstrapBlock>> {
    let Some(table) = registry
        .tables
        .iter()
        .find(|table| table.table_kind == table_kind)
    else {
        return Err(vec![ProductionBootstrapBlock::MissingTable(
            table_kind.to_string(),
        )]);
    };
    Ok(table
        .mappings
        .iter()
        .map(|mapping| (mapping.canonical_id.clone(), mapping.ordinal))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_production_runtime_artifact, extract_production_chunk_graph,
        ExecutableProgramFingerprint, ProductionRootChunkInput, ResumeBoundaryId, ResumeBuildId,
        ResumeManifest, SharedChunkCandidatePlan,
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

    #[test]
    fn k10_builds_closed_indexes_only_after_artifact_table_validation() {
        let manifest = manifest();
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
        assert!(
            build_production_bootstrap_plan(&artifact, &manifest)
                .expect("plan")
                .ready
        );
        let mut malformed = artifact;
        malformed
            .tables
            .tables
            .retain(|table| table.table_kind != "events");
        assert_eq!(
            build_production_bootstrap_plan(&malformed, &manifest),
            Err(vec![ProductionBootstrapBlock::MissingTable(
                "events".to_string()
            )])
        );
    }
}
