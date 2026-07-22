use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use ezc_core::{
    parse_production_runtime_artifact_v1, ProductionArtifactIntegrityViolation, ResumeBuildId,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn build(root: &Path, input: &str, name: &str, production: bool) -> PathBuf {
    let output = root.join("target/ezc-test-output").join(name);
    if output.exists() {
        std::fs::remove_dir_all(&output).expect("clean K19 output");
    }
    let mut arguments = vec![
        "build".to_string(),
        input.to_string(),
        "--out".to_string(),
        output.to_string_lossy().into_owned(),
    ];
    if production {
        arguments.push("--production".to_string());
    }
    let result = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .current_dir(root)
        .args(arguments)
        .output()
        .expect("run K19 fixture build");
    assert!(
        result.status.success(),
        "K19 build failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}

fn without_production_artifact(html: &str) -> String {
    let marker = "    <script type=\"application/json\" id=\"presolve-production-runtime\">";
    let Some(start) = html.find(marker) else {
        return html.to_string();
    };
    let relative_end = html[start..]
        .find("</script>\n")
        .expect("production artifact closing script");
    let end = start + relative_end + "</script>\n".len();
    format!("{}{}", &html[..start], &html[end..])
}

fn files_in(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(path).expect("read output directory") {
        let entry = entry.expect("output entry");
        let child = entry.path();
        if child.is_dir() {
            for (name, bytes) in files_in(&child) {
                files.insert(
                    format!("{}/{name}", entry.file_name().to_string_lossy()),
                    bytes,
                );
            }
        } else {
            files.insert(
                entry.file_name().to_string_lossy().into_owned(),
                std::fs::read(child).expect("read output file"),
            );
        }
    }
    files
}

#[test]
fn k19_development_and_production_builds_are_observably_equivalent_and_deterministic() {
    let root = repo_root();
    let input = "fixtures/0059-context-runtime-matrix/input/ContextRuntimeMatrix.tsx";
    let development = build(&root, input, "k19-development", false);
    let first = build(&root, input, "k19-production-first", true);
    let second = build(&root, input, "k19-production-second", true);

    for artifact in [
        "template.manifest.json",
        "computed.runtime.json",
        "context.runtime.json",
        "effect.runtime.json",
        "component.runtime.json",
        "forms.runtime.json",
        "resume.runtime.json",
        "production.runtime.json",
        "runtime-cost-report.json",
        "runtime.js",
    ] {
        assert_eq!(
            std::fs::read(development.join(artifact)).expect("development artifact"),
            std::fs::read(first.join(artifact)).expect("production artifact"),
            "{artifact} parity"
        );
    }
    assert_eq!(
        files_in(&first.join("production")),
        files_in(&second.join("production"))
    );
    assert_eq!(
        std::fs::read(first.join("production.runtime.json")).expect("first packed artifact"),
        std::fs::read(second.join("production.runtime.json")).expect("second packed artifact")
    );
    assert_eq!(
        std::fs::read(first.join("optimization-report.json")).expect("first optimization report"),
        std::fs::read(second.join("optimization-report.json")).expect("second optimization report")
    );
    assert_eq!(
        std::fs::read(first.join("runtime-cost-report.json")).expect("first cost report"),
        std::fs::read(second.join("runtime-cost-report.json")).expect("second cost report")
    );
    let development_html =
        std::fs::read_to_string(development.join("index.html")).expect("development HTML");
    let production_html =
        std::fs::read_to_string(first.join("index.html")).expect("production HTML");
    assert_eq!(
        without_production_artifact(&production_html),
        development_html
    );
    assert!(production_html.contains("id=\"presolve-production-runtime\""));
    let mut development_report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(development.join("optimization-report.json"))
            .expect("development optimization report"),
    )
    .expect("development optimization JSON");
    let mut production_report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(first.join("optimization-report.json"))
            .expect("production optimization report"),
    )
    .expect("production optimization JSON");
    let development_bytes = development_report
        .as_object_mut()
        .expect("development report object")
        .remove("developmentBytes")
        .expect("development byte evidence");
    let production_bytes = production_report
        .as_object_mut()
        .expect("production report object")
        .remove("developmentBytes")
        .expect("production byte evidence");
    assert_ne!(development_bytes, production_bytes);
    assert_eq!(development_report, production_report);

    for (name, bytes) in files_in(&first.join("production")) {
        let source = String::from_utf8(bytes).expect("generated module UTF-8");
        assert!(!source.contains("eval("), "{name}");
        assert!(!source.contains("Function("), "{name}");
        assert!(!source.contains("import("), "{name}");
        assert!(!source.contains("//# sourceMappingURL"), "{name}");
        assert!(Command::new("node")
            .args([
                "--check",
                first
                    .join("production")
                    .join(name)
                    .to_str()
                    .expect("module path UTF-8")
            ])
            .status()
            .expect("node syntax check")
            .success());
    }
}

#[test]
fn k19_packed_tables_are_dense_and_malformed_output_rejects_before_execution() {
    let root = repo_root();
    let output = build(
        &root,
        "fixtures/0065-component-runtime/input/RuntimeComponents.tsx",
        "k19-packed-validation",
        true,
    );
    let bytes = std::fs::read_to_string(output.join("production.runtime.json"))
        .expect("packed production artifact");
    let document: serde_json::Value = serde_json::from_str(&bytes).expect("production JSON");
    let build_id = ResumeBuildId::from_str(document["buildId"].as_str().expect("build ID"))
        .expect("canonical build ID");
    let artifact =
        parse_production_runtime_artifact_v1(&bytes, &build_id).expect("valid generated artifact");
    for table in &artifact.tables.tables {
        assert_eq!(table.count as usize, table.mappings.len());
        assert_eq!(
            table
                .mappings
                .iter()
                .map(|mapping| mapping.ordinal)
                .collect::<Vec<_>>(),
            (0..u32::try_from(table.mappings.len()).expect("fixture table length"))
                .collect::<Vec<_>>()
        );
    }

    let malformed = bytes.replacen("\"schemaVersion\":1", "\"schemaVersion\":2", 1);
    let errors = parse_production_runtime_artifact_v1(&malformed, &build_id)
        .expect_err("schema mutation must reject");
    assert!(errors.contains(&ProductionArtifactIntegrityViolation::SchemaVersionMismatch));
}
