//! L9-C complete-request adapter for the frozen L4 compiler service.
//!
//! Callers own source acquisition. This module receives exact complete source
//! inputs, derives the canonical candidate snapshot, and delegates once to
//! L4. It does not read paths, parse source, or recreate compiler products.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::path::Path;

use presolve_compiler::persistent_cache::CacheReportSelector;
use presolve_compiler::platform::{
    derive_workspace_id_v1, ContractVersion, RequestedCompilationMode, WorkspaceConfiguration,
    WorkspaceInput, WorkspaceSnapshot, WorkspaceSource,
};
use presolve_compiler::service::{
    CompileRequest, CompilerServiceHost, CompleteSource, IncrementalReportSelector, ServiceError,
};

/// One exact caller-provided source input. Its path is an L3 logical path,
/// rather than an authority to read a host filesystem path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSourceInputV1 {
    pub logical_path: String,
    pub content: String,
}

/// Complete L9-C project candidate. There is deliberately no path loader in
/// this API: the later command layer supplies already-authorized inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCompilationCandidateV1 {
    pub configuration: WorkspaceConfiguration,
    pub sources: Vec<CliSourceInputV1>,
    pub verify_clean_equivalence: bool,
    pub report: IncrementalReportSelector,
    pub cache_report: CacheReportSelector,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCompilationResultV1 {
    pub workspace_id: String,
    pub commit_sequence: u64,
    pub snapshot_id: String,
    pub graph_snapshot_id: String,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCompilationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

fn compiler_contract() -> ContractVersion {
    ContractVersion::new(format!("presolve-compiler:{}", env!("CARGO_PKG_VERSION")))
}

fn service_error(error: ServiceError) -> CliCompilationErrorV1 {
    CliCompilationErrorV1 {
        code: error.code,
        message: error.message,
    }
}

fn candidate_snapshot(
    candidate: &CliCompilationCandidateV1,
    contract: &ContractVersion,
) -> Result<WorkspaceSnapshot, CliCompilationErrorV1> {
    WorkspaceSnapshot::from_input(&WorkspaceInput {
        configuration: candidate.configuration.clone(),
        sources: candidate
            .sources
            .iter()
            .map(|source| WorkspaceSource {
                path: source.logical_path.clone(),
                source: source.content.clone(),
                language: None,
            })
            .collect(),
        compiler_contract: contract.clone(),
    })
    .map_err(|error| CliCompilationErrorV1 {
        code: "L9C201_INVALID_COMPLETE_CANDIDATE",
        message: error.message,
    })
}

/// Delegates a complete candidate to a fresh local L4 service lifecycle.
///
/// This is intentionally the only L9-C compilation path. It never calls the
/// legacy CLI parser or code-generation helpers.
pub fn compile_complete_candidate_v1(
    service_root: &Path,
    candidate: CliCompilationCandidateV1,
) -> Result<CliCompilationResultV1, CliCompilationErrorV1> {
    let contract = compiler_contract();
    let workspace_id = derive_workspace_id_v1(&candidate.configuration).map_err(|error| {
        CliCompilationErrorV1 {
            code: "L9C202_INVALID_CONFIGURATION",
            message: error.message,
        }
    })?;
    let snapshot = candidate_snapshot(&candidate, &contract)?;
    let mut service = CompilerServiceHost::start(service_root, contract).map_err(service_error)?;
    let session_id = service
        .open_session(candidate.configuration.clone(), &workspace_id)
        .map_err(service_error)?;
    let response = service
        .compile(
            &session_id,
            0,
            CompileRequest {
                configuration: candidate.configuration,
                candidate_snapshot: snapshot,
                sources: candidate
                    .sources
                    .into_iter()
                    .map(|source| CompleteSource {
                        path: source.logical_path,
                        source: source.content,
                        language: None,
                    })
                    .collect(),
                mode: RequestedCompilationMode::Automatic,
                incremental_report: candidate.report,
                verify_exact_equivalence: candidate.verify_clean_equivalence,
                cache_report: candidate.cache_report,
            },
        )
        .map_err(service_error)?;
    service.close_session(&session_id).map_err(service_error)?;
    Ok(CliCompilationResultV1 {
        workspace_id: workspace_id.to_string(),
        commit_sequence: response.commit_sequence,
        snapshot_id: response.snapshot.snapshot_id.to_string(),
        graph_snapshot_id: response.graph.snapshot_id.to_string(),
        mode: response.mode,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

    fn candidate() -> CliCompilationCandidateV1 {
        CliCompilationCandidateV1 {
            configuration: WorkspaceConfiguration::default(),
            sources: vec![CliSourceInputV1 {
                logical_path: "src/main.ts".into(),
                content: "export const L9_C_SOURCE_SENTINEL = 1;\n".into(),
            }],
            verify_clean_equivalence: true,
            report: IncrementalReportSelector::Summary,
            cache_report: CacheReportSelector::Summary,
        }
    }

    #[test]
    fn l9c_delegates_complete_caller_owned_source_input_to_l4() {
        let root = std::env::temp_dir().join(format!(
            "presolve-l9c-{}",
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        let result = compile_complete_candidate_v1(&root, candidate()).unwrap();
        assert_eq!(result.commit_sequence, 1);
        assert!(result.snapshot_id.starts_with("snapshot:"));
        assert_eq!(result.snapshot_id, result.graph_snapshot_id);
        let session = std::fs::read_dir(root.join("service").join("sessions"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("session.json");
        let durable = std::fs::read_to_string(session).unwrap();
        assert!(!durable.contains("L9_C_SOURCE_SENTINEL"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
