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
fn asm_command_reports_text_summary() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli asm");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8"),
        "File: fixtures/0001-source-summary/input/Counter.tsx\nApplicationSemanticModel:\n  components: 1\n  templates: 1\n  ownership: 11\n  references: 4\n  provenance: 11\n  diagnostics: 0\n  validation: 0\n"
    );
}

#[test]
fn asm_command_emits_deterministic_json_inspection() {
    let repo_root = repo_root();
    let args = [
        "asm",
        "fixtures/0001-source-summary/input/Counter.tsx",
        "--format",
        "json",
    ];

    let first = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(args)
        .output()
        .expect("failed to run ezc_cli asm --format json");
    let second = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(args)
        .output()
        .expect("failed to rerun ezc_cli asm --format json");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);

    let document: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("ASM inspection output was not valid JSON");
    assert_eq!(document["schema_version"], 1);
    assert_eq!(
        document["file"],
        "fixtures/0001-source-summary/input/Counter.tsx"
    );
    assert_eq!(document["entities"].as_array().map(Vec::len), Some(11));
    assert_eq!(document["diagnostics"], serde_json::json!([]));
    assert_eq!(document["validation"], serde_json::json!([]));
    assert!(document["references"].as_array().is_some_and(|references| {
        references.iter().any(|reference| {
            reference["kind"] == "event-method"
                && reference["source"]
                    == "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/template:render/event-attribute:root.data-ez-on-click"
                && reference["target"]
                    == "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:increment"
        })
    }));
}

#[test]
fn asm_command_inspects_a_sorted_multi_file_unit() {
    let repo_root = repo_root();
    let input_paths = [
        "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
        "fixtures/0001-source-summary/input/Counter.tsx",
    ];
    let args = ["asm", input_paths[0], input_paths[1], "--format", "json"];

    let text_output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", input_paths[0], input_paths[1]])
        .output()
        .expect("failed to run multi-file ezc_cli asm");

    assert!(text_output.status.success());
    assert!(
        String::from_utf8(text_output.stdout)
            .expect("CLI stdout was not valid UTF-8")
            .starts_with(
                "Files:\n  fixtures/0001-source-summary/input/Counter.tsx\n  fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx\nApplicationSemanticModel:\n"
            )
    );

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(args)
        .output()
        .expect("failed to run multi-file ezc_cli asm --format json");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    assert_eq!(
        document["file"],
        "fixtures/0001-source-summary/input/Counter.tsx"
    );
    assert_eq!(
        document["files"],
        serde_json::json!([
            "fixtures/0001-source-summary/input/Counter.tsx",
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
        ])
    );
    assert_eq!(document["entities"].as_array().map(Vec::len), Some(26));
    assert!(document["entities"].as_array().is_some_and(|entities| {
        entities.iter().any(|entity| {
            entity["id"]
                == "module:fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx/component:x-dynamic-attribute-button"
        })
    }));
}

#[test]
fn asm_command_exposes_declared_state_types() {
    let repo_root = repo_root();
    let path = "fixtures/0025-typed-state-annotations/input/TypedState.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run typed-state ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let count = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities.iter().find(|entity| {
                entity["id"]
                    == "module:fixtures/0025-typed-state-annotations/input/TypedState.tsx/component:x-typed-state/state:count"
            })
        })
        .expect("count state entity");

    assert_eq!(
        count["declared_type"],
        serde_json::json!({
            "text": "number",
            "kind": "number",
            "provenance": {
                "path": path,
                "start": 72,
                "end": 80,
                "line": 3,
                "column": 8,
            }
        })
    );

    let status = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities.iter().find(|entity| {
                entity["id"]
                    == "module:fixtures/0025-typed-state-annotations/input/TypedState.tsx/component:x-typed-state/state:status"
            })
        })
        .expect("status state entity");

    assert!(status["declared_type"].get("kind").is_none());
}

#[test]
fn asm_command_reports_primitive_declared_state_type_mismatches() {
    let repo_root = repo_root();
    let path = "fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run invalid typed-state ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("ASM inspection diagnostics");

    assert_eq!(diagnostics.len(), 4);
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic["code"] == "EZC1016"
            && diagnostic["message"].as_str().is_some_and(|message| {
                message.contains("declares") && message.contains("initializes")
            })
    }));

    let count = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `count`"))
        })
        .expect("count mismatch diagnostic");
    assert_eq!(
        count["provenance"]["path"],
        "fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx"
    );
    assert_eq!(count["provenance"]["line"], 3);
    assert_eq!(count["provenance"]["column"], 8);
}

