//! K2 descriptive reachability over immutable Phase A-J executable products.

use std::collections::BTreeSet;

use crate::{
    ResumeManifest, RuntimeComponentArtifact, RuntimeComputedArtifact, RuntimeContextArtifact,
    RuntimeEffectArtifact, RuntimeFormsArtifact,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProductionReachabilityReason {
    ColdBoot,
    ResumeCapture,
    ResumeRestore,
    ActivationRoot,
    EventMapping,
    ArtifactProgram,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionExecutableRoot {
    pub subject_id: String,
    pub reason: ProductionReachabilityReason,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionProgramEdge {
    pub from: String,
    pub to: String,
    pub reason: ProductionReachabilityReason,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionUnreachableRecord {
    pub subject_id: String,
    pub reason: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionReachabilityBlock {
    pub subject_id: String,
    pub reason: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionReachabilityGraph {
    pub roots: Vec<ProductionExecutableRoot>,
    pub edges: Vec<ProductionProgramEdge>,
    pub reachable_programs: Vec<String>,
    pub unreachable: Vec<ProductionUnreachableRecord>,
    pub blocks: Vec<ProductionReachabilityBlock>,
}

/// Consumes only already-built artifacts. K2 describes closure; it never removes
/// a record or reads source syntax.
#[must_use]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn build_production_reachability_graph(
    resume: &ResumeManifest,
    component: &RuntimeComponentArtifact,
    computed: &RuntimeComputedArtifact,
    context: &RuntimeContextArtifact,
    effect: &RuntimeEffectArtifact,
    forms: &RuntimeFormsArtifact,
) -> ProductionReachabilityGraph {
    let mut roots = BTreeSet::new();
    let mut edges = BTreeSet::new();
    let mut programs = BTreeSet::new();
    let mut blocks = BTreeSet::new();
    let mut add = |from: String, to: String, reason: ProductionReachabilityReason| {
        if to.is_empty() {
            blocks.insert((from, "empty canonical program reference".to_string()));
        } else {
            programs.insert(to.clone());
            edges.insert((from, to, reason));
        }
    };
    roots.insert((
        "cold-boot".to_string(),
        ProductionReachabilityReason::ColdBoot,
    ));
    for chunk in &resume.chunks {
        let root = chunk.chunk_id.to_string();
        roots.insert((root.clone(), ProductionReachabilityReason::ActivationRoot));
        for program in &chunk.provided_program_ids {
            add(
                root.clone(),
                program.clone(),
                ProductionReachabilityReason::ActivationRoot,
            );
        }
    }
    for program in &resume.capture_programs {
        let root = program.program_id.to_string();
        roots.insert((root.clone(), ProductionReachabilityReason::ResumeCapture));
        add(
            root.clone(),
            root,
            ProductionReachabilityReason::ResumeCapture,
        );
    }
    for program in &resume.restore_programs {
        let root = program.program_id.to_string();
        roots.insert((root.clone(), ProductionReachabilityReason::ResumeRestore));
        add(
            root.clone(),
            root,
            ProductionReachabilityReason::ResumeRestore,
        );
    }
    for event in &resume.events {
        add(
            event.resume_event_id.to_string(),
            event.action_or_submit_program_id.clone(),
            ProductionReachabilityReason::EventMapping,
        );
    }
    for binding in &component.ordinary_template_bindings {
        add(
            "cold-boot".to_string(),
            binding.program_id.clone(),
            ProductionReachabilityReason::ColdBoot,
        );
    }
    for event in &component.ordinary_template_events {
        add(
            "cold-boot".to_string(),
            event.program_id.clone(),
            ProductionReachabilityReason::ColdBoot,
        );
    }
    for evaluation in &computed.evaluations {
        add(
            "cold-boot".to_string(),
            evaluation.evaluation_function.clone(),
            ProductionReachabilityReason::ArtifactProgram,
        );
    }
    for source in &context.sources {
        add(
            "cold-boot".to_string(),
            source.source_function.clone(),
            ProductionReachabilityReason::ArtifactProgram,
        );
    }
    for item in &effect.effects {
        add(
            "cold-boot".to_string(),
            item.execution_function.clone(),
            ProductionReachabilityReason::ArtifactProgram,
        );
    }
    for host in &forms.hosts {
        add(
            host.event.clone(),
            host.submit_action.clone(),
            ProductionReachabilityReason::EventMapping,
        );
    }
    for instance in &forms.instances {
        for program in &instance.programs.initialize {
            add(
                "cold-boot".to_string(),
                program.clone(),
                ProductionReachabilityReason::ColdBoot,
            );
        }
        for program in &instance.programs.reset {
            add(
                "cold-boot".to_string(),
                program.clone(),
                ProductionReachabilityReason::ArtifactProgram,
            );
        }
        for field in &instance.programs.input {
            for operation in &field.operations {
                add(
                    field.field.clone(),
                    operation.clone(),
                    ProductionReachabilityReason::ArtifactProgram,
                );
            }
        }
        for field in &instance.programs.blur {
            for operation in &field.operations {
                add(
                    field.field.clone(),
                    operation.clone(),
                    ProductionReachabilityReason::ArtifactProgram,
                );
            }
        }
    }
    ProductionReachabilityGraph {
        roots: roots
            .into_iter()
            .map(|(subject_id, reason)| ProductionExecutableRoot { subject_id, reason })
            .collect(),
        edges: edges
            .into_iter()
            .map(|(from, to, reason)| ProductionProgramEdge { from, to, reason })
            .collect(),
        reachable_programs: programs.into_iter().collect(),
        unreachable: Vec::new(),
        blocks: blocks
            .into_iter()
            .map(|(subject_id, reason)| ProductionReachabilityBlock { subject_id, reason })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_production_reachability_graph, ProductionReachabilityReason};
    use crate::resume_manifest::ResumeManifestChunkRootKind;
    use crate::runtime_component_artifact::SerializedDestructionMetadata;
    use crate::{
        ResumeBoundaryId, ResumeBuildId, ResumeManifest, ResumeManifestChunkRecord,
        RuntimeComponentArtifact, RuntimeComputedArtifact, RuntimeContextArtifact,
        RuntimeEffectArtifact, RuntimeFormsArtifact,
    };
    use std::str::FromStr;

    #[test]
    fn k2_reachability_keeps_canonical_chunk_programs_without_elimination() {
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
            chunks: vec![ResumeManifestChunkRecord {
                chunk_id: crate::ResumeChunkId::from_str("resume-chunk:Eager:application")
                    .expect("chunk"),
                root_kind: ResumeManifestChunkRootKind::Eager,
                root_id: "application".to_string(),
                module_path: "boot.js".to_string(),
                content_hash: "hash".to_string(),
                required_boundary_ids: Vec::new(),
                provided_program_ids: vec!["runtime-bootstrap".to_string()],
                dependency_chunk_ids: Vec::new(),
            }],
            activations: Vec::new(),
            anchors: Vec::new(),
            events: Vec::new(),
            phase_i_component_resume_records: Vec::new(),
            phase_i_form_resume_records: Vec::new(),
        };
        let graph = build_production_reachability_graph(
            &manifest,
            &RuntimeComponentArtifact {
                schema_version: 3,
                component_definitions: Vec::new(),
                instances: Vec::new(),
                initialization_batches: Vec::new(),
                slot_binding_programs: Vec::new(),
                instance_context_bindings: Vec::new(),
                ordinary_template_targets: Vec::new(),
                ordinary_template_bindings: Vec::new(),
                ordinary_template_events: Vec::new(),
                destruction: SerializedDestructionMetadata {
                    operation: "destroy".to_string(),
                    enabled: true,
                },
                structural_programs: Vec::new(),
            },
            &RuntimeComputedArtifact {
                schema_version: 3,
                state: Vec::new(),
                invalidations: Vec::new(),
                resource_invalidations: Vec::new(),
                evaluations: Vec::new(),
                evaluation_order: Vec::new(),
                update_batches: Vec::new(),
            },
            &RuntimeContextArtifact {
                schema_version: 2,
                sources: Vec::new(),
                consumers: Vec::new(),
                initial_batches: Vec::new(),
                action_updates: Vec::new(),
            },
            &RuntimeEffectArtifact {
                schema_version: 1,
                effects: Vec::new(),
            },
            &RuntimeFormsArtifact {
                schema_version: 1,
                registry_version: 1,
                forms: Vec::new(),
                instances: Vec::new(),
                hosts: Vec::new(),
            },
        );
        assert!(graph
            .reachable_programs
            .contains(&"runtime-bootstrap".to_string()));
        assert!(graph.unreachable.is_empty());
        assert!(graph
            .roots
            .iter()
            .any(|root| root.reason == ProductionReachabilityReason::ColdBoot));
    }
}
