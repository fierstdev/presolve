use std::path::{Path, PathBuf};
use std::process::Command;

use ezc_core::{
    build_application_semantic_model, build_resume_plan, lower_components_to_ir,
    optimize_computed_ir, IrConstant, IrInstructionKind, SerializationCompatibility,
};

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
fn explain_command_inspects_entities_through_the_canonical_asm_path() {
    let path = "fixtures/0001-source-summary/input/Counter.tsx";
    let entity_id = "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter";
    let inspection_args = [
        path,
        "--entity",
        entity_id,
        "--child-kind",
        "method",
        "--reference-kind",
        "event-method",
        "--format",
        "json",
    ];

    let explain = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .arg("explain")
        .args(inspection_args)
        .output()
        .expect("failed to inspect an ASM entity through ezc_cli explain");
    let asm = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .arg("asm")
        .args(inspection_args)
        .output()
        .expect("failed to inspect an ASM entity through ezc_cli asm");

    assert!(explain.status.success());
    assert!(asm.status.success());
    assert_eq!(explain.stdout, asm.stdout);

    let document: serde_json::Value =
        serde_json::from_slice(&explain.stdout).expect("entity inspection JSON");
    assert_eq!(document["entity"]["id"], entity_id);
    assert_eq!(
        document["children"],
        serde_json::json!([
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:increment",
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:render"
        ])
    );
}

#[test]
fn explain_command_rejects_entity_filters_without_a_selector() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "explain",
            "fixtures/0001-source-summary/input/Counter.tsx",
            "--child-kind",
            "method",
        ])
        .output()
        .expect("failed to run filtered ezc_cli explain");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("require an ASM entity selector"));
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
        "File: fixtures/0001-source-summary/input/Counter.tsx\nApplicationSemanticModel:\n  components: 1\n  templates: 1\n  ownership: 11\n  references: 4\n  provenance: 11\n  semantic types: 2\n  diagnostics: 0\n  validation: 0\n"
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
    assert_eq!(document["schema_version"], 4);
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
fn asm_command_exports_a_deterministic_semantic_graph() {
    let repo_root = repo_root();
    let path = "fixtures/0001-source-summary/input/Counter.tsx";
    let args = ["asm", path, "--format", "graph"];

    let first = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(args)
        .output()
        .expect("failed to export a semantic graph");
    let second = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(args)
        .output()
        .expect("failed to re-export a semantic graph");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);

    let graph: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("semantic graph output was not valid JSON");
    let component_id = "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter";
    assert_eq!(graph["schema_version"], 5);
    assert_eq!(graph["roots"], serde_json::json!([component_id]));
    assert_eq!(graph["nodes"].as_array().map(Vec::len), Some(11));
    assert_eq!(graph["edges"].as_array().map(Vec::len), Some(14));
    assert!(graph["nodes"].as_array().is_some_and(|nodes| {
        nodes.iter().any(|node| {
            node["id"]
                == "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/state:count"
                && node["kind"] == "state-field"
        })
    }));
    assert!(graph["edges"].as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            edge["kind"] == "ownership"
                && edge["source"] == component_id
                && edge["target"]
                    == "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:increment"
        }) && edges.iter().any(|edge| {
            edge["kind"] == "action-state"
                && edge["source"]
                    == "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/action:increment:0"
                && edge["target"]
                    == "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/state:count"
        })
    }));

    let selected = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", path, "--format", "graph", "--entity", component_id])
        .output()
        .expect("failed to reject a selected graph export");
    assert!(!selected.status.success());
    assert!(String::from_utf8_lossy(&selected.stderr)
        .contains("cannot be combined with ASM entity selection or filters"));
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
fn asm_command_inspects_one_semantic_entity() {
    let path = "fixtures/0001-source-summary/input/Counter.tsx";
    let entity_id =
        "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/state:count";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--entity", entity_id, "--format", "json"])
        .output()
        .expect("failed to inspect ASM entity");

    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("entity JSON");
    assert_eq!(document["schema_version"], 4);
    assert_eq!(document["entity"]["id"], entity_id);
    assert_eq!(document["entity"]["kind"], "state-field");
    assert_eq!(document["entity"]["semantic_type"]["type_text"], "number");
    assert_eq!(document["entity"]["semantic_type"]["status"], "inferred");
    assert_eq!(
        document["parents"],
        serde_json::json!([
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter"
        ])
    );
    assert_eq!(document["children"], serde_json::json!([]));
    assert_eq!(document["descendant_count"], 0);
    assert_eq!(document["outgoing_references"], serde_json::json!([]));
    assert_eq!(
        document["incoming_references"].as_array().map(Vec::len),
        Some(2)
    );

    let text_output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--entity", entity_id])
        .output()
        .expect("failed to inspect ASM entity as text");
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("entity text");
    assert!(text.contains(&format!("ASM Entity: {entity_id}\n")));
    assert!(text.contains("  kind: state-field\n"));
    assert!(text.contains("  semantic type: number\n"));
    assert!(text.contains("    status: inferred\n"));
    assert!(text.contains("  parents: 1\n"));
    assert!(text.contains("  incoming references: 2\n"));

    let explain_output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["explain", path, "--entity", entity_id, "--format", "json"])
        .output()
        .expect("failed to inspect ASM entity through explain");
    assert!(explain_output.status.success());
    let explain_document: serde_json::Value =
        serde_json::from_slice(&explain_output.stdout).expect("explain entity JSON");
    assert_eq!(
        explain_document["entity"]["semantic_type"]["type_text"],
        "number"
    );
}