#[test]
fn asm_command_omits_unavailable_diagnostic_provenance() {
    let repo_root = repo_root();
    let path = "fixtures/0003-semantic-errors/input/BrokenSemantics.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run semantic-errors ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let diagnostic = document["diagnostics"]
        .as_array()
        .and_then(|diagnostics| {
            diagnostics
                .iter()
                .find(|diagnostic| diagnostic["code"] == "EZC1003")
        })
        .expect("unlocated semantic diagnostic");

    assert!(diagnostic.get("provenance").is_none());
}

#[test]
fn asm_command_reports_primitive_action_type_mismatches() {
    let repo_root = repo_root();
    let path = "fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run invalid typed-actions ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("ASM inspection diagnostics");

    assert_eq!(diagnostics.len(), 4);
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic["code"] == "EZC1017"
            && diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("action `apply` assigns"))
    }));

    let count = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `count`"))
        })
        .expect("count action mismatch diagnostic");
    assert_eq!(count["provenance"]["path"], path);
    assert_eq!(count["provenance"]["line"], 11);
    assert_eq!(count["provenance"]["column"], 5);
}

#[test]
fn asm_command_reports_non_boolean_primitive_toggle_actions() {
    let repo_root = repo_root();
    let path = "fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run invalid typed-toggles ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("ASM inspection diagnostics");

    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic["code"] == "EZC1018"
            && diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("applies a boolean toggle"))
    }));

    let count = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `count`"))
        })
        .expect("count toggle diagnostic");
    assert_eq!(count["provenance"]["path"], path);
    assert_eq!(count["provenance"]["line"], 10);
    assert_eq!(count["provenance"]["column"], 5);
}

#[test]
fn asm_command_reports_non_numeric_primitive_increment_and_decrement_actions() {
    let repo_root = repo_root();
    let path =
        "fixtures/0030-primitive-numeric-action-type-diagnostics/input/InvalidTypedNumericActions.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run invalid typed-numeric-actions ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("ASM inspection diagnostics");

    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic["code"] == "EZC1019"
            && diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("applies numeric"))
    }));

    let title = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `title`"))
        })
        .expect("title numeric action diagnostic");
    assert_eq!(title["provenance"]["path"], path);
    assert_eq!(title["provenance"]["line"], 10);
    assert_eq!(title["provenance"]["column"], 5);
}

#[test]
fn asm_command_reports_compound_numeric_action_target_and_operand_mismatches() {
    let repo_root = repo_root();
    let path =
        "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to run invalid typed-compound-actions ezc_cli asm --format json");

    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output was not valid JSON");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("ASM inspection diagnostics");

    assert_eq!(diagnostics.len(), 5);
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic["code"] == "EZC1020" || diagnostic["code"] == "EZC1021"));

    let title = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `title`"))
        })
        .expect("title compound action diagnostic");
    assert_eq!(title["provenance"]["path"], path);
    assert_eq!(title["provenance"]["line"], 9);
    assert_eq!(title["provenance"]["column"], 5);
}

#[test]
fn asm_command_text_reports_source_provenanced_compiler_diagnostics() {
    let repo_root = repo_root();
    let path =
        "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path])
        .output()
        .expect("failed to run invalid typed-compound-actions ezc_cli asm");

    assert!(output.status.success());

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(actual.contains("  compiler diagnostics:\n"));
    assert!(actual.contains("    EZC1020: state field `title`"));
    assert!(actual.contains("    EZC1021: action `apply`"));
    assert!(actual.contains(&format!("      at {path}:9:5 span=")));
}

#[test]
fn check_command_succeeds_without_diagnostics() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["check", "fixtures/0001-source-summary/input/Counter.tsx"])
        .output()
        .expect("failed to run ezc_cli check");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8"),
        "Check:\n  files: 1\n  parser diagnostics: 0\n  compiler diagnostics: 0\n  ASM validation diagnostics: 0\n  parser fail on: Error\n"
    );
}

#[test]
fn check_command_fails_for_compiler_diagnostics() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "check",
            "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli check");

    assert!(!output.status.success());
    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(actual.contains("  compiler diagnostics: 5\n"));
    assert!(actual.contains("    EZC1020: state field `title`"));
}

