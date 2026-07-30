use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

fn setup() -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "presolve-application-publication-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    let config = root.join("presolve.json");
    fs::write(
        &config,
        r#"{"source_roots":["src"],"feature_flags":[],"target_profile":"default","platform_options":[]}"#,
    )
    .unwrap();
    fs::write(root.join("src/Helper.ts"), "const value = 1;\n").unwrap();
    fs::write(
        root.join("src/App.tsx"),
        r#"@component("x-app") class App extends Component { render() { return <main>App</main>; } }"#,
    )
    .unwrap();
    let output = root.join("dist");
    (root, config, output)
}

fn application_build(config: &Path, output: &Path, entry: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "application",
            "build",
            "--config",
            config.to_str().unwrap(),
            "--source",
            "src/App.tsx=src/App.tsx",
            "--source",
            "src/Helper.ts=src/Helper.ts",
            "--entry",
            entry,
            "--out",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap()
}

fn publication_stages(output: &Path) -> Vec<PathBuf> {
    let output_name = output.file_name().unwrap().to_string_lossy();
    let prefix = format!(".{output_name}.presolve-release-");
    let mut stages = fs::read_dir(output.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                return false;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Some(suffix) = name.strip_prefix(&prefix) else {
                return false;
            };
            let Some((process_id, sequence)) = suffix.split_once('-') else {
                return false;
            };
            !process_id.is_empty()
                && process_id.bytes().all(|byte| byte.is_ascii_digit())
                && !sequence.is_empty()
                && sequence.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    stages.sort();
    stages
}

#[test]
fn application_build_publishes_a_complete_multi_source_product_through_an_atomic_pointer() {
    let (root, config, output) = setup();
    let obsolete_release = root.join(".dist.presolve-release-999-999");
    let caller_directory = root.join(".dist.presolve-release-not-a-stage");
    fs::create_dir_all(&obsolete_release).unwrap();
    fs::create_dir_all(&caller_directory).unwrap();
    let first = application_build(&config, &output, "src/App.tsx");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(fs::symlink_metadata(&output)
        .unwrap()
        .file_type()
        .is_symlink());
    let first_release = fs::read_link(&output).unwrap();
    assert!(output.join("index.html").is_file());
    assert!(output.join("application.manifest.json").is_file());
    let manifest = fs::read_to_string(output.join("application.manifest.json")).unwrap();
    assert!(manifest.contains("presolve-application-publication:1"));
    assert!(
        !obsolete_release.exists(),
        "a stale exact Presolve release directory must be retired"
    );
    assert!(
        caller_directory.is_dir(),
        "publication cleanup must not remove a caller directory with a similar name"
    );

    let second = application_build(&config, &output, "src/App.tsx");
    assert!(second.status.success());
    let second_release = fs::read_link(&output).unwrap();
    assert_ne!(first_release, second_release);
    assert_eq!(
        publication_stages(&output),
        vec![output.parent().unwrap().join(second_release)],
        "a successful atomic publication must retain only its active release"
    );

    let failed = application_build(&config, &output, "src/Missing.tsx");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("PSAPP1004_ENTRY_NOT_IN_SOURCE_SET"));
    assert!(output.join("index.html").is_file());
    assert_eq!(
        publication_stages(&output).len(),
        1,
        "a failed build must preserve the one active release"
    );
    fs::remove_dir_all(root).unwrap();
}