#[test]
fn asm_and_explain_inspect_canonical_computed_metadata() {
    let path = "fixtures/0044-computed-runtime-execution/input/RuntimeComputed.tsx";
    let component = "module:fixtures/0044-computed-runtime-execution/input/RuntimeComputed.tsx/component:x-runtime-computed";
    let doubled = format!("{component}/computed:doubled");
    let label = format!("{component}/computed:label");
    let count = format!("{component}/state:count");
    let inspection_args = [path, "--entity", &label, "--format", "json"];

    let asm = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .arg("asm")
        .args(inspection_args)
        .output()
        .expect("failed to inspect computed ASM entity");
    let explain = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .arg("explain")
        .args(inspection_args)
        .output()
        .expect("failed to inspect computed entity through explain");
    let full_asm = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect the full computed ASM document");
    let text = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--entity", &label])
        .output()
        .expect("failed to inspect computed ASM entity as text");

    assert!(asm.status.success());
    assert!(explain.status.success());
    assert!(full_asm.status.success());
    assert!(text.status.success());
    assert_eq!(asm.stdout, explain.stdout);

    let document: serde_json::Value =
        serde_json::from_slice(&asm.stdout).expect("computed entity inspection JSON");
    assert_eq!(document["schema_version"], 4);
    assert_eq!(document["entity"]["computed"]["computed_type"], "number");
    assert_eq!(
        document["entity"]["computed"]["dependencies"],
        serde_json::json!([doubled, count])
    );
    assert_eq!(
        document["entity"]["computed"]["dependents"],
        serde_json::json!([])
    );
    assert_eq!(document["entity"]["computed"]["evaluation_order"], 1);
    assert_eq!(document["entity"]["computed"]["evaluation_batch"], 1);
    assert_eq!(document["entity"]["computed"]["purity"], "pure");
    assert_eq!(
        document["entity"]["computed"]["serializability"],
        "serializable"
    );
    assert_eq!(document["entity"]["computed"]["ir_function"], label);

    let full_document: serde_json::Value =
        serde_json::from_slice(&full_asm.stdout).expect("full computed ASM inspection JSON");
    let full_entity = full_document["entities"]
        .as_array()
        .expect("full ASM entities")
        .iter()
        .find(|entity| entity["id"] == label)
        .expect("computed entity in full ASM inspection");
    assert_eq!(full_entity["computed"], document["entity"]["computed"]);

    let text = String::from_utf8(text.stdout).expect("computed entity text");
    assert!(text.contains("  computed:\n"));
    assert!(text.contains("    evaluation order: Some(1)\n"));
    assert!(text.contains("    purity: pure\n"));
}

#[test]
fn asm_and_explain_project_one_canonical_effect_inspection_record() {
    let path = "fixtures/0053-effect-initial-runtime/input/InitialEffectRuntime.tsx";
    let component = format!("module:{path}/component:x-effect-initial-runtime");
    let effect = format!("{component}/effect:report");
    let count = format!("{component}/state:count");
    let title = format!("{component}/state:title");
    let doubled = format!("{component}/computed:doubled");
    let batch = format!("{component}/action-batch:update");
    let args = [path, "--entity", &effect, "--format", "json"];

    let asm = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .arg("asm")
        .args(args)
        .output()
        .expect("failed to inspect effect through asm");
    let explain = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .arg("explain")
        .args(args)
        .output()
        .expect("failed to inspect effect through explain");
    let full = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect full ASM document");
    let text = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--entity", &effect])
        .output()
        .expect("failed to inspect effect text");

    assert!(asm.status.success());
    assert!(explain.status.success());
    assert!(full.status.success());
    assert!(text.status.success());
    assert_eq!(asm.stdout, explain.stdout);

    let selected: serde_json::Value = serde_json::from_slice(&asm.stdout).expect("effect JSON");
    let inspection = &selected["entity"]["effect"];
    assert_eq!(selected["schema_version"], 4);
    assert_eq!(inspection["validation"]["status"], "valid");
    assert_eq!(
        inspection["direct_dependencies"]["state"],
        serde_json::json!([title])
    );
    assert_eq!(
        inspection["direct_dependencies"]["computed"],
        serde_json::json!([doubled])
    );
    assert_eq!(
        inspection["transitive_dependencies"]["state"],
        serde_json::json!([count, title])
    );
    assert_eq!(inspection["dependents"], serde_json::json!([]));
    assert_eq!(
        inspection["initial_trigger"]["render_boundary"],
        "after_initial_render"
    );
    assert_eq!(inspection["action_triggers"][0]["action_batch_id"], batch);
    assert_eq!(
        inspection["action_triggers"][0]["required_computed"],
        serde_json::json!([doubled])
    );
    assert_eq!(inspection["capabilities"].as_array().map(Vec::len), Some(3));
    assert_eq!(inspection["ir"]["function_id"], effect);
    assert_eq!(inspection["runtime"]["registered"], true);
    assert_eq!(inspection["resumability"]["initial_status"], "pending");

    let full: serde_json::Value = serde_json::from_slice(&full.stdout).expect("full effect JSON");
    let full_entity = full["entities"]
        .as_array()
        .expect("full entities")
        .iter()
        .find(|entity| entity["id"] == effect)
        .expect("effect entity in full document");
    assert_eq!(full_entity["effect"], inspection.clone());

    let text = String::from_utf8(text.stdout).expect("effect text");
    assert!(text.contains("  Effect:\n"));
    assert!(text.contains("    Resumability:"));
}

