use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve repository root")
}

fn ezc_cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ezc_cli")
}

#[test]
fn explain_command_matches_text_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["explain", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli explain");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0001-source-summary/expected/explain.txt"),
    )
    .expect("failed to read expected text fixture");

    assert_eq!(actual, expected);
}

#[test]
fn explain_command_matches_json_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "explain",
            "fixtures/0001-source-summary/input/Counter.tsx",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run ezc_cli explain --format json");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0001-source-summary/expected/explain.json"),
    )
    .expect("failed to read expected JSON fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual CLI JSON output was invalid");

    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected JSON fixture was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn parse_command_reports_valid_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["parse", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli parse");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    assert!(actual.contains("File: fixtures/0001-source-summary/input/Counter.tsx"));
    assert!(actual.contains("Diagnostics:\n  none"));
    assert!(actual.contains("class Counter at 1:1"));
    assert!(actual.contains("@route(\"/counter\") at 1:1"));
    assert!(actual.contains("@component(\"x-counter\") at 2:1"));
    assert!(actual.contains("count = state(...)"));
    assert!(actual.contains("render at 10:3"));
    assert!(actual.contains("jsx root <button>"));
    assert!(actual.contains("attributes: onClick={...}"));
    assert!(actual.contains("bindings: this.count"));
}

#[test]
fn parse_command_reports_broken_tsx_diagnostic() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["parse", "fixtures/0002-broken-tsx/input/BrokenCounter.tsx"])
        .output()
        .expect("failed to run ezc_cli parse");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    assert!(actual.contains("File: fixtures/0002-broken-tsx/input/BrokenCounter.tsx"));
    assert!(actual.contains("Error: Unexpected token"));
    assert!(actual.contains("at 9:16 span=198..199"));
    assert!(actual.contains("Classes:\n  none"));
}
