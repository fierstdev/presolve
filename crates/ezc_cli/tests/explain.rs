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
fn html_command_matches_string_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0006-string-state/input/StringGreeting.tsx",
        ])
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
        std::fs::read_to_string(repo_root.join("fixtures/0006-string-state/expected/html.html"))
            .expect("failed to read expected string html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_boolean_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", "fixtures/0007-boolean-state/input/BooleanFlags.tsx"])
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
        std::fs::read_to_string(repo_root.join("fixtures/0007-boolean-state/expected/html.html"))
            .expect("failed to read expected boolean html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_null_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", "fixtures/0008-null-state/input/NullSelection.tsx"])
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
        std::fs::read_to_string(repo_root.join("fixtures/0008-null-state/expected/html.html"))
            .expect("failed to read expected null html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_decrement_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0009-decrement-counter/expected/html.html"),
    )
    .expect("failed to read expected decrement html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_add_subtract_assign_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0010-add-subtract-assign/expected/html.html"),
    )
    .expect("failed to read expected add/subtract html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_direct_assignment_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0011-direct-assignment/expected/html.html"),
    )
    .expect("failed to read expected direct assignment html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_boolean_toggle_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", "fixtures/0012-boolean-toggle/input/ToggleFlag.tsx"])
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
        std::fs::read_to_string(repo_root.join("fixtures/0012-boolean-toggle/expected/html.html"))
            .expect("failed to read expected boolean toggle html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_multi_step_action_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0013-multi-step-action/expected/html.html"),
    )
    .expect("failed to read expected multi-step action html fixture");

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
fn template_command_matches_string_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0006-string-state/input/StringGreeting.tsx",
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
        std::fs::read_to_string(repo_root.join("fixtures/0006-string-state/expected/template.txt"))
            .expect("failed to read expected string template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_boolean_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0007-boolean-state/input/BooleanFlags.tsx",
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

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0007-boolean-state/expected/template.txt"),
    )
    .expect("failed to read expected boolean template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_null_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0008-null-state/input/NullSelection.tsx",
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
        std::fs::read_to_string(repo_root.join("fixtures/0008-null-state/expected/template.txt"))
            .expect("failed to read expected null template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_decrement_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
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

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0009-decrement-counter/expected/template.txt"),
    )
    .expect("failed to read expected decrement template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_add_subtract_assign_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
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

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0010-add-subtract-assign/expected/template.txt"),
    )
    .expect("failed to read expected add/subtract template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_direct_assignment_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
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

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0011-direct-assignment/expected/template.txt"),
    )
    .expect("failed to read expected direct assignment template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_boolean_toggle_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0012-boolean-toggle/input/ToggleFlag.tsx",
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

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0012-boolean-toggle/expected/template.txt"),
    )
    .expect("failed to read expected boolean toggle template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_multi_step_action_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
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

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0013-multi-step-action/expected/template.txt"),
    )
    .expect("failed to read expected multi-step action template fixture");

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

fn assert_json_eq(actual: &str, expected: &str) {
    let actual: serde_json::Value =
        serde_json::from_str(actual).expect("actual output was not valid JSON");
    let expected: serde_json::Value =
        serde_json::from_str(expected).expect("expected fixture was not valid JSON");

    assert_eq!(actual, expected);
}

#[test]
fn manifest_command_matches_valid_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["manifest", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0001-source-summary/expected/manifest.json"),
    )
    .expect("failed to read expected manifest fixture");

    assert_json_eq(&actual, &expected);
}

#[test]
fn manifest_command_matches_broken_tsx_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0002-broken-tsx/input/BrokenCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0002-broken-tsx/expected/manifest.json"))
            .expect("failed to read expected broken manifest fixture");

    assert_json_eq(&actual, &expected);
}

#[test]
fn manifest_command_matches_nested_jsx_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0004-nested-jsx/input/NestedCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0004-nested-jsx/expected/manifest.json"))
            .expect("failed to read expected nested manifest fixture");

    assert_json_eq(&actual, &expected);
}

#[test]
fn manifest_command_matches_string_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0006-string-state/input/StringGreeting.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0006-string-state/expected/manifest.json"),
    )
    .expect("failed to read expected string manifest fixture");

    assert_json_eq(&actual, &expected);
}