#[test]
fn computed_fixture_suite_covers_arithmetic_diamond_and_cycles() {
    let repo_root = repo_root();
    let arithmetic_path = "fixtures/0046-computed-arithmetic/input/ComputedArithmetic.tsx";
    let arithmetic_component = format!("module:{arithmetic_path}/component:x-computed-arithmetic");
    let total = format!("{arithmetic_component}/computed:total");
    let count = format!("{arithmetic_component}/state:count");
    let offset = format!("{arithmetic_component}/state:offset");
    let arithmetic = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "asm",
            arithmetic_path,
            "--entity",
            &total,
            "--format",
            "json",
        ])
        .output()
        .expect("failed to inspect computed arithmetic fixture");
    assert!(arithmetic.status.success());
    let arithmetic: serde_json::Value =
        serde_json::from_slice(&arithmetic.stdout).expect("computed arithmetic fixture JSON");
    assert_eq!(arithmetic["entity"]["computed"]["computed_type"], "number");
    assert_eq!(
        arithmetic["entity"]["computed"]["dependencies"],
        serde_json::json!([count, offset])
    );

    let diamond_path = "fixtures/0047-computed-diamond/input/ComputedDiamond.tsx";
    let diamond_component = format!("module:{diamond_path}/component:x-computed-diamond");
    let diamond_total = format!("{diamond_component}/computed:total");
    let diamond = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "asm",
            diamond_path,
            "--entity",
            &diamond_total,
            "--format",
            "json",
        ])
        .output()
        .expect("failed to inspect computed diamond fixture");
    assert!(diamond.status.success());
    let diamond: serde_json::Value =
        serde_json::from_slice(&diamond.stdout).expect("computed diamond fixture JSON");
    let dependencies = diamond["entity"]["computed"]["dependencies"]
        .as_array()
        .expect("computed diamond dependencies");
    assert!(dependencies
        .iter()
        .any(|dependency| dependency == &format!("{diamond_component}/computed:doubled")));
    assert!(dependencies
        .iter()
        .any(|dependency| dependency == &format!("{diamond_component}/computed:tripled")));
    assert!(dependencies
        .iter()
        .any(|dependency| dependency == &format!("{diamond_component}/state:count")));

    let cycle = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "check",
            "fixtures/0048-computed-cycle-diagnostics/input/ComputedCycle.tsx",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to check computed cycle fixture");
    assert!(!cycle.status.success());
    let cycle: serde_json::Value =
        serde_json::from_slice(&cycle.stdout).expect("computed cycle fixture JSON");
    assert!(cycle["compiler_diagnostics"]
        .as_array()
        .is_some_and(|diagnostics| {
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["code"] == "EZC1035")
        }));
}

#[test]
fn computed_fixture_suite_covers_folding_and_serialization() {
    let repo_root = repo_root();
    let folding_path =
        repo_root.join("fixtures/0049-computed-constant-folding/input/ComputedConstantFolding.tsx");
    let folding_source =
        std::fs::read_to_string(&folding_path).expect("failed to read computed folding fixture");
    let folding_parsed = ezc_parser::parse_file(&folding_path, &folding_source);
    let folding_asm = build_application_semantic_model(&folding_parsed);
    let answer = folding_asm.components[0].id.computed("answer");
    let optimized = optimize_computed_ir(&lower_components_to_ir(&folding_asm));
    let function = optimized.output.modules[0]
        .functions
        .iter()
        .find(|function| function.id == answer)
        .expect("optimized computed folding function");
    assert_eq!(function.blocks[0].instructions.len(), 1);
    assert!(matches!(
        function.blocks[0].instructions[0].kind,
        IrInstructionKind::Constant {
            value: IrConstant::Number(ref value)
        } if value == "3"
    ));

    let serialization_path =
        repo_root.join("fixtures/0050-computed-serialization/input/ComputedSerialization.tsx");
    let serialization_source = std::fs::read_to_string(&serialization_path)
        .expect("failed to read computed serialization fixture");
    let serialization_parsed = ezc_parser::parse_file(&serialization_path, &serialization_source);
    let serialization_asm = build_application_semantic_model(&serialization_parsed);
    let snapshot = serialization_asm.components[0].id.computed("snapshot");
    assert_eq!(
        serialization_asm.serialization_compatibility_of(&snapshot),
        Some(SerializationCompatibility::Serializable)
    );
    assert_eq!(
        build_resume_plan(&serialization_asm).components[0].computed[0].computed,
        snapshot
    );
}

#[test]
fn computed_fixture_suite_covers_multi_file_identity() {
    let repo_root = repo_root();
    let alpha_path = "fixtures/0051-computed-multi-file/input/AlphaComputed.tsx";
    let beta_path = "fixtures/0051-computed-multi-file/input/BetaComputed.tsx";
    let multi_file = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["asm", alpha_path, beta_path, "--format", "json"])
        .output()
        .expect("failed to inspect computed multi-file fixture");
    assert!(multi_file.status.success());
    let multi_file: serde_json::Value =
        serde_json::from_slice(&multi_file.stdout).expect("computed multi-file fixture JSON");
    assert_eq!(
        multi_file["files"],
        serde_json::json!([alpha_path, beta_path])
    );
    assert!(multi_file["entities"].as_array().is_some_and(|entities| {
        entities.iter().any(|entity| entity["id"]
            == "module:fixtures/0051-computed-multi-file/input/AlphaComputed.tsx/component:x-alpha-computed/computed:doubled")
            && entities.iter().any(|entity| entity["id"]
                == "module:fixtures/0051-computed-multi-file/input/BetaComputed.tsx/component:x-beta-computed/computed:title")
    }));
}

#[test]
fn asm_entity_inspection_navigates_action_ancestors_children_and_references() {
    let path = "fixtures/0001-source-summary/input/Counter.tsx";
    let action_id =
        "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/action:increment:0";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--entity", action_id, "--format", "json"])
        .output()
        .expect("failed to navigate an ASM action entity");

    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("entity JSON");
    assert_eq!(
        document["parents"],
        serde_json::json!([
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:increment",
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter"
        ])
    );
    assert_eq!(document["children"], serde_json::json!([]));
    assert_eq!(
        document["outgoing_references"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(document["outgoing_references"][0]["kind"], "action-state");
    assert_eq!(document["incoming_references"], serde_json::json!([]));
}

#[test]
fn asm_command_rejects_unknown_semantic_entity() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "asm",
            "fixtures/0001-source-summary/input/Counter.tsx",
            "--entity",
            "component:unknown",
        ])
        .output()
        .expect("failed to inspect unknown ASM entity");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown ASM entity"));
}

