use std::fs;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use presolve_compiler::platform::{
    derive_workspace_id_v1, CacheLimits, CancellationToken, CompilationOutcome,
    CompileWorkspaceRequest, CompilerSessionState, RequestedCompilationMode, WorkspaceInput,
    WorkspaceSource,
};
use presolve_compiler::{
    build_production_runtime_artifact, build_tooling_artifact_graph_v1,
    build_tooling_build_trace_v1, build_tooling_compile_cost_report_v1,
    extract_production_chunk_graph, tooling_artifact_graph_json_v1, tooling_build_trace_json_v1,
    tooling_compile_cost_report_json_v1, ExecutableProgramFingerprint, OptimizationPolicyId,
    OptimizationReportV1, ProductionRootChunkInput, ResumeBoundaryId, ResumeBuildId,
    ResumeManifest, RuntimeCostReportV1, SharedChunkCandidatePlan, ToolingBuildTraceStageV1,
    ToolingTraceIdentityV1, ToolingTraceOutcomeV1, ToolingTraceStageKindV1,
};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

fn project() -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "presolve-l9d-cli-{}",
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
    (root, config)
}

fn tooling_products(root: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let workspace = WorkspaceInput::new(vec![WorkspaceSource {
        path: "src/ToolingCliFixture.ts".into(),
        source: "export const toolingCliFixture = 1;\n".into(),
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
        panic!("tooling CLI fixture compilation must commit");
    };
    let snapshot = root.join("workspace-snapshot.json");
    let graph = root.join("workspace-graph.json");
    fs::write(&snapshot, result.snapshot.to_canonical_json().unwrap()).unwrap();
    fs::write(&graph, result.graph.to_canonical_json().unwrap()).unwrap();
    (snapshot, graph)
}

#[test]
fn l9d_explicit_check_emits_the_stable_json_result_envelope() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(&root)
        .args([
            "check",
            "--config",
            config.to_str().unwrap(),
            "--source",
            "src/main.ts=src/main.ts",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["schema"], "presolve.cli-result");
    assert_eq!(result["version"], 1);
    assert_eq!(result["command"], "check");
    assert_eq!(result["status"], "succeeded");
    assert_eq!(result["exit_code"], 0);
    assert!(result["result"]["snapshot_id"]
        .as_str()
        .unwrap()
        .starts_with("snapshot:"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l9d_explicit_build_reports_configuration_errors_on_stderr() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(&root)
        .args(["build", "--config", config.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("L9D003_MISSING_SOURCE_INPUT"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l9e_cache_inspection_requires_an_initialized_persistent_cache() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(&root)
        .args([
            "cache",
            "inspect",
            "--config",
            config.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("L6C023"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l9f_workspace_executes_the_explicit_project_through_l7() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(&root)
        .args([
            "workspace",
            "--config",
            config.to_str().unwrap(),
            "--source",
            "src/main.ts=src/main.ts",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["schema"], "presolve.cli-workspace-result");
    assert_eq!(result["status"], "succeeded");
    assert!(result["plan_identity"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l9g_version_help_and_reserved_command_exit_contracts_are_stable() {
    let binary = env!("CARGO_BIN_EXE_presolve");
    let version = Command::new(binary)
        .args(["version", "--format", "json"])
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&version.stdout).unwrap()["schema"],
        "presolve.cli-version"
    );
    let help = Command::new(binary).arg("help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("workspace"));
    let reserved = Command::new(binary).arg("create").output().unwrap();
    assert_eq!(reserved.status.code(), Some(6));
    assert!(reserved.stdout.is_empty());
    assert!(String::from_utf8_lossy(&reserved.stderr).contains("reserved"));
}

#[test]
fn l9_watch_once_submits_the_complete_candidate_to_l8() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(&root)
        .args([
            "watch",
            "--once",
            "--config",
            config.to_str().unwrap(),
            "--source",
            "src/main.ts=src/main.ts",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "presolve.cli-watch-once");
    assert_eq!(value["outcome"], "succeeded");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l11c_projects_only_validated_workspace_products() {
    let (root, _) = project();
    let (snapshot, graph) = tooling_products(&root);
    let binary = env!("CARGO_BIN_EXE_presolve");

    let inspect = Command::new(binary)
        .current_dir(&root)
        .args([
            "inspect",
            "workspace-snapshot",
            "--schema",
            "presolve.workspace-snapshot",
            "--product",
            snapshot.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(inspect.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&inspect.stdout).unwrap()["schema_version"],
        1
    );

    let dot = Command::new(binary)
        .current_dir(&root)
        .args([
            "graph",
            "workspace",
            "--schema",
            "presolve.workspace-graph",
            "--product",
            graph.to_str().unwrap(),
            "--format",
            "dot",
        ])
        .output()
        .unwrap();
    assert!(
        dot.status.success(),
        "workspace graph stderr: {}",
        String::from_utf8_lossy(&dot.stderr)
    );
    assert!(
        String::from_utf8_lossy(&dot.stdout).starts_with("digraph \"presolve.workspace-graph\"")
    );

    let mismatch = Command::new(binary)
        .current_dir(&root)
        .args([
            "inspect",
            "workspace-graph",
            "--schema",
            "presolve.workspace-snapshot",
            "--product",
            snapshot.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(mismatch.status.code(), Some(6));
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("L11T006"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l11g_trace_projects_only_a_validated_explicit_product() {
    let (root, _) = project();
    let trace = build_tooling_build_trace_v1(
        "workspace:cli-fixture".into(),
        "compiler-contract:v1".into(),
        None,
        ToolingTraceOutcomeV1::Succeeded,
        vec![ToolingBuildTraceStageV1 {
            ordinal: 0,
            kind: ToolingTraceStageKindV1::L3Snapshot,
            outcome: ToolingTraceOutcomeV1::Succeeded,
            identities: vec![ToolingTraceIdentityV1 {
                name: "snapshot_id".into(),
                value: "snapshot:cli-fixture".into(),
            }],
        }],
    )
    .unwrap();
    let product = root.join("build-trace.json");
    fs::write(&product, tooling_build_trace_json_v1(&trace)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "trace",
            "--schema",
            "presolve.build-trace",
            "--product",
            product.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["traceId"],
        trace.trace_id
    );
    let mismatch = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "trace",
            "--schema",
            "presolve.workspace-graph",
            "--product",
            product.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(mismatch.status.code(), Some(6));
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("L11T003"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l11g_profile_projects_only_a_validated_explicit_product() {
    let (root, _) = project();
    let binary = env!("CARGO_BIN_EXE_presolve");
    let build_id = ResumeBuildId::zero_sentinel();
    let optimization = OptimizationReportV1 {
        schema_version: 1,
        build_id: build_id.clone(),
        optimization_policy: OptimizationPolicyId::production_v1(),
        dead_products_removed: 0,
        constants_pooled: 0,
        programs_deduplicated: 0,
        shared_chunks_extracted: 0,
        shared_candidates_rejected: 0,
        binding_writes_coalesced: 0,
        runtime_table_count: 0,
        development_bytes: 100,
        production_bytes: 80,
        retained_exclusions: vec!["wall-clock-timing".into()],
        validation_status: "valid".into(),
    };
    let cost = RuntimeCostReportV1 {
        schema_version: 1,
        build_id,
        bootstrap_module_bytes: 0,
        production_artifact_bytes: 80,
        eager_program_count: 0,
        lazy_root_chunk_count: 0,
        shared_chunk_count: 0,
        max_lazy_dependency_depth: 0,
        runtime_table_count: 0,
        runtime_record_count: 0,
        estimated_boot_decode_units: 0,
        estimated_boot_validation_units: 0,
        estimated_cold_init_operation_count: 0,
        estimated_resume_restore_operation_count: 0,
        max_action_batch_operation_count: 0,
        max_scheduler_batch_width: 0,
        max_dom_patch_count_per_action: 0,
        retained_slot_count: 0,
    };
    let report = build_tooling_compile_cost_report_v1(optimization, cost).unwrap();
    let product = root.join("compile-cost-report.json");
    fs::write(&product, tooling_compile_cost_report_json_v1(&report)).unwrap();
    let output = Command::new(binary)
        .args([
            "profile",
            "--schema",
            "presolve.compile-cost-report",
            "--product",
            product.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["reportId"],
        report.report_id
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("timestamp"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l11g_artifact_graph_projects_only_a_validated_explicit_product() {
    let (root, _) = project();
    let manifest = ResumeManifest {
        schema_version: 6,
        build_id: ResumeBuildId::zero_sentinel(),
        snapshot_schema_version: 1,
        runtime_protocol_version: 1,
        application_root_boundary_id: ResumeBoundaryId::from_str("resume-boundary:root").unwrap(),
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
    .unwrap()
    .0;
    let artifact = build_production_runtime_artifact(&manifest, &graph).unwrap();
    let product_value = build_tooling_artifact_graph_v1(&graph, &artifact).unwrap();
    let product = root.join("artifact-graph.json");
    fs::write(&product, tooling_artifact_graph_json_v1(&product_value)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "graph",
            "artifact",
            "--schema",
            "presolve.artifact-graph",
            "--product",
            product.to_str().unwrap(),
            "--format",
            "dot",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).starts_with("digraph \"presolve.artifact-graph\"")
    );
    fs::remove_dir_all(root).unwrap();
}
