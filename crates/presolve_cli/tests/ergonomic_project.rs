use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

fn project_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "presolve-ergonomic-project-{label}-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("app/routes")).unwrap();
    root
}

#[test]
fn default_build_discovers_an_imported_semantic_package_contract() {
    let root = project_root("package");
    let package = root.join("node_modules/@acme/analytics");
    fs::create_dir_all(package.join("dist")).unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"
import { trackPurchase } from "@acme/analytics";

@component()
class Home extends Component {
  @action() @opaque("@acme/analytics", "trackPurchase")
  track(): void {}

  render() { return <button onClick={this.track}>Buy</button>; }
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        r#"{"schema_version":1,"package":"@acme/analytics","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"trackPurchase":{"kind":"opaque","type_signature":"() -> void","runtime_module":"dist/track-purchase.js","resume_policy":"cold_fallback","opaque_terminal":{"execution_boundary":"client","resume":"cold_fallback"}}}}"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/track-purchase.js"),
        "export function trackPurchase() {}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .current_dir(&root)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("dist/index.html").is_file());
    let opaque = fs::read_to_string(root.join("dist/opaque.runtime.json")).unwrap();
    assert!(opaque.contains("@acme/analytics"));
    assert!(opaque.contains("dist/track-purchase.js"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn default_check_reports_file_route_pattern_conflicts() {
    let root = project_root("route-conflict");
    fs::create_dir_all(root.join("app/routes/posts")).unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main />; } }"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/posts/[id].tsx"),
        r#"@component() class ById extends Component { render() { return <article />; } }"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/posts/[slug].tsx"),
        r#"@component() class BySlug extends Component { render() { return <article />; } }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("PSROUTE1013_FILE_ROUTE_CONFLICT"));
    fs::remove_dir_all(root).unwrap();
}