#[test]
fn asm_command_inspects_entity_selected_by_source_offset() {
    let path = "fixtures/0001-source-summary/input/Counter.tsx";
    let entity_id =
        "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/state:count";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "asm", path, "--source", path, "--offset", "79", "--format", "json",
        ])
        .output()
        .expect("failed to inspect ASM entity by source offset");

    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("entity JSON");
    assert_eq!(document["entity"]["id"], entity_id);

    let no_entity = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--source", path, "--offset", "100000"])
        .output()
        .expect("failed to inspect missing source offset");
    assert!(!no_entity.status.success());
    assert!(String::from_utf8_lossy(&no_entity.stderr).contains("no ASM entity"));

    let conflicting_selector = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "asm", path, "--entity", entity_id, "--source", path, "--offset", "79",
        ])
        .output()
        .expect("failed to inspect conflicting ASM selectors");
    assert!(!conflicting_selector.status.success());
    assert!(String::from_utf8_lossy(&conflicting_selector.stderr)
        .contains("--entity cannot be combined"));
}

#[test]
fn asm_command_filters_selected_entity_children_and_relations() {
    let path = "fixtures/0001-source-summary/input/Counter.tsx";
    let component_id = "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter";
    let state_id =
        "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/state:count";

    let children = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "asm",
            path,
            "--entity",
            component_id,
            "--child-kind",
            "method",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to filter ASM entity children");
    assert!(children.status.success());
    let children: serde_json::Value = serde_json::from_slice(&children.stdout).expect("child JSON");
    assert_eq!(
        children["children"],
        serde_json::json!([
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:increment",
            "module:fixtures/0001-source-summary/input/Counter.tsx/component:x-counter/method:render"
        ])
    );

    let relations = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "asm",
            path,
            "--entity",
            state_id,
            "--reference-kind",
            "action-state",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to filter ASM entity references");
    assert!(relations.status.success());
    let relations: serde_json::Value =
        serde_json::from_slice(&relations.stdout).expect("relation JSON");
    assert_eq!(
        relations["incoming_references"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(relations["incoming_references"][0]["kind"], "action-state");

    let unselected = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--child-kind", "method"])
        .output()
        .expect("failed to reject unselected ASM filters");
    assert!(!unselected.status.success());
    assert!(String::from_utf8_lossy(&unselected.stderr).contains("require an ASM entity selector"));
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
    assert_eq!(count["semantic_type"]["type_text"], "number");
    assert_eq!(count["semantic_type"]["status"], "declared");
    assert_eq!(count["semantic_type"]["provenance"]["path"], path);
    assert_eq!(
        count["semantic_type"]["origin"],
        "module:fixtures/0025-typed-state-annotations/input/TypedState.tsx/component:x-typed-state/state:count"
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
fn asm_command_covers_the_semantic_type_system_fixture_across_modules() {
    let root = repo_root();
    let paths = [
        "fixtures/0042-semantic-type-system/input/TypeSystem.tsx",
        "fixtures/0042-semantic-type-system/input/types.ts",
    ];
    let output = Command::new(ezc_cli_bin())
        .current_dir(&root)
        .args(["asm", paths[0], paths[1], "--format", "json"])
        .output()
        .expect("failed to inspect semantic type-system fixture");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");

    assert_eq!(document["entities"].as_array().map(Vec::len), Some(20));
    assert_eq!(
        document["entities"]
            .as_array()
            .expect("entities")
            .iter()
            .filter(|entity| entity["semantic_type"].is_object())
            .count(),
        10
    );
    assert!(document["entities"].as_array().is_some_and(|entities| {
        entities.iter().any(|entity| {
            entity["semantic_type"]["type_text"] == "\"active\" | \"all\" | \"completed\""
                && entity["semantic_type"]["status"] == "declared"
        })
    }));
    let diagnostics = document["diagnostics"].as_array().expect("diagnostics");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic["code"] == "EZC1031"));
    assert!(!diagnostics
        .iter()
        .any(|diagnostic| diagnostic["code"] == "EZC1032"));
}

#[test]
fn asm_command_exposes_constant_state_initializers() {
    let path = "fixtures/0032-arithmetic-state-initializer/input/ArithmeticState.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect arithmetic state");
    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output");
    let state = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities
                .iter()
                .find(|entity| entity["kind"] == "state-field")
        })
        .expect("state entity");
    assert_eq!(state["initial_expression"], "((1 + 2) * 3)");
}

#[test]
fn asm_command_exposes_constant_comparison_state_initializers() {
    let path = "fixtures/0033-constant-comparison-state-initializer/input/ComparisonState.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect comparison state");
    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output");
    let state = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities.iter().find(|entity| {
                entity["id"]
                    == "module:fixtures/0033-constant-comparison-state-initializer/input/ComparisonState.tsx/component:x-comparison-state/state:ready"
            })
        })
        .expect("ready state entity");
    assert_eq!(state["initial_expression"], "(((1 + 2) * 3) >= 9)");
}

#[test]
fn asm_command_exposes_constant_logical_state_initializers() {
    let path = "fixtures/0034-constant-logical-state-initializer/input/LogicalState.tsx";

    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect logical state");
    assert!(output.status.success());

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ASM inspection output");
    let state = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities.iter().find(|entity| {
                entity["id"]
                    == "module:fixtures/0034-constant-logical-state-initializer/input/LogicalState.tsx/component:x-logical-state/state:ready"
            })
        })
        .expect("ready state entity");
    assert_eq!(state["initial_expression"], "((1 < 2) && (3 >= 3))");
}

