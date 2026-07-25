use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepresentativeCorpus {
    schema_version: u32,
    suite: String,
    applications: Vec<RepresentativeApplication>,
}

#[derive(Deserialize)]
struct RepresentativeApplication {
    name: String,
    root: String,
    evidence: Vec<String>,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(1);

fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("representative destination directory");
    for entry in std::fs::read_dir(source).expect("representative source directory") {
        let entry = entry.expect("representative source entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("representative entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy representative source file");
        }
    }
}

fn find_artifact(root: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.file_name().is_some_and(|file_name| file_name == name) {
            return Some(path);
        }
        if entry.file_type().ok()?.is_dir() {
            if let Some(path) = find_artifact(&path, name) {
                return Some(path);
            }
        }
    }
    None
}

#[test]
fn representative_applications_publish_routes_resume_and_production_audits() {
    let root = repository_root();
    let corpus: RepresentativeCorpus = serde_json::from_str(include_str!(
        "../../../fixtures/representative-applications.json"
    ))
    .expect("representative corpus JSON");
    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.suite, "presolve-representative-applications");
    assert_eq!(corpus.applications.len(), 2);
    for application in corpus.applications {
        assert!(application.evidence.contains(&"production-build".to_string()));
        let temporary_root = std::env::temp_dir().join(format!(
            "presolve-representative-{}-{}-{}",
            application.name,
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_tree(&root.join(application.root), &temporary_root);
        let output = temporary_root.join("dist");
        let result = Command::new(env!("CARGO_BIN_EXE_presolve"))
            .current_dir(&temporary_root)
            .arg("build")
            .output()
            .expect("representative application build");
        assert!(
            result.status.success(),
            "{} build failed: {}",
            application.name,
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(find_artifact(&output, "resume.runtime.json").is_some());
        let audit_path = find_artifact(&output, "production-audit.json")
            .expect("route-scoped production audit");
        let audit: serde_json::Value = serde_json::from_slice(
            &std::fs::read(audit_path).expect("production audit"),
        )
        .expect("production audit JSON");
        assert_eq!(audit["status"], "passed");
        std::fs::remove_dir_all(temporary_root).expect("remove representative project");
    }
}