#[test]
fn check_command_emits_json_diagnostics() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["check", "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx", "--format", "json"])
        .output().expect("failed to run ezc_cli check");
    assert!(!output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    assert_eq!(document["schema_version"], 1);
    assert_eq!(
        document["compiler_diagnostics"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(document["fail_on"], "Error");
}

#[test]
fn check_command_filters_displayed_diagnostic_categories_without_changing_failure() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["check", "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx", "--format", "json", "--category", "parser"])
        .output().expect("failed to run ezc_cli check");
    assert!(!output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    assert_eq!(document["summary"]["compiler_diagnostics"], 5);
    assert_eq!(document["categories"], serde_json::json!(["parser"]));
    assert_eq!(document["compiler_diagnostics"], serde_json::json!([]));
}

#[test]
fn check_command_fails_for_parser_diagnostics() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["check", "fixtures/0002-broken-tsx/input/BrokenCounter.tsx"])
        .output()
        .expect("failed to run ezc_cli check");

    assert!(!output.status.success());
    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(actual.contains("  parser diagnostics: 1\n"));
    assert!(actual.contains("  parser Error:"));
    assert!(actual
        .contains("    at fixtures/0002-broken-tsx/input/BrokenCounter.tsx:9:16 span=198..199\n"));
}

#[test]
fn check_command_exposes_parser_diagnostic_label_provenance_in_json() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "check",
            "fixtures/0002-broken-tsx/input/BrokenCounter.tsx",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run ezc_cli check");

    assert!(!output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    assert_eq!(
        document["parser_diagnostics"][0]["labels"],
        serde_json::json!([{"line": 9, "column": 16, "start": 198, "end": 199}])
    );
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
fn parse_command_matches_keyed_list_semantics_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "parse",
            "fixtures/0019-keyed-list-semantics/input/KeyedList.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli parse");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0019-keyed-list-semantics/expected/parse.txt"),
    )
    .expect("failed to read expected keyed list parse fixture");

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
fn graph_command_matches_keyed_list_semantics_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "graph",
            "fixtures/0019-keyed-list-semantics/input/KeyedList.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli graph");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0019-keyed-list-semantics/expected/graph.txt"),
    )
    .expect("failed to read expected keyed list graph fixture");

    assert_eq!(actual, expected);
}

#[test]
fn graph_command_matches_keyed_list_diagnostics_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "graph",
            "fixtures/0022-keyed-list-diagnostics/input/ListKeyDiagnostics.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli graph");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0022-keyed-list-diagnostics/expected/graph.txt"),
    )
    .expect("failed to read expected keyed list diagnostics graph fixture");

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
fn html_command_matches_static_attributes_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0014-static-attributes/input/StaticAttributePanel.tsx",
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
        repo_root.join("fixtures/0014-static-attributes/expected/html.html"),
    )
    .expect("failed to read expected static attributes html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_dynamic_attributes_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
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
        repo_root.join("fixtures/0015-dynamic-attributes/expected/html.html"),
    )
    .expect("failed to read expected dynamic attributes html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_fragments_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", "fixtures/0016-fragments/input/FragmentPanel.tsx"])
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
        std::fs::read_to_string(repo_root.join("fixtures/0016-fragments/expected/html.html"))
            .expect("failed to read expected fragments html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_conditional_rendering_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx",
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
        repo_root.join("fixtures/0017-conditional-rendering/expected/html.html"),
    )
    .expect("failed to read expected conditional html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_logical_and_conditional_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx",
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
        repo_root.join("fixtures/0018-logical-and-conditional/expected/html.html"),
    )
    .expect("failed to read expected logical-and html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_static_keyed_list_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0020-static-keyed-list/expected/html.html"),
    )
    .expect("failed to read expected static keyed list html fixture");

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
fn template_command_matches_static_attributes_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0014-static-attributes/input/StaticAttributePanel.tsx",
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
        repo_root.join("fixtures/0014-static-attributes/expected/template.txt"),
    )
    .expect("failed to read expected static attributes template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_dynamic_attributes_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
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
        repo_root.join("fixtures/0015-dynamic-attributes/expected/template.txt"),
    )
    .expect("failed to read expected dynamic attributes template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_fragments_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0016-fragments/input/FragmentPanel.tsx",
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
        std::fs::read_to_string(repo_root.join("fixtures/0016-fragments/expected/template.txt"))
            .expect("failed to read expected fragments template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_conditional_rendering_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx",
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
        repo_root.join("fixtures/0017-conditional-rendering/expected/template.txt"),
    )
    .expect("failed to read expected conditional template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_logical_and_conditional_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx",
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
        repo_root.join("fixtures/0018-logical-and-conditional/expected/template.txt"),
    )
    .expect("failed to read expected logical-and template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_keyed_list_semantics_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0019-keyed-list-semantics/input/KeyedList.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli template");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0019-keyed-list-semantics/expected/template.txt"),
    )
    .expect("failed to read expected keyed list template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn template_command_matches_static_keyed_list_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli template");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0020-static-keyed-list/expected/template.txt"),
    )
    .expect("failed to read expected static keyed list template fixture");

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
fn manifest_command_matches_static_attributes_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0014-static-attributes/input/StaticAttributePanel.tsx",
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
        repo_root.join("fixtures/0014-static-attributes/expected/manifest.json"),
    )
    .expect("failed to read expected static attributes manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_dynamic_attributes_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
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
        repo_root.join("fixtures/0015-dynamic-attributes/expected/manifest.json"),
    )
    .expect("failed to read expected dynamic attributes manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_fragments_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0016-fragments/input/FragmentPanel.tsx",
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
        std::fs::read_to_string(repo_root.join("fixtures/0016-fragments/expected/manifest.json"))
            .expect("failed to read expected fragments manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_conditional_rendering_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx",
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
        repo_root.join("fixtures/0017-conditional-rendering/expected/manifest.json"),
    )
    .expect("failed to read expected conditional manifest fixture");

    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("actual manifest JSON was invalid");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).expect("expected manifest JSON was invalid");

    assert_eq!(actual_json, expected_json);
}