#[test]
fn manifest_command_matches_boolean_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0007-boolean-state/input/BooleanFlags.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0007-boolean-state/expected/manifest.json"),
    )
    .expect("failed to read expected boolean manifest fixture");

    assert_json_eq(&actual, &expected);
}

#[test]
fn manifest_command_matches_null_state_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0008-null-state/input/NullSelection.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");

    let expected =
        std::fs::read_to_string(repo_root.join("fixtures/0008-null-state/expected/manifest.json"))
            .expect("failed to read expected null manifest fixture");

    assert_json_eq(&actual, &expected);
}

#[test]
fn manifest_command_matches_decrement_counter_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0009-decrement-counter/expected/manifest.json"),
    )
    .expect("failed to read expected decrement manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_add_subtract_assign_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0010-add-subtract-assign/expected/manifest.json"),
    )
    .expect("failed to read expected add/subtract manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_direct_assignment_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0011-direct-assignment/expected/manifest.json"),
    )
    .expect("failed to read expected direct assignment manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_boolean_toggle_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0012-boolean-toggle/input/ToggleFlag.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0012-boolean-toggle/expected/manifest.json"),
    )
    .expect("failed to read expected boolean toggle manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_multi_step_action_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli manifest");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0013-multi-step-action/expected/manifest.json"),
    )
    .expect("failed to read expected multi-step action manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn build_command_writes_page_manifest_and_runtime_artifacts() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-test-output/nested-counter");

    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir).expect("failed to clean previous test output");
    }

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0004-nested-jsx/input/NestedCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run ezc_cli build");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(stdout.contains("index.html"));
    assert!(stdout.contains("template.manifest.json"));
    assert!(stdout.contains("runtime.js"));

    let actual_html =
        std::fs::read_to_string(out_dir.join("index.html")).expect("failed to read built HTML");

    assert!(actual_html.starts_with("<!doctype html>\n"));
    assert!(actual_html.contains("<title>NestedCounter</title>"));
    assert!(actual_html.contains("<section data-ez-node=\"n0\">"));
    assert!(actual_html.contains("<button data-ez-node=\"n1\""));
    assert!(actual_html.contains("<!-- ez-binding:n2:this.count -->"));
    assert!(actual_html.contains("id=\"ez-template-manifest\""));
    assert!(actual_html.contains("\"name\": \"NestedCounter\""));
    assert!(actual_html.contains("<script src=\"./runtime.js\" defer></script>"));

    let actual_manifest = std::fs::read_to_string(out_dir.join("template.manifest.json"))
        .expect("failed to read built manifest");
    let expected_manifest =
        std::fs::read_to_string(repo_root.join("fixtures/0004-nested-jsx/expected/manifest.json"))
            .expect("failed to read expected nested manifest");

    assert_json_eq(&actual_manifest, &expected_manifest);

    let actual_runtime =
        std::fs::read_to_string(out_dir.join("runtime.js")).expect("failed to read built runtime");

    assert!(actual_runtime.contains("ez-template-manifest"));
    assert!(actual_runtime.contains("SUPPORTED_SCHEMA_VERSION = 1"));
    assert!(actual_runtime.contains("RUNTIME_VERSION = \"0.0.0\""));
    assert!(actual_runtime.contains("validateManifestSchema"));
    assert!(actual_runtime.contains("EZR_UNSUPPORTED_SCHEMA"));
    assert!(actual_runtime.contains("diagnostics"));
    assert!(actual_runtime.contains("data-ez-node"));
    assert!(actual_runtime.contains("ez-binding:"));
    assert!(actual_runtime.contains("normalizeHandlerReference"));
    assert!(actual_runtime.contains("createRuntimeStore"));
    assert!(actual_runtime.contains("readField"));
    assert!(actual_runtime.contains("writeField"));
    assert!(actual_runtime.contains("notifyField"));
    assert!(actual_runtime.contains("installDelegatedEventListeners"));
    assert!(actual_runtime.contains("document.addEventListener(eventType"));
    assert!(!actual_runtime.contains("element.addEventListener(\"click\""));
    assert!(actual_runtime.contains("action.operation !== \"increment\""));
    assert!(actual_runtime.contains("dataset.ezRuntime"));
    assert!(actual_runtime.contains("edgezero:ready"));
    assert!(actual_runtime.contains("window.__EDGEZERO__"));
}
