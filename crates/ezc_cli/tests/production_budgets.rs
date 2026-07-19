use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    schema_version: u32,
    phase: String,
    fixtures: Vec<CorpusFixture>,
}

#[derive(Debug, Deserialize)]
struct CorpusFixture {
    name: String,
    kind: String,
    input: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Budgets {
    schema_version: u32,
    established_by: String,
    fixtures: Vec<Budget>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Budget {
    name: String,
    production_executable_bytes: u64,
    eager_bytes: u64,
    artifact_bytes: u64,
    runtime_record_count: u64,
    static_operation_count: u64,
    module_count: usize,
    phase_j_executable_bytes: Option<u64>,
    phase_j_eager_bytes: Option<u64>,
    lifecycle_cycles: Option<u32>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn documents() -> (Corpus, Budgets) {
    let corpus = serde_json::from_str(include_str!(
        "../../../fixtures/phase-k-benchmarks/corpus.json"
    ))
    .expect("K16 corpus JSON");
    let budgets = serde_json::from_str(include_str!(
        "../../../fixtures/phase-k-benchmarks/budgets.json"
    ))
    .expect("K16 budget JSON");
    (corpus, budgets)
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).expect("artifact metadata").len()
}

fn report_value(report: &serde_json::Value, key: &str) -> u64 {
    report[key]
        .as_u64()
        .unwrap_or_else(|| panic!("report field {key}"))
}

fn assert_fixture_budget(root: &Path, fixture: &CorpusFixture, budget: &Budget) {
    let Some(input) = &fixture.input else {
        assert_eq!(fixture.kind, "validator-rejection");
        return;
    };
    let output = root
        .join("target/ezc-test-output")
        .join(format!("phase-k-budget-{}", fixture.name));
    if output.exists() {
        std::fs::remove_dir_all(&output).expect("clean prior budget output");
    }
    let result = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(root)
        .args([
            "build",
            input,
            "--out",
            output.to_str().expect("output UTF-8"),
            "--production",
        ])
        .output()
        .expect("run production benchmark build");
    assert!(
        result.status.success(),
        "{} failed: {}",
        fixture.name,
        String::from_utf8_lossy(&result.stderr)
    );

    let mut modules = std::fs::read_dir(output.join("production"))
        .expect("production module directory")
        .map(|entry| entry.expect("module entry").path())
        .collect::<Vec<_>>();
    modules.sort();
    let executable_bytes = modules.iter().map(|path| file_size(path)).sum::<u64>();
    let eager = modules
        .iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("boot."))
        })
        .expect("eager production module");
    let artifact_bytes = file_size(&output.join("production.runtime.json"));
    let cost: serde_json::Value = serde_json::from_slice(
        &std::fs::read(output.join("runtime-cost-report.json")).expect("cost report"),
    )
    .expect("cost report JSON");
    let static_operations = [
        "estimatedColdInitOperationCount",
        "estimatedResumeRestoreOperationCount",
        "maxActionBatchOperationCount",
        "maxSchedulerBatchWidth",
        "maxDomPatchCountPerAction",
    ]
    .iter()
    .map(|key| report_value(&cost, key))
    .sum::<u64>();

    assert!(
        executable_bytes <= budget.production_executable_bytes,
        "{} executable bytes",
        fixture.name
    );
    assert!(
        file_size(eager) <= budget.eager_bytes,
        "{} eager bytes",
        fixture.name
    );
    assert!(
        artifact_bytes <= budget.artifact_bytes,
        "{} artifact bytes",
        fixture.name
    );
    assert!(
        report_value(&cost, "runtimeRecordCount") <= budget.runtime_record_count,
        "{} records",
        fixture.name
    );
    assert!(
        static_operations <= budget.static_operation_count,
        "{} operations",
        fixture.name
    );
    assert!(
        modules.len() <= budget.module_count,
        "{} module count",
        fixture.name
    );

    if fixture.kind == "static" {
        assert_eq!(modules.len(), 1, "static fixture must not emit lazy chunks");
        assert!(modules.iter().all(|path| path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("boot."))));
    }
    if fixture.kind == "phase-j-comparison" {
        let phase_j_total = budget.phase_j_executable_bytes.expect("Phase J total");
        let phase_j_eager = budget.phase_j_eager_bytes.expect("Phase J eager");
        assert!(executable_bytes < phase_j_total);
        assert!(file_size(eager) <= phase_j_eager + 128);
    }
}

#[test]
fn k16_corpus_is_complete_ordered_and_has_one_exact_budget_per_case() {
    let (corpus, budgets) = documents();
    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.phase, "K16");
    assert_eq!(budgets.schema_version, 1);
    assert_eq!(budgets.established_by, "K16");
    assert_eq!(corpus.fixtures.len(), 16);
    assert_eq!(budgets.fixtures.len(), 16);
    assert_eq!(
        corpus
            .fixtures
            .iter()
            .map(|fixture| fixture.name.as_str())
            .collect::<Vec<_>>(),
        budgets
            .fixtures
            .iter()
            .map(|fixture| fixture.name.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        corpus
            .fixtures
            .iter()
            .filter(|fixture| fixture.kind == "validator-rejection")
            .count(),
        1
    );
    assert_eq!(
        budgets
            .fixtures
            .iter()
            .find(|fixture| fixture.name == "create-destroy-lifecycle")
            .and_then(|fixture| fixture.lifecycle_cycles),
        Some(100)
    );
}

#[test]
fn k16_production_outputs_do_not_exceed_committed_static_budgets() {
    let root = repo_root();
    let (corpus, budgets) = documents();
    let corpus_by_name = corpus
        .fixtures
        .iter()
        .map(|fixture| (fixture.name.as_str(), fixture))
        .collect::<BTreeMap<_, _>>();

    for budget in &budgets.fixtures {
        let fixture = corpus_by_name[budget.name.as_str()];
        assert_fixture_budget(&root, fixture, budget);
    }
}
