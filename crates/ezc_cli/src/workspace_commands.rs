//! L9-F explicit single-project workspace requests over the L7 service API.

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use ezc_core::persistent_cache::CacheReportSelector;
use ezc_core::platform::{
    derive_workspace_id_v1, ContractVersion, RequestedCompilationMode, WorkspaceInput,
    WorkspaceSnapshot, WorkspaceSource,
};
use ezc_core::service::{
    CompileRequest, CompilerServiceHost, CompleteSource, IncrementalReportSelector,
    WorkspaceCompileRequestV1, WorkspacePackageCompileRequestV1,
};
use ezc_core::workspace::{
    WorkspaceManifestV1, WorkspacePackageDescriptorV1, WorkspacePolicyV1,
    WORKSPACE_MANIFEST_V1_SCHEMA,
};

use crate::{
    load_explicit_project_envelope_v1, load_explicit_source_inputs_v1, CliExplicitSourceSpecV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliWorkspaceResultV1 {
    pub workspace_id: String,
    pub status: String,
    pub manifest_identity: String,
    pub graph_identity: String,
    pub plan_identity: String,
    pub package_snapshot_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliWorkspaceErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for CliWorkspaceErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CliWorkspaceErrorV1 {}

fn compiler_contract() -> ContractVersion {
    ContractVersion::new(format!("presolve-compiler:{}", env!("CARGO_PKG_VERSION")))
}

/// Executes one complete, explicitly supplied project as a one-package L7
/// workspace. Package discovery is intentionally not part of this command.
pub fn run_explicit_workspace_v1(
    configuration_path: &Path,
    source_specs: &[CliExplicitSourceSpecV1],
    verify_clean_equivalence: bool,
) -> Result<CliWorkspaceResultV1, CliWorkspaceErrorV1> {
    let project_root = configuration_path.parent().ok_or(CliWorkspaceErrorV1 {
        code: "L9F001_CONFIGURATION_PATH_INVALID",
        message: "configuration path must have a parent directory".into(),
    })?;
    let envelope =
        load_explicit_project_envelope_v1(project_root, configuration_path).map_err(|error| {
            CliWorkspaceErrorV1 {
                code: error.code,
                message: error.message,
            }
        })?;
    let sources =
        load_explicit_source_inputs_v1(&envelope.project_root, source_specs).map_err(|error| {
            CliWorkspaceErrorV1 {
                code: error.code,
                message: error.message,
            }
        })?;
    let contract = compiler_contract();
    let workspace_id =
        derive_workspace_id_v1(&envelope.configuration).map_err(|error| CliWorkspaceErrorV1 {
            code: "L9F002_INVALID_CONFIGURATION",
            message: error.message,
        })?;
    let snapshot = WorkspaceSnapshot::from_input(&WorkspaceInput {
        configuration: envelope.configuration.clone(),
        sources: sources
            .iter()
            .map(|source| WorkspaceSource {
                path: source.logical_path.clone(),
                source: source.content.clone(),
                language: None,
            })
            .collect(),
        compiler_contract: contract.clone(),
    })
    .map_err(|error| CliWorkspaceErrorV1 {
        code: "L9F003_INVALID_COMPLETE_CANDIDATE",
        message: error.message,
    })?;
    let mut service = CompilerServiceHost::start(envelope.project_root.join(".presolve"), contract)
        .map_err(|error| CliWorkspaceErrorV1 {
            code: error.code,
            message: error.message,
        })?;
    let session_id = service
        .open_session(envelope.configuration.clone(), &workspace_id)
        .map_err(|error| CliWorkspaceErrorV1 {
            code: error.code,
            message: error.message,
        })?;
    let manifest = WorkspaceManifestV1 {
        schema: WORKSPACE_MANIFEST_V1_SCHEMA.into(),
        version: 1,
        workspace_id: workspace_id.to_string(),
        packages: vec![WorkspacePackageDescriptorV1 {
            package_id: "project".into(),
            session_id: session_id.clone(),
            display_name: Some("project".into()),
            configuration_identity_hint: None,
            metadata: BTreeMap::new(),
        }],
        dependencies: Vec::new(),
        policy: WorkspacePolicyV1 {
            failure_mode: "fail_fast".into(),
            execution_mode: "deterministic_serial".into(),
            result_detail: "summary".into(),
        },
    };
    let request = CompileRequest {
        configuration: envelope.configuration,
        candidate_snapshot: snapshot,
        sources: sources
            .into_iter()
            .map(|source| CompleteSource {
                path: source.logical_path,
                source: source.content,
                language: None,
            })
            .collect(),
        mode: RequestedCompilationMode::Automatic,
        incremental_report: IncrementalReportSelector::Summary,
        verify_exact_equivalence: verify_clean_equivalence,
        cache_report: CacheReportSelector::Summary,
    };
    let result = service
        .compile_workspace_v1(WorkspaceCompileRequestV1 {
            manifest,
            packages: vec![WorkspacePackageCompileRequestV1 {
                package_id: "project".into(),
                expected_commit_sequence: 0,
                request,
            }],
            operation_id: "presolve-cli-workspace-v1".into(),
        })
        .map_err(|error| CliWorkspaceErrorV1 {
            code: error.code,
            message: error.message,
        })?;
    service
        .verify_workspace(&result.workspace_id)
        .map_err(|error| CliWorkspaceErrorV1 {
            code: error.code,
            message: error.message,
        })?;
    service
        .close_session(&session_id)
        .map_err(|error| CliWorkspaceErrorV1 {
            code: error.code,
            message: error.message,
        })?;
    Ok(CliWorkspaceResultV1 {
        workspace_id: result.workspace_id,
        status: result.status,
        manifest_identity: result.manifest_identity,
        graph_identity: result.graph_identity,
        plan_identity: result.plan_identity,
        package_snapshot_id: result
            .package_results
            .into_iter()
            .next()
            .and_then(|item| item.snapshot_id),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::parse_explicit_source_spec_v1;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn l9f_delegates_one_explicit_package_to_l7_and_verifies_the_result() {
        let root = std::env::temp_dir().join(format!(
            "presolve-l9f-{}",
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        let config = root.join("presolve.json");
        fs::write(
            &config,
            include_bytes!("../fixtures/configuration/minimum-cli-v1.json"),
        )
        .unwrap();
        fs::write(root.join("src/main.ts"), "export const value = 1;\n").unwrap();
        let result = run_explicit_workspace_v1(
            &config,
            &[parse_explicit_source_spec_v1("src/main.ts=src/main.ts").unwrap()],
            true,
        )
        .unwrap();
        assert_eq!(result.status, "succeeded");
        assert!(result.package_snapshot_id.unwrap().starts_with("snapshot:"));
        fs::remove_dir_all(root).unwrap();
    }
}
