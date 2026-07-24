use std::fs;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

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
    assert!(root.join("dist/routes/root/index.html").is_file());
    let opaque = fs::read_to_string(root.join("dist/routes/root/opaque.runtime.json")).unwrap();
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

#[test]
fn dev_once_builds_a_default_project_without_configuration() {
    let root = project_root("dev");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Home</main>; } }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["dev", "--once"])
        .current_dir(&root)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("dist/routes/root/index.html").is_file());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Built"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn default_build_composes_a_conventional_layout_without_framework_wrapping() {
    let root = project_root("layout");
    fs::write(
        root.join("app/layout.tsx"),
        r#"
@component() class AppLayout extends Component {
  @slot() children!: SlotContent;
  render() { return <main><slot /></main>; }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <article>Home</article>; } }"#,
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
    let html = fs::read_to_string(root.join("dist/routes/root/index.html")).unwrap();
    assert!(html.contains("<main"));
    assert!(html.contains("<article"));
    assert!(html.contains("Home"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn default_check_and_build_publish_a_compiler_route_loader_handoff() {
    let root = project_root("route-loader");
    let package = root.join("node_modules/post-service");
    fs::create_dir_all(package.join("dist")).unwrap();
    fs::create_dir_all(root.join("app/routes/posts")).unwrap();
    fs::write(
        root.join("app/routes/posts/[slug].tsx"),
        r#"
import { loadPost } from "post-service";
@component() class Post {
  @loader("loadPost") post!: Resource<Post, NotFound>;
  render() { return <article />; }
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        r#"{"schema_version":1,"package":"post-service","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadPost":{"kind":"resource","type_signature":"RouteParameters -> Resource<Post, NotFound>","runtime_module":"dist/load-post.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"public","max_age_seconds":60},"failure":"typed"}}}}"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/load-post.js"),
        "export const loadPost = () => {};\n",
    )
    .unwrap();

    let check = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    let build = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let plan = fs::read_to_string(root.join("dist/route-loaders.plan.json")).unwrap();
    assert!(plan.contains("post-service"));
    assert!(plan.contains("route_parameters"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn default_check_and_build_publish_a_compiler_route_server_action_handoff() {
    let root = project_root("route-server-action");
    let package = root.join("node_modules/post-service");
    fs::create_dir_all(package.join("dist")).unwrap();
    fs::create_dir_all(root.join("app/routes/posts")).unwrap();
    fs::write(
        root.join("app/routes/posts/[slug].tsx"),
        r#"
import { savePost } from "post-service";
@component() class Post {
  @action() @serverAction("savePost") save(): void {}
  render() { return <form />; }
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        r#"{"schema_version":1,"package":"post-service","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"savePost":{"kind":"server_action","type_signature":"FormData -> ServerActionResult","runtime_module":"dist/save-post.js","resume_policy":"cold_fallback","server_action":{"input":"form_data","response":"json","failure":"typed"}}}}"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/save-post.js"),
        "export const savePost = () => {};\n",
    )
    .unwrap();

    let check = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    let build = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let plan = fs::read_to_string(root.join("dist/route-server-actions.plan.json")).unwrap();
    assert!(plan.contains("post-service"));
    assert!(plan.contains("form_data"));
    assert!(plan.contains("cold_fallback"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deploy_prepare_projects_compiler_artifacts_to_cloudflare_workers_static_assets() {
    let root = project_root("cloudflare");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Presolve</main>; } }"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/about.tsx"),
        r#"@component() class About extends Component { render() { return <main>About</main>; } }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "deploy",
            "cloudflare",
            "--prepare",
            "--name",
            "presolve-docs",
            "--secret",
            "DOCS_TOKEN",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let adapter = root.join(".presolve/cloudflare");
    let plan = fs::read_to_string(adapter.join("deployment.plan.json")).unwrap();
    let worker = fs::read_to_string(adapter.join("worker.mjs")).unwrap();
    let config = fs::read_to_string(adapter.join("wrangler.jsonc")).unwrap();
    assert!(plan.contains("cloudflare_workers_static_assets"));
    assert!(plan.contains("routes/segment-about/index.html"));
    assert!(worker.contains("routeFor"));
    assert!(config.contains("run_worker_first"));
    assert!(config.contains("DOCS_TOKEN"));
    let syntax = Command::new("node")
        .arg("--check")
        .arg(adapter.join("worker.mjs"))
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "worker syntax stderr: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn deploy_rejects_rollback_mixed_with_preparation_options() {
    let root = project_root("cloudflare-rollback-arguments");
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "cloudflare", "--rollback", "--prepare"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("PSCFL1011_DEPLOY_ARGUMENT_INVALID"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explain_projects_compiler_route_and_prepared_deployment_facts() {
    let root = project_root("explain");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Home</main>; } }"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/about.tsx"),
        r#"@component() class About extends Component { render() { return <main>About</main>; } }"#,
    )
    .unwrap();
    let route = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["explain", "route"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(route.status.success());
    let route = String::from_utf8(route.stdout).unwrap();
    assert!(route.contains("Routes"));
    assert!(route.contains("/about"));
    let prepare = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "deploy",
            "cloudflare",
            "--prepare",
            "--name",
            "presolve-docs",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(prepare.status.success());
    let deployment = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["explain", "deployment"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(deployment.status.success());
    let deployment = String::from_utf8(deployment.stdout).unwrap();
    assert!(deployment.contains("Cloudflare deployment"));
    assert!(deployment.contains("presolve-docs"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dev_serves_the_compiler_published_page() {
    let root = project_root("server");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Served</main>; } }"#,
    )
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut child = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["dev", "--port", &port.to_string()])
        .current_dir(&root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut response = Vec::new();
    for _ in 0..120 {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
            stream.read_to_end(&mut response).unwrap();
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    child.kill().unwrap();
    child.wait().unwrap();

    let response = String::from_utf8(response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("Served"));
    fs::remove_dir_all(root).unwrap();
}