#[test]
fn asm_command_exposes_constant_nullish_state_initializers() {
    let path = "fixtures/0035-constant-nullish-state-initializer/input/NullishState.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect nullish state");
    assert!(output.status.success());

    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");
    let state = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities
                .iter()
                .find(|entity| entity["kind"] == "state-field")
        })
        .expect("state entity");
    assert_eq!(state["initial_expression"], "(null ?? \"fallback\")");
}

#[test]
fn asm_command_exposes_method_local_constants() {
    let path = "fixtures/0037-method-local-constants/input/LocalConstants.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect method locals");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");
    let method = document["entities"]
        .as_array()
        .and_then(|entities| entities.iter().find(|entity| entity["kind"] == "method"))
        .expect("render method entity");
    assert_eq!(
        method["local_variables"],
        serde_json::json!(["title = String(\"EdgeZero\")", "enabled = Boolean(true)"])
    );
}

#[test]
fn asm_command_exposes_constrained_method_parameters() {
    let path = "fixtures/0038-constrained-method-parameters/input/MethodParameters.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect method parameters");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");
    let method = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities.iter().find(|entity| {
                entity["kind"] == "method"
                    && entity["id"]
                        .as_str()
                        .is_some_and(|id| id.ends_with("/method:save"))
            })
        })
        .expect("save method entity");
    assert_eq!(
        method["parameters"],
        serde_json::json!([
            {
                "name": "title",
                "provenance": {
                    "path": path,
                    "start": 84,
                    "end": 97,
                    "line": 3,
                    "column": 8
                }
            },
            {
                "name": "retries",
                "provenance": {
                    "path": path,
                    "start": 99,
                    "end": 115,
                    "line": 3,
                    "column": 23
                }
            }
        ])
    );
}

#[test]
fn asm_command_resolves_supported_method_local_bindings() {
    let path = "fixtures/0039-method-local-resolution/input/LocalResolution.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect local-variable resolution");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");
    let local_id = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities.iter().find_map(|entity| {
                (entity["kind"] == "local-variable").then(|| entity["id"].as_str())?
            })
        })
        .expect("local variable entity");
    let references = document["references"].as_array().expect("ASM references");
    assert_eq!(
        references
            .iter()
            .filter(|reference| reference["kind"] == "template-local")
            .map(|reference| reference["target"].as_str())
            .collect::<Vec<_>>(),
        vec![Some(local_id), Some(local_id)]
    );

    let html = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["html", path])
        .output()
        .expect("failed to render local-variable fixture");
    assert!(html.status.success());
    assert!(String::from_utf8(html.stdout)
        .expect("HTML output")
        .contains("title=\"EdgeZero\""));
}

#[test]
fn asm_command_filters_template_computed_references() {
    let path = "fixtures/0043-template-computed-bindings/input/ComputedTemplate.tsx";
    let computed_id = "module:fixtures/0043-template-computed-bindings/input/ComputedTemplate.tsx/component:x-computed-template/computed:label";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "asm",
            path,
            "--entity",
            computed_id,
            "--reference-kind",
            "template-computed",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to filter template-computed references");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");
    let references = document["incoming_references"]
        .as_array()
        .expect("incoming computed references");

    assert_eq!(references.len(), 2);
    assert!(references.iter().all(|reference| {
        reference["kind"] == "template-computed" && reference["target"] == computed_id
    }));
    assert_eq!(
        document["outgoing_references"].as_array().map(Vec::len),
        Some(0)
    );
}

#[test]
fn immutable_constant_folding_fixture_reaches_html_backends() {
    let repo_root = repo_root();
    let input = "fixtures/0040-immutable-constant-folding/input/FoldedState.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", input])
        .output()
        .expect("failed to render folded constant fixture");
    assert!(output.status.success());
    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0040-immutable-constant-folding/expected/html.html"),
    )
    .expect("failed to read folded constant HTML expectation");
    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn asm_command_inspects_canonical_expression_graph_fixture() {
    let path = "fixtures/0041-canonical-expression-graph/input/ExpressionGraph.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect expression graph fixture");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ASM JSON");
    let state = document["entities"]
        .as_array()
        .and_then(|entities| {
            entities
                .iter()
                .find(|entity| entity["kind"] == "state-field")
        })
        .expect("state field entity");
    assert_eq!(state["initial_expression"], "((1 + 2) * 3)");
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

    assert_eq!(diagnostics.len(), 6);
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
        count["primary_provenance"]["path"],
        "fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx"
    );
    assert_eq!(count["primary_provenance"]["line"], 3);
    assert_eq!(count["primary_provenance"]["column"], 8);
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

    assert_eq!(diagnostics.len(), 7);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["code"] == "EZC1016"
            && diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `collection`"))
    }));
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic["code"] == "EZC1017")
            .count(),
        6
    );
    assert!(diagnostics
        .iter()
        .filter(|diagnostic| diagnostic["code"] == "EZC1017")
        .all(|diagnostic| diagnostic["message"]
            .as_str()
            .is_some_and(|message| message.contains("action `apply` assigns"))));

    let count = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `count`"))
        })
        .expect("count action mismatch diagnostic");
    assert_eq!(count["primary_provenance"]["path"], path);
    assert_eq!(count["primary_provenance"]["line"], 11);
    assert_eq!(count["primary_provenance"]["column"], 5);
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

    assert_eq!(diagnostics.len(), 4);
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
    assert_eq!(count["primary_provenance"]["path"], path);
    assert_eq!(count["primary_provenance"]["line"], 10);
    assert_eq!(count["primary_provenance"]["column"], 5);
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

    assert_eq!(diagnostics.len(), 4);
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
    assert_eq!(title["primary_provenance"]["path"], path);
    assert_eq!(title["primary_provenance"]["line"], 10);
    assert_eq!(title["primary_provenance"]["column"], 5);
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

    assert_eq!(diagnostics.len(), 7);
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
    assert_eq!(title["primary_provenance"]["path"], path);
    assert_eq!(title["primary_provenance"]["line"], 9);
    assert_eq!(title["primary_provenance"]["column"], 5);
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
    assert!(actual.contains("    error[EZC1020]: state field `title`"));
    assert!(actual.contains("    error[EZC1021]: action `apply`"));
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
    assert!(actual.contains("  compiler diagnostics: 7\n"));
    assert!(actual.contains("    error[EZC1020]: state field `title`"));
}

