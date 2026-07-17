use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BaselineDocument {
    phase: String,
    fixtures: Vec<FixtureBaseline>,
}

#[derive(Debug, Deserialize)]
struct FixtureBaseline {
    name: String,
    input: String,
    artifacts: BTreeMap<String, u64>,
    resume: ResumeBaseline,
}

#[derive(Debug, Deserialize)]
struct ResumeBaseline {
    boundaries: usize,
    slot_schemas: usize,
    capture_programs: usize,
    restore_programs: usize,
    anchors: usize,
    events: usize,
    activations: usize,
    root_kinds: Vec<String>,
    root_ids: Vec<String>,
    provided_program_ids: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve repository root")
}

fn baseline() -> BaselineDocument {
    serde_json::from_str(include_str!(
        "../../../fixtures/phase-k-production-baseline.json"
    ))
    .expect("Phase K baseline fixture should be valid JSON")
}

fn logical_artifact_name(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .expect("artifact name should be UTF-8");
    if name.starts_with("boot.application.")
        && Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
    {
        "resume.eager.module.js".to_string()
    } else {
        name.to_string()
    }
}

fn build_fixture(repo_root: &Path, fixture: &FixtureBaseline, suffix: &str) -> PathBuf {
    let output = repo_root
        .join("target/ezc-test-output")
        .join(format!("phase-k-{}-{suffix}", fixture.name));
    if output.exists() {
        std::fs::remove_dir_all(&output).expect("failed to clean prior Phase K output");
    }
    let result = Command::new(env!("CARGO_BIN_EXE_ezc_cli"))
        .current_dir(repo_root)
        .args([
            "build",
            &fixture.input,
            "--out",
            output.to_str().expect("output path should be UTF-8"),
        ])
        .output()
        .expect("failed to run Phase K baseline build");
    assert!(
        result.status.success(),
        "baseline build failed for {}: {}",
        fixture.name,
        String::from_utf8_lossy(&result.stderr)
    );
    output
}

fn artifact_sizes(output: &Path) -> BTreeMap<String, u64> {
    let mut actual = BTreeMap::new();
    for entry in std::fs::read_dir(output).expect("baseline output should be readable") {
        let path = entry.expect("output entry should be readable").path();
        if path.is_file() {
            actual.insert(
                logical_artifact_name(&path),
                std::fs::metadata(path)
                    .expect("artifact metadata should be readable")
                    .len(),
            );
        }
    }
    actual
}

fn values(document: &serde_json::Value, key: &str) -> usize {
    document[key]
        .as_array()
        .unwrap_or_else(|| panic!("resume field {key} should be an array"))
        .len()
}

fn assert_resume_matches_baseline(resume: &serde_json::Value, fixture: &FixtureBaseline) {
    assert_eq!(values(resume, "boundaries"), fixture.resume.boundaries);
    assert_eq!(values(resume, "slot_schemas"), fixture.resume.slot_schemas);
    assert_eq!(
        values(resume, "capture_programs"),
        fixture.resume.capture_programs
    );
    assert_eq!(
        values(resume, "restore_programs"),
        fixture.resume.restore_programs
    );
    assert_eq!(values(resume, "anchors"), fixture.resume.anchors);
    assert_eq!(values(resume, "events"), fixture.resume.events);
    assert_eq!(values(resume, "activations"), fixture.resume.activations);
    let chunks = resume["chunks"]
        .as_array()
        .expect("chunks should be an array");
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| chunk["root_kind"].as_str().expect("root kind"))
            .collect::<Vec<_>>(),
        fixture.resume.root_kinds
    );
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| chunk["root_id"].as_str().expect("root ID"))
            .collect::<Vec<_>>(),
        fixture.resume.root_ids
    );
    let mut programs = chunks
        .iter()
        .flat_map(|chunk| {
            chunk["provided_program_ids"]
                .as_array()
                .expect("provided programs")
                .iter()
                .map(|program| program.as_str().expect("program ID").to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    programs.sort();
    assert_eq!(programs, fixture.resume.provided_program_ids);
    assert!(chunks.iter().all(|chunk| chunk["dependency_chunk_ids"]
        .as_array()
        .is_some_and(Vec::is_empty)));
}

#[test]
fn k8_production_artifact_is_deterministic_and_preserves_the_k0_baseline() {
    let repo_root = repo_root();
    let baseline = baseline();
    assert_eq!(baseline.phase, "K0");
    for fixture in &baseline.fixtures {
        let first = build_fixture(&repo_root, fixture, "first");
        let second = build_fixture(&repo_root, fixture, "second");
        let mut first_artifacts = artifact_sizes(&first);
        let production_size = first_artifacts
            .remove("production.runtime.json")
            .expect("K8 production artifact should be emitted");
        assert!(production_size > 0, "production artifact must not be empty");
        assert_eq!(
            first_artifacts, fixture.artifacts,
            "{} byte baseline",
            fixture.name
        );
        let mut second_artifacts = artifact_sizes(&second);
        assert_eq!(
            second_artifacts.remove("production.runtime.json"),
            Some(production_size),
            "{} repeated production artifact size",
            fixture.name
        );
        assert_eq!(
            first_artifacts, second_artifacts,
            "{} repeated build",
            fixture.name
        );
        for forbidden in ["optimization-report.json", "runtime-cost-report.json"] {
            assert!(
                !first.join(forbidden).exists(),
                "K0 must not emit {forbidden}"
            );
        }
        let resume: serde_json::Value = serde_json::from_slice(
            &std::fs::read(first.join("resume.runtime.json"))
                .expect("resume manifest should be emitted"),
        )
        .expect("resume manifest should be valid JSON");
        let production: serde_json::Value = serde_json::from_slice(
            &std::fs::read(first.join("production.runtime.json"))
                .expect("production artifact should be emitted"),
        )
        .expect("production artifact should be valid JSON");
        assert_eq!(production["schemaVersion"], 1);
        assert_eq!(production["buildId"], resume["build_id"]);
        assert_eq!(production["runtimeProtocolVersion"], 1);
        assert_eq!(
            std::fs::read(first.join("production.runtime.json"))
                .expect("first production artifact bytes"),
            std::fs::read(second.join("production.runtime.json"))
                .expect("second production artifact bytes"),
            "{} production artifact bytes",
            fixture.name
        );
        assert_resume_matches_baseline(&resume, fixture);
    }
}
