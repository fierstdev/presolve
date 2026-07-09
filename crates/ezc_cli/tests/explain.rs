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

fn normalize_html_for_fixture(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" >", ">")
        .trim()
        .to_string()
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
fn parse_command_matches_valid_counter_fixture() {
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

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0001-source-summary/expected/parse.txt"))
            .expect("failed to read expected parse fixture");

    assert_eq!(actual, expected);
}

#[test]
fn parse_command_matches_broken_tsx_fixture() {
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

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0002-broken-tsx/expected/parse.txt"))
            .expect("failed to read expected broken parse fixture");

    assert_eq!(actual, expected);
}

#[test]
fn graph_command_matches_valid_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["graph", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli graph");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0001-source-summary/expected/graph.txt"))
            .expect("failed to read expected graph fixture");

    assert_eq!(actual, expected);
}

#[test]
fn graph_command_matches_broken_tsx_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["graph", "fixtures/0002-broken-tsx/input/BrokenCounter.tsx"])
        .output()
        .expect("failed to run ezc_cli graph");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0002-broken-tsx/expected/graph.txt"))
            .expect("failed to read expected broken graph fixture");

    assert_eq!(actual, expected);
}

#[test]
fn graph_command_matches_semantic_errors_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "graph",
            "fixtures/0003-semantic-errors/input/BrokenSemantics.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli graph");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0003-semantic-errors/expected/graph.txt"))
            .expect("failed to read expected semantic graph fixture");

    assert_eq!(actual, expected);
}

#[test]
fn html_command_matches_valid_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0001-source-summary/expected/html.html"))
            .expect("failed to read expected html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_broken_tsx_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", "fixtures/0002-broken-tsx/input/BrokenCounter.tsx"])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0002-broken-tsx/expected/html.html"))
            .expect("failed to read expected broken html fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_valid_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["template", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli template");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0001-source-summary/expected/template.txt"),
    )
    .expect("failed to read expected template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_broken_tsx_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0002-broken-tsx/input/BrokenCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli template");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0002-broken-tsx/expected/template.txt"))
            .expect("failed to read expected broken template fixture");

    assert_eq!(actual, expected);
}