#[test]
fn check_command_emits_json_diagnostics() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["check", "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx", "--format", "json"])
        .output().expect("failed to run ezc_cli check");
    assert!(!output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    assert_eq!(document["schema_version"], 2);
    assert_eq!(
        document["compiler_diagnostics"].as_array().map(Vec::len),
        Some(7)
    );
    assert_eq!(document["fail_on"], "Error");
    let title_diagnostic = document["compiler_diagnostics"]
        .as_array()
        .expect("compiler diagnostics")
        .iter()
        .find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("state field `title`"))
        })
        .expect("title diagnostic");
    assert_eq!(
        title_diagnostic["primary_provenance"],
        serde_json::json!({
            "path": "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx",
            "start": 261,
            "end": 276,
            "line": 9,
            "column": 5,
        })
    );
}

#[test]
fn check_command_omits_unavailable_compiler_diagnostic_provenance_in_json() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args([
            "check",
            "fixtures/0003-semantic-errors/input/BrokenSemantics.tsx",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run ezc_cli check");

    assert!(!output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    let component_diagnostic = document["compiler_diagnostics"]
        .as_array()
        .expect("compiler diagnostics")
        .iter()
        .find(|diagnostic| diagnostic["code"] == "EZC1001")
        .expect("missing component diagnostic");
    assert!(component_diagnostic.get("provenance").is_none());
}

#[test]
fn effect_diagnostics_share_check_and_selected_explain_projection() {
    let path = "fixtures/0052-effect-body-lowering/input/Effects.tsx";
    let effect_id = "module:fixtures/0052-effect-body-lowering/input/Effects.tsx/component:x-effects/effect:invalid";
    let check = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["check", path, "--format", "json"])
        .output()
        .expect("failed to run effect check JSON");
    assert!(!check.status.success());
    let check: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check JSON");
    let expected = check["compiler_diagnostics"]
        .as_array()
        .expect("compiler diagnostics")
        .iter()
        .filter(|diagnostic| diagnostic["effect_id"] == effect_id)
        .cloned()
        .collect::<Vec<_>>();
    assert!(!expected.is_empty());
    assert!(expected.iter().all(|diagnostic| {
        diagnostic["severity"] == "error"
            && diagnostic["statement_id"].is_string()
            && diagnostic["primary_provenance"].is_object()
            && diagnostic["secondary_labels"].is_array()
    }));

    let explain = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["explain", path, "--entity", effect_id, "--format", "json"])
        .output()
        .expect("failed to run selected effect explain JSON");
    assert!(explain.status.success());
    let explain: serde_json::Value =
        serde_json::from_slice(&explain.stdout).expect("selected effect explain JSON");
    assert_eq!(explain["schema_version"], 4);
    assert_eq!(explain["diagnostics"], serde_json::Value::Array(expected));
}

#[test]
fn effect_fixture_matrix_covers_capabilities_dependencies_and_action_batch_deduplication() {
    let path = "fixtures/0054-effect-fixture-matrix/input/EffectFixtureMatrix.tsx";
    let first = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to inspect effect fixture matrix");
    assert!(first.status.success());
    let second = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", path, "--format", "json"])
        .output()
        .expect("failed to repeat effect fixture matrix inspection");
    assert_eq!(first.stdout, second.stdout);

    let document: serde_json::Value = serde_json::from_slice(&first.stdout).expect("matrix JSON");
    assert!(document["diagnostics"]
        .as_array()
        .is_some_and(Vec::is_empty));
    let component = format!("module:{path}/component:x-effect-fixture-matrix");
    let effect = |name: &str| {
        document["entities"]
            .as_array()
            .and_then(|entities| {
                entities
                    .iter()
                    .find(|entity| entity["id"] == format!("{component}/effect:{name}"))
            })
            .and_then(|entity| entity.get("effect"))
            .expect("effect inspection")
    };
    let increment_batch = format!("{component}/action-batch:incrementTwice");
    let rename_batch = format!("{component}/action-batch:rename");

    let bootstrap = effect("bootstrap");
    assert_eq!(
        bootstrap["direct_dependencies"]["state"],
        serde_json::json!([])
    );
    assert_eq!(
        bootstrap["direct_dependencies"]["computed"],
        serde_json::json!([])
    );
    assert_eq!(
        bootstrap["initial_trigger"]["policy"],
        "after_initial_render"
    );

    for name in ["persistTotal", "rememberTotal"] {
        let inspection = effect(name);
        assert_eq!(
            inspection["action_triggers"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            inspection["action_triggers"][0]["action_batch_id"],
            increment_batch
        );
        assert_eq!(inspection["action_triggers"][0]["effect_batch_index"], 0);
        assert_eq!(
            inspection["action_triggers"][0]["matched_states"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            inspection["action_triggers"][0]["required_computed"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
    }

    let title = effect("syncTitle");
    assert_eq!(title["action_triggers"][0]["action_batch_id"], rename_batch);
    assert_eq!(
        title["action_triggers"][0]["required_computed"],
        serde_json::json!([])
    );
    assert!(effect("persistTotal")["capabilities"]
        .as_array()
        .is_some_and(|operations| operations.iter().any(
            |operation| operation["operation_id"] == "builtin.browser.local_storage.set_item"
        )));
    assert!(effect("rememberTotal")["capabilities"]
        .as_array()
        .is_some_and(|operations| operations.iter().any(
            |operation| operation["operation_id"] == "builtin.browser.session_storage.set_item"
        )));
}

#[test]
fn multi_file_effect_fixture_keeps_module_qualified_effect_identity() {
    let alpha = "fixtures/0055-effect-multi-file-identity/input/AlphaEffect.tsx";
    let beta = "fixtures/0055-effect-multi-file-identity/input/BetaEffect.tsx";
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["asm", alpha, beta, "--format", "json"])
        .output()
        .expect("failed to inspect multi-file effect fixture");
    assert!(output.status.success());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("multi-file JSON");
    let ids = document["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .filter(|entity| entity["kind"] == "effect")
        .filter_map(|entity| entity["id"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids
        .iter()
        .any(|id| id.starts_with(&format!("module:{alpha}/"))));
    assert!(ids
        .iter()
        .any(|id| id.starts_with(&format!("module:{beta}/"))));
}

#[test]
fn check_command_filters_displayed_diagnostic_categories_without_changing_failure() {
    let output = Command::new(ezc_cli_bin())
        .current_dir(repo_root())
        .args(["check", "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx", "--format", "json", "--category", "parser"])
        .output().expect("failed to run ezc_cli check");
    assert!(!output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    assert_eq!(document["summary"]["compiler_diagnostics"], 7);
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
fn parse_command_matches_arithmetic_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "parse",
            "fixtures/0032-arithmetic-state-initializer/input/ArithmeticState.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli parse");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0032-arithmetic-state-initializer/expected/parse.txt"),
    )
    .expect("failed to read expected arithmetic state initializer parse fixture");

    assert_eq!(actual, expected);
}

#[test]
fn parse_command_matches_comparison_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "parse",
            "fixtures/0033-constant-comparison-state-initializer/input/ComparisonState.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli parse");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0033-constant-comparison-state-initializer/expected/parse.txt"),
    )
    .expect("failed to read expected comparison state initializer parse fixture");

    assert_eq!(actual, expected);
}

#[test]
fn parse_command_matches_logical_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "parse",
            "fixtures/0034-constant-logical-state-initializer/input/LogicalState.tsx",
        ])
        .output()
        .expect("failed to run ezc_cli parse");

    assert!(
        output.status.success(),
        "expected command to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0034-constant-logical-state-initializer/expected/parse.txt"),
    )
    .expect("failed to read expected logical state initializer parse fixture");

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
fn graph_command_matches_arithmetic_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "graph",
            "fixtures/0032-arithmetic-state-initializer/input/ArithmeticState.tsx",
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
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0032-arithmetic-state-initializer/expected/graph.txt"),
    )
    .expect("failed to read expected arithmetic state initializer graph fixture");

    assert_eq!(actual, expected);
}