#[test]
fn manifest_command_matches_logical_and_conditional_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "manifest",
            "fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx",
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
        repo_root.join("fixtures/0018-logical-and-conditional/expected/manifest.json"),
    )
    .expect("failed to read expected logical-and manifest fixture");

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

#[test]
fn html_command_matches_keyed_list_reconciliation_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0021-keyed-list-reconciliation/input/KeyedListReconciliation.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0021-keyed-list-reconciliation/expected/html.html"),
    )
    .expect("failed to read expected keyed list reconciliation html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_object_keyed_list_reconciliation_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0025-object-keyed-list-reconciliation/input/ObjectKeyedListReconciliation.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0025-object-keyed-list-reconciliation/expected/html.html"),
    )
    .expect("failed to read expected object keyed list reconciliation html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_dynamic_list_item_behavior_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0026-dynamic-list-item-behavior/input/DynamicListItemBehavior.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0026-dynamic-list-item-behavior/expected/html.html"),
    )
    .expect("failed to read expected dynamic list item behavior html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_static_object_keyed_list_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0024-static-object-keyed-list/input/StaticObjectKeyedList.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli html");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0024-static-object-keyed-list/expected/html.html"),
    )
    .expect("failed to read expected static object keyed list html fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn template_command_matches_keyed_list_reconciliation_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "template",
            "fixtures/0021-keyed-list-reconciliation/input/KeyedListReconciliation.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli template");

    assert!(
        output.status.success(),
        "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0021-keyed-list-reconciliation/expected/template.txt"),
    )
    .expect("failed to read expected keyed list reconciliation template fixture");

    assert_eq!(actual, expected);
}

#[test]
fn manifest_command_matches_list_and_object_value_fixtures() {
    let repo_root = repo_root();

    for (input, expected) in [
        (
            "fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx",
            "fixtures/0020-static-keyed-list/expected/manifest.json",
        ),
        (
            "fixtures/0021-keyed-list-reconciliation/input/KeyedListReconciliation.tsx",
            "fixtures/0021-keyed-list-reconciliation/expected/manifest.json",
        ),
        (
            "fixtures/0023-recursive-object-values/input/RecursiveObjectValues.tsx",
            "fixtures/0023-recursive-object-values/expected/manifest.json",
        ),
        (
            "fixtures/0024-static-object-keyed-list/input/StaticObjectKeyedList.tsx",
            "fixtures/0024-static-object-keyed-list/expected/manifest.json",
        ),
        (
            "fixtures/0025-object-keyed-list-reconciliation/input/ObjectKeyedListReconciliation.tsx",
            "fixtures/0025-object-keyed-list-reconciliation/expected/manifest.json",
        ),
        (
            "fixtures/0026-dynamic-list-item-behavior/input/DynamicListItemBehavior.tsx",
            "fixtures/0026-dynamic-list-item-behavior/expected/manifest.json",
        ),
    ] {
        let output = Command::new(ezc_cli_bin())
            .current_dir(&repo_root)
            .args(["manifest", input])
            .output()
            .expect("failed to run ezc_cli manifest");

        assert!(
            output.status.success(),
            "expected command to succeed\\nstatus: {}\\nstderr:\\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
        let expected = std::fs::read_to_string(repo_root.join(expected))
            .expect("failed to read expected manifest fixture");

        assert_json_eq(&actual, &expected);
    }
}
