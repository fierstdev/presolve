//! J5 deterministic resume chunk roots and canonical program closures.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_resume_activation_plan, build_resume_boundary_graph, ApplicationSemanticModel,
    ResumeActivationPolicy, ResumeActivationPrerequisite, ResumeActivationRootKind,
    ResumeBoundaryActivationProgram, ResumeBoundaryId, ResumeChunkId, ResumeExistingSlot,
    SemanticId, SourceProvenance,
};

pub const RESUME_CHUNK_GRAPH_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeChunkRootKind {
    Eager,
    Interaction,
    Visible,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeChunkProgram {
    RuntimeBootstrap,
    RuntimeRegistries,
    EventDelegation,
    PostRestoreRecomputation(ResumeExistingSlot),
    ImmediateFormRuntime(crate::FormInstanceId),
    OrdinaryEvent {
        event: SemanticId,
        handler: SemanticId,
        action_batch: SemanticId,
        program: SemanticId,
    },
    FormSubmit {
        submission_host: crate::SubmissionHostId,
        submission_plan: crate::SubmissionPlanId,
        submit_action: SemanticId,
        action_batch: SemanticId,
        serialization_plan: crate::SerializationPlanId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeChunkProgramInclusion {
    pub chunk_id: ResumeChunkId,
    pub program: ResumeChunkProgram,
    pub required_boundaries: Vec<ResumeBoundaryId>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeChunkModulePlan {
    pub module_path_stem: String,
    pub canonical_module_bytes: String,
    pub content_hash: String,
    pub module_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeChunk {
    pub id: ResumeChunkId,
    pub root_kind: ResumeChunkRootKind,
    pub root_boundary: Option<ResumeBoundaryId>,
    pub required_boundaries: Vec<ResumeBoundaryId>,
    pub programs: Vec<ResumeChunkProgram>,
    pub dependency_chunks: Vec<ResumeChunkId>,
    pub module: ResumeChunkModulePlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeChunkBlockReason {
    MissingActivationBoundary,
    MissingProgram,
    UnsupportedActivationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeChunkBlock {
    pub root_boundary: Option<ResumeBoundaryId>,
    pub reason: ResumeChunkBlockReason,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeChunkGraph {
    pub version: u32,
    pub eager_chunk: ResumeChunkId,
    pub chunks: Vec<ResumeChunk>,
    pub inclusions: Vec<ResumeChunkProgramInclusion>,
    pub blocks: Vec<ResumeChunkBlock>,
    pub chunk_index: BTreeMap<ResumeChunkId, usize>,
}

impl ResumeChunkGraph {
    #[must_use]
    pub fn chunk(&self, id: &ResumeChunkId) -> Option<&ResumeChunk> {
        self.chunk_index
            .get(id)
            .and_then(|index| self.chunks.get(*index))
    }

    #[must_use]
    pub fn chunk_for_root(&self, boundary: &ResumeBoundaryId) -> Option<&ResumeChunk> {
        self.chunks
            .iter()
            .find(|chunk| chunk.root_boundary.as_ref() == Some(boundary))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeChunkIntegrityCode {
    DuplicateInclusion,
    MissingProgram,
    DependencyCycle,
    RootCorrespondence,
    UnrelatedProgram,
    OrderingOrOutputDrift,
}

impl ResumeChunkIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateInclusion => "EZASM1343",
            Self::MissingProgram => "EZASM1344",
            Self::DependencyCycle => "EZASM1345",
            Self::RootCorrespondence => "EZASM1346",
            Self::UnrelatedProgram => "EZASM1347",
            Self::OrderingOrOutputDrift => "EZASM1348",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeChunkIntegrityDiagnostic {
    pub code: ResumeChunkIntegrityCode,
    pub chunk: Option<ResumeChunkId>,
    pub message: String,
}

#[must_use]
#[allow(clippy::too_many_lines)]
/// # Panics
///
/// Panics only when the canonical J3 graph lacks its required application
/// root boundary, which indicates an earlier compiler integrity failure.
pub fn build_resume_chunk_graph(model: &ApplicationSemanticModel) -> ResumeChunkGraph {
    let activation = build_resume_activation_plan(model);
    let boundaries = build_resume_boundary_graph(model);
    let eager_id =
        ResumeChunkId::for_activation_root(ResumeActivationRootKind::Eager, "application");
    let eager_provenance = boundaries
        .boundaries
        .iter()
        .find(|boundary| boundary.kind == crate::ResumeBoundaryKind::ApplicationRoot)
        .map(|boundary| boundary.provenance.clone())
        .expect("canonical application root boundary");
    let mut eager_programs = vec![
        ResumeChunkProgram::RuntimeBootstrap,
        ResumeChunkProgram::RuntimeRegistries,
        ResumeChunkProgram::EventDelegation,
    ];
    let mut eager_boundaries = Vec::new();
    for decision in activation
        .decisions
        .iter()
        .filter(|decision| decision.policy == ResumeActivationPolicy::Eager)
    {
        eager_boundaries.push(decision.boundary.clone());
        for prerequisite in &decision.prerequisites {
            match prerequisite {
                ResumeActivationPrerequisite::PostRestoreRecomputation(slot)
                | ResumeActivationPrerequisite::RecomputableSlot(slot) => {
                    eager_programs.push(ResumeChunkProgram::PostRestoreRecomputation(slot.clone()));
                }
                ResumeActivationPrerequisite::ImmediateFormRuntime(form) => {
                    eager_programs.push(ResumeChunkProgram::ImmediateFormRuntime(form.clone()));
                }
                ResumeActivationPrerequisite::ApplicationBootstrap
                | ResumeActivationPrerequisite::RuntimeRegistryInstallation
                | ResumeActivationPrerequisite::EventDelegationInstallation
                | ResumeActivationPrerequisite::ExactInteraction(_)
                | ResumeActivationPrerequisite::RequiredBoundary(_)
                | ResumeActivationPrerequisite::RetainedSlot(_) => {}
            }
        }
    }
    eager_boundaries.sort();
    eager_boundaries.dedup();
    eager_programs.sort();
    eager_programs.dedup();
    let mut chunks = vec![chunk(
        eager_id.clone(),
        ResumeChunkRootKind::Eager,
        None,
        eager_boundaries.clone(),
        eager_programs.clone(),
    )];
    let mut inclusions = eager_programs
        .iter()
        .cloned()
        .map(|program| ResumeChunkProgramInclusion {
            chunk_id: eager_id.clone(),
            program,
            required_boundaries: eager_boundaries.clone(),
            provenance: eager_provenance.clone(),
        })
        .collect::<Vec<_>>();
    let mut blocks = Vec::new();

    for decision in activation.decisions.iter().filter(|decision| {
        matches!(
            decision.policy,
            ResumeActivationPolicy::Interaction
                | ResumeActivationPolicy::Visible
                | ResumeActivationPolicy::Manual
        )
    }) {
        let Some(reference) = boundaries
            .activation_references
            .iter()
            .find(|reference| reference.interaction_boundary == decision.boundary)
        else {
            blocks.push(ResumeChunkBlock {
                root_boundary: Some(decision.boundary.clone()),
                reason: ResumeChunkBlockReason::MissingActivationBoundary,
                provenance: decision.provenance.clone(),
            });
            continue;
        };
        let (root_kind, activation_kind) = match decision.policy {
            ResumeActivationPolicy::Interaction => (
                ResumeChunkRootKind::Interaction,
                ResumeActivationRootKind::Event,
            ),
            ResumeActivationPolicy::Visible => (
                ResumeChunkRootKind::Visible,
                ResumeActivationRootKind::Visible,
            ),
            ResumeActivationPolicy::Manual => (
                ResumeChunkRootKind::Manual,
                ResumeActivationRootKind::Manual,
            ),
            ResumeActivationPolicy::Eager | ResumeActivationPolicy::None => unreachable!(),
        };
        let id = ResumeChunkId::for_activation_root(activation_kind, decision.boundary.as_str());
        let program = match &reference.program {
            ResumeBoundaryActivationProgram::OrdinaryEvent {
                declaration_event,
                handler_method,
                action_batch,
                existing_program,
                ..
            } => ResumeChunkProgram::OrdinaryEvent {
                event: declaration_event.clone(),
                handler: handler_method.clone(),
                action_batch: action_batch.clone(),
                program: existing_program.clone(),
            },
            ResumeBoundaryActivationProgram::FormSubmit {
                submission_host,
                submission_plan,
                submit_action,
                action_batch,
                serialization_plan,
                ..
            } => ResumeChunkProgram::FormSubmit {
                submission_host: submission_host.clone(),
                submission_plan: submission_plan.clone(),
                submit_action: submit_action.clone(),
                action_batch: action_batch.clone(),
                serialization_plan: serialization_plan.clone(),
            },
        };
        let mut required_boundaries = reference.required_boundaries.clone();
        required_boundaries.sort();
        required_boundaries.dedup();
        chunks.push(chunk(
            id.clone(),
            root_kind,
            Some(decision.boundary.clone()),
            required_boundaries.clone(),
            vec![program.clone()],
        ));
        inclusions.push(ResumeChunkProgramInclusion {
            chunk_id: id,
            program,
            required_boundaries,
            provenance: reference.provenance.clone(),
        });
    }

    chunks.sort_by(|left, right| {
        (
            left.root_kind != ResumeChunkRootKind::Eager,
            &left.root_boundary,
            &left.id,
        )
            .cmp(&(
                right.root_kind != ResumeChunkRootKind::Eager,
                &right.root_boundary,
                &right.id,
            ))
    });
    inclusions.sort_by(|left, right| {
        (&left.chunk_id, &left.program).cmp(&(&right.chunk_id, &right.program))
    });
    blocks.sort_by(|left, right| {
        (&left.root_boundary, left.reason).cmp(&(&right.root_boundary, right.reason))
    });
    let chunk_index = chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| (chunk.id.clone(), index))
        .collect();
    ResumeChunkGraph {
        version: RESUME_CHUNK_GRAPH_VERSION,
        eager_chunk: eager_id,
        chunks,
        inclusions,
        blocks,
        chunk_index,
    }
}

fn chunk(
    id: ResumeChunkId,
    root_kind: ResumeChunkRootKind,
    root_boundary: Option<ResumeBoundaryId>,
    required_boundaries: Vec<ResumeBoundaryId>,
    programs: Vec<ResumeChunkProgram>,
) -> ResumeChunk {
    let kind = match root_kind {
        ResumeChunkRootKind::Eager => "boot",
        ResumeChunkRootKind::Interaction => "event",
        ResumeChunkRootKind::Visible => "visible",
        ResumeChunkRootKind::Manual => "manual",
    };
    let short = safe_stem(
        root_boundary
            .as_ref()
            .map_or("application", ResumeBoundaryId::as_str),
    );
    let canonical_module_bytes = format!(
        "// chunk={id}\n// kind={root_kind:?}\n// root={}\n// boundaries={}\n// programs={}\nexport {{}};\n",
        root_boundary.as_ref().map_or("", ResumeBoundaryId::as_str),
        required_boundaries
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        programs
            .iter()
            .map(|program| format!("{program:?}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let build_hash = crate::ResumeBuildId::for_public_inputs(&canonical_module_bytes);
    let content_hash = build_hash
        .as_str()
        .strip_prefix("resume-build:")
        .unwrap_or(build_hash.as_str())
        .to_string();
    let module_path_stem = format!("{kind}.{short}");
    let module_path = format!("{module_path_stem}.{content_hash}.js");
    ResumeChunk {
        id,
        root_kind,
        root_boundary,
        required_boundaries,
        programs,
        dependency_chunks: Vec::new(),
        module: ResumeChunkModulePlan {
            module_path_stem,
            canonical_module_bytes,
            content_hash,
            module_path,
        },
    }
}

fn safe_stem(value: &str) -> String {
    let mut stem = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    while stem.contains("--") {
        stem = stem.replace("--", "-");
    }
    stem.trim_matches('-').chars().take(48).collect()
}

#[must_use]
pub fn validate_resume_chunk_graph(
    model: &ApplicationSemanticModel,
    graph: &ResumeChunkGraph,
) -> Vec<ResumeChunkIntegrityDiagnostic> {
    let canonical = build_resume_chunk_graph(model);
    let activation = build_resume_activation_plan(model);
    let mut diagnostics = Vec::new();
    let mut inclusions = BTreeSet::new();
    for inclusion in &graph.inclusions {
        if !inclusions.insert((inclusion.chunk_id.clone(), inclusion.program.clone())) {
            diagnostics.push(integrity(
                ResumeChunkIntegrityCode::DuplicateInclusion,
                Some(inclusion.chunk_id.clone()),
                "generated program was included more than once in one chunk",
            ));
        }
        if graph.chunk(&inclusion.chunk_id).is_none() {
            diagnostics.push(integrity(
                ResumeChunkIntegrityCode::MissingProgram,
                Some(inclusion.chunk_id.clone()),
                "program inclusion references an unknown chunk",
            ));
        }
    }
    for chunk in &graph.chunks {
        if !chunk.dependency_chunks.is_empty() {
            diagnostics.push(integrity(
                ResumeChunkIntegrityCode::DependencyCycle,
                Some(chunk.id.clone()),
                "Phase J v1 root chunks cannot depend on other lazy chunks",
            ));
        }
        if chunk.programs.iter().any(|program| {
            !graph
                .inclusions
                .iter()
                .any(|inclusion| inclusion.chunk_id == chunk.id && inclusion.program == *program)
        }) {
            diagnostics.push(integrity(
                ResumeChunkIntegrityCode::MissingProgram,
                Some(chunk.id.clone()),
                "chunk program has no reciprocal inclusion record",
            ));
        }
    }
    for decision in activation.decisions.iter().filter(|decision| {
        decision.policy != ResumeActivationPolicy::None
            && decision.policy != ResumeActivationPolicy::Eager
    }) {
        if graph.chunk_for_root(&decision.boundary).is_none() {
            diagnostics.push(integrity(
                ResumeChunkIntegrityCode::RootCorrespondence,
                None,
                "non-eager activation has no exact lazy root chunk",
            ));
        }
    }
    if graph.version != RESUME_CHUNK_GRAPH_VERSION || graph != &canonical {
        diagnostics.push(integrity(
            ResumeChunkIntegrityCode::OrderingOrOutputDrift,
            None,
            "chunk graph drifted from canonical roots, closure, module bytes, or order",
        ));
    }
    diagnostics.sort_by(|left, right| {
        (left.code, &left.chunk, left.message.as_str()).cmp(&(
            right.code,
            &right.chunk,
            right.message.as_str(),
        ))
    });
    diagnostics.dedup();
    diagnostics
}

fn integrity(
    code: ResumeChunkIntegrityCode,
    chunk: Option<ResumeChunkId>,
    message: &str,
) -> ResumeChunkIntegrityDiagnostic {
    ResumeChunkIntegrityDiagnostic {
        code,
        chunk,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_one_eager_and_one_isolated_chunk_per_interaction() {
        let model = crate::build_application_semantic_model(&ezc_parser::parse_file(
            "src/Chunks.tsx",
            r#"
@component("x-chunk-child") class Child {
  a = state(1); b = state(2);
  @action() first() { this.a++; }
  @action() second() { this.b++; }
  render() { return <><button onClick={() => this.first()}>A</button><button onClick={() => this.second()}>B</button></>; }
}
@component("x-chunk-page") @route("/") class Page { render() { return <Child />; } }
"#,
        ));
        let graph = build_resume_chunk_graph(&model);
        assert!(validate_resume_chunk_graph(&model, &graph).is_empty());
        assert_eq!(graph.chunks.len(), 3);
        assert_eq!(
            graph
                .chunks
                .iter()
                .filter(|chunk| chunk.root_kind == ResumeChunkRootKind::Eager)
                .count(),
            1
        );
        let lazy = graph
            .chunks
            .iter()
            .filter(|chunk| chunk.root_kind == ResumeChunkRootKind::Interaction)
            .collect::<Vec<_>>();
        assert_eq!(lazy.len(), 2);
        assert!(lazy.iter().all(|chunk| {
            chunk.programs.len() == 1
                && chunk.dependency_chunks.is_empty()
                && chunk.module.module_path.starts_with("event.")
        }));
        assert_ne!(lazy[0].programs, lazy[1].programs);
    }

    #[test]
    fn chunk_output_is_deterministic_under_source_reversal() {
        let first = ezc_parser::parse_file(
            "src/A.tsx",
            r#"@component("x-a") @route("/a") class A { @action() go() {} render() { return <button onClick={() => this.go()}>A</button>; } }"#,
        );
        let second = ezc_parser::parse_file(
            "src/B.tsx",
            r#"@component("x-b") @route("/b") class B { @action() go() {} render() { return <button onClick={() => this.go()}>B</button>; } }"#,
        );
        let forward = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![first.clone(), second.clone()]),
        );
        let reverse = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![second, first]),
        );
        assert_eq!(
            build_resume_chunk_graph(&forward),
            build_resume_chunk_graph(&reverse)
        );
    }

    #[test]
    fn reserves_the_complete_j5_integrity_range() {
        assert_eq!(
            [
                ResumeChunkIntegrityCode::DuplicateInclusion,
                ResumeChunkIntegrityCode::MissingProgram,
                ResumeChunkIntegrityCode::DependencyCycle,
                ResumeChunkIntegrityCode::RootCorrespondence,
                ResumeChunkIntegrityCode::UnrelatedProgram,
                ResumeChunkIntegrityCode::OrderingOrOutputDrift,
            ]
            .map(ResumeChunkIntegrityCode::code),
            [
                "EZASM1343",
                "EZASM1344",
                "EZASM1345",
                "EZASM1346",
                "EZASM1347",
                "EZASM1348",
            ]
        );
    }
}