#[test]
fn graph_command_matches_comparison_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "graph",
            "fixtures/0033-constant-comparison-state-initializer/input/ComparisonState.tsx",
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
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0033-constant-comparison-state-initializer/expected/graph.txt"),
    )
    .expect("failed to read expected comparison state initializer graph fixture");

    assert_eq!(actual, expected);
}

#[test]
fn graph_command_matches_logical_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "graph",
            "fixtures/0034-constant-logical-state-initializer/input/LogicalState.tsx",
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
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0034-constant-logical-state-initializer/expected/graph.txt"),
    )
    .expect("failed to read expected logical state initializer graph fixture");

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
fn html_command_matches_arithmetic_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0032-arithmetic-state-initializer/input/ArithmeticState.tsx",
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
        repo_root.join("fixtures/0032-arithmetic-state-initializer/expected/html.html"),
    )
    .expect("failed to read expected arithmetic state initializer HTML fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_comparison_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0033-constant-comparison-state-initializer/input/ComparisonState.tsx",
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
        repo_root.join("fixtures/0033-constant-comparison-state-initializer/expected/html.html"),
    )
    .expect("failed to read expected comparison state initializer HTML fixture");

    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}

#[test]
fn html_command_matches_logical_state_initializer_fixture() {
    let repo_root = repo_root();

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "html",
            "fixtures/0034-constant-logical-state-initializer/input/LogicalState.tsx",
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
        repo_root.join("fixtures/0034-constant-logical-state-initializer/expected/html.html"),
    )
    .expect("failed to read expected logical state initializer HTML fixture");

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
    assert!(actual_runtime.contains("SUPPORTED_SCHEMA_VERSION = 2"));
    assert!(actual_runtime.contains("RUNTIME_VERSION = \"0.0.0\""));
    assert!(actual_runtime.contains("validateManifestSchema"));
    assert!(actual_runtime.contains("EZR_UNSUPPORTED_SCHEMA"));
    assert!(actual_runtime.contains("diagnostics"));
    assert!(actual_runtime.contains("data-ez-node"));
    assert!(actual_runtime.contains("ez-binding:"));
    assert!(!actual_runtime.contains("normalizeHandlerReference"));
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
fn build_command_writes_compiler_generated_computed_runtime_metadata() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-test-output/computed-runtime-artifact");

    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir).expect("failed to clean previous test output");
    }

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0043-template-computed-bindings/input/ComputedTemplate.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build computed runtime artifact fixture");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(stdout.contains("computed.runtime.json"));

    let artifact = std::fs::read_to_string(out_dir.join("computed.runtime.json"))
        .expect("failed to read computed runtime artifact");
    let artifact: serde_json::Value = serde_json::from_str(&artifact).expect("artifact JSON");
    let component = "module:fixtures/0043-template-computed-bindings/input/ComputedTemplate.tsx/component:x-computed-template";
    let label = format!("{component}/computed:label");
    let visible = format!("{component}/computed:visible");

    assert_eq!(artifact["schema_version"], 3);
    assert_eq!(
        artifact["evaluation_order"],
        serde_json::json!([label, visible])
    );
    assert_eq!(
        artifact["update_batches"],
        serde_json::json!([[label, visible]])
    );
    assert_eq!(artifact["evaluations"].as_array().map(Vec::len), Some(2));
    assert!(artifact["evaluations"]
        .as_array()
        .is_some_and(|evaluations| {
            evaluations.iter().all(|evaluation| {
                evaluation["computed"] == evaluation["evaluation_function"]
                    && evaluation["dirty_flag"]["initial_value"] == true
                    && evaluation["serialization"] == "serializable"
                    && evaluation["dependencies"] == serde_json::json!([])
                    && evaluation["program"]["instructions"].is_array()
            })
        }));
}

