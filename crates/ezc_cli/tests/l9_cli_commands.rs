use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

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

#[test]
fn l9d_explicit_check_emits_the_stable_json_result_envelope() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
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
        .args(["build", "--config", config.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("L9D003_MISSING_SOURCE_INPUT"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l9e_cache_inspection_projects_l6_canonical_json() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
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
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema"], "presolve.cache-inspection-report.v1");
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["enabled"], true);
    assert!(root.join(".presolve/cache/manifest.json").is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn l9f_workspace_executes_the_explicit_project_through_l7() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
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
    let reserved = Command::new(binary).arg("dev").output().unwrap();
    assert_eq!(reserved.status.code(), Some(6));
    assert!(reserved.stdout.is_empty());
    assert!(String::from_utf8_lossy(&reserved.stderr).contains("reserved"));
}

#[test]
fn l9_watch_once_submits_the_complete_candidate_to_l8() {
    let (root, config) = project();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
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