#[test]
fn build_command_writes_compiler_generated_effect_runtime_metadata() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-test-output/effect-runtime-artifact");

    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir).expect("failed to clean previous test output");
    }

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0053-effect-initial-runtime/input/InitialEffectRuntime.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build effect runtime artifact fixture");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(stdout.contains("effect.runtime.json"));

    let page = std::fs::read_to_string(out_dir.join("index.html"))
        .expect("failed to read generated effect runtime page");
    assert!(page.contains("id=\"ez-effect-runtime\""));

    let manifest = std::fs::read_to_string(out_dir.join("template.manifest.json"))
        .expect("failed to read generated template manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("manifest JSON");
    assert_eq!(manifest["schema_version"], 2);
    assert_eq!(
        manifest["components"][0]["template"]["events"][0]["kind"],
        "action"
    );
    assert!(
        manifest["components"][0]["template"]["events"][0]["method_id"]
            .as_str()
            .is_some_and(|id| id.ends_with("/method:update"))
    );
    assert!(
        manifest["components"][0]["template"]["events"][0]["action_batch_id"]
            .as_str()
            .is_some_and(|id| id.ends_with("/action-batch:update"))
    );

    let artifact = std::fs::read_to_string(out_dir.join("effect.runtime.json"))
        .expect("failed to read effect runtime artifact");
    let artifact: serde_json::Value = serde_json::from_str(&artifact).expect("artifact JSON");
    assert_eq!(artifact["schema_version"], 1);
    assert_eq!(artifact["effects"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        artifact["effects"][0]["initial_trigger"]["effect_batch_index"],
        0
    );
    assert_eq!(
        artifact["effects"][0]["capability_operations"],
        serde_json::json!([
            {
                "operation": "builtin.browser.console.log",
                "runtime_lowering": "builtin.browser.console.log"
            },
            {
                "operation": "builtin.browser.document.title.assign",
                "runtime_lowering": "builtin.browser.document.title.assign"
            },
            {
                "operation": "builtin.browser.local_storage.set_item",
                "runtime_lowering": "builtin.browser.local_storage.set_item"
            }
        ])
    );
}

#[test]
fn build_command_writes_and_embeds_context_runtime_artifact() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-test-output/context-runtime-artifact");
    let input = out_dir.join("ContextRuntime.tsx");

    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir).expect("failed to clean previous test output");
    }
    std::fs::create_dir_all(&out_dir).expect("failed to create test output directory");
    std::fs::write(
        &input,
        r#"
@component("x-context-runtime")
class ContextRuntime extends Component {
  count = state(1);
  @context()
  total!: number;
  @provide(ContextRuntime.total)
  providedTotal: number = this.count + 2;
  @consume(ContextRuntime.total)
  total!: number;
  render() { return <main />; }
}
"#,
    )
    .expect("failed to write Context runtime input");

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("test input path was not valid UTF-8"),
            "--out",
            out_dir
                .to_str()
                .expect("test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build Context runtime artifact");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    assert!(stdout.contains("context.runtime.json"));
    let page = std::fs::read_to_string(out_dir.join("index.html"))
        .expect("failed to read generated Context runtime page");
    assert!(page.contains("id=\"ez-context-runtime\""));
    let artifact = std::fs::read_to_string(out_dir.join("context.runtime.json"))
        .expect("failed to read Context runtime artifact");
    let artifact: serde_json::Value = serde_json::from_str(&artifact).expect("artifact JSON");
    assert_eq!(artifact["schema_version"], 1);
    assert_eq!(artifact["sources"].as_array().map(Vec::len), Some(1));
    assert_eq!(artifact["consumers"].as_array().map(Vec::len), Some(1));
    assert!(artifact["sources"][0]["program"]["instructions"]
        .as_array()
        .is_some_and(|instructions| instructions
            .iter()
            .any(|instruction| { instruction["kind"] == "initialize_context_slot" })));
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

#[test]
fn unary_state_initializer_fixture_matches_parse_graph_and_html() {
    let repo_root = repo_root();
    let input = "fixtures/0036-constant-unary-state-initializer/input/UnaryState.tsx";

    for (command, expected) in [
        (
            "parse",
            "fixtures/0036-constant-unary-state-initializer/expected/parse.txt",
        ),
        (
            "graph",
            "fixtures/0036-constant-unary-state-initializer/expected/graph.txt",
        ),
    ] {
        let output = Command::new(ezc_cli_bin())
            .current_dir(&repo_root)
            .args([command, input])
            .output()
            .expect("failed to run unary fixture command");
        assert!(output.status.success());
        let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
        let expected = std::fs::read_to_string(repo_root.join(expected))
            .expect("failed to read unary fixture expectation");
        assert_eq!(actual, expected);
    }

    let output = Command::new(ezc_cli_bin())
        .current_dir(&repo_root)
        .args(["html", input])
        .output()
        .expect("failed to run unary fixture HTML command");
    assert!(output.status.success());
    let actual = String::from_utf8(output.stdout).expect("CLI stdout was not valid UTF-8");
    let expected = std::fs::read_to_string(
        repo_root.join("fixtures/0036-constant-unary-state-initializer/expected/html.html"),
    )
    .expect("failed to read unary fixture HTML expectation");
    assert_eq!(
        normalize_html_for_fixture(&actual),
        normalize_html_for_fixture(&expected)
    );
}
