use std::fs;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use sha2::{Digest as _, Sha256};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

fn publication_stage_count(root: &PathBuf) -> usize {
    let prefix = ".dist.presolve-release-";
    fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|kind| kind.is_dir())
                && entry.file_name().to_string_lossy().starts_with(prefix)
        })
        .count()
}

fn development_get(port: u16, path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .ok()?;
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    Some(response)
}

fn raw_http_request(port: u16, request: &str) -> String {
    for _ in 0..120 {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream.write_all(request.as_bytes()).unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).unwrap();
            return String::from_utf8(response).unwrap();
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("Node host did not accept a request on port {port}");
}

#[cfg(unix)]
fn chrome_bin() -> PathBuf {
    if let Some(path) = std::env::var_os("PRESOLVE_CHROME") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return path;
        }
    }
    [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/opt/google/chrome/chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
    .expect("headless Chrome was not found")
}

#[cfg(unix)]
fn run_chrome_with_timeout(chrome: PathBuf, arguments: &[String], timeout: Duration) -> Output {
    let mut child = Command::new(chrome)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run headless Chrome");
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let stdout_output = Arc::new(Mutex::new(Vec::new()));
    let stdout_buffer = Arc::clone(&stdout_output);
    let stdout_reader = thread::spawn(move || {
        let mut chunk = [0_u8; 16 * 1024];
        while let Ok(count) = stdout.read(&mut chunk) {
            if count == 0 {
                break;
            }
            stdout_buffer
                .lock()
                .unwrap()
                .extend_from_slice(&chunk[..count]);
        }
    });
    let stderr_output = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::clone(&stderr_output);
    let stderr_reader = thread::spawn(move || {
        let mut chunk = [0_u8; 16 * 1024];
        while let Ok(count) = stderr.read(&mut chunk) {
            if count == 0 {
                break;
            }
            stderr_buffer
                .lock()
                .unwrap()
                .extend_from_slice(&chunk[..count]);
        }
    });
    let started = Instant::now();
    let status = loop {
        if stdout_output
            .lock()
            .unwrap()
            .windows(b"</html>".len())
            .any(|window| window == b"</html>")
        {
            child.kill().unwrap();
            break child.wait().unwrap();
        }
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > timeout {
            child.kill().unwrap();
            break child.wait().unwrap();
        }
        thread::sleep(Duration::from_millis(50));
    };
    stdout_reader.join().unwrap();
    stderr_reader.join().unwrap();
    let stdout = stdout_output.lock().unwrap().clone();
    let stderr = stderr_output.lock().unwrap().clone();
    Output {
        status,
        stdout,
        stderr,
    }
}

#[cfg(unix)]
#[test]
fn decorator_free_v2_source_uses_installed_authority_for_file_route_assembly() {
    let root = project_root("v2-authority");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"import { Component, state, action, environment } from "presolve";
const applicationName = environment.public("PRESOLVE_PUBLIC_NAME");
export class Home extends Component {
  count = state(0);
  increment = action(() => { this.count += 1; });
  get doubled() { return this.count * 2; }
  render() { return <button onClick={() => this.increment()}>Home: {this.doubled}</button>; }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("environment.manifest.json"),
        r#"{"schemaVersion":1,"sourceLabel":".env","browserValues":{"PRESOLVE_PUBLIC_NAME":"Presolve"},"serverValueNames":[]}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"noEmit":true}}"#,
    )
    .unwrap();
    let executable = root.join("node_modules/.bin/presolve-typescript-authority");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        r#"#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
const request = JSON.parse(readFileSync(0, "utf8"));
writeFileSync("authority-ran", "yes");
const identity = name => ({ name, flags: 32, declarationModules: ["presolve"] });
process.stdout.write(JSON.stringify({
  schemaVersion: 13,
  diagnostics: [],
  components: request.components.map(site => ({ id: site.id, identity: identity("Component") })),
  states: request.states.map(site => ({ id: site.id, identity: identity("state") })),
  actions: request.actions.map(site => ({ id: site.id, identity: identity("action") })),
  effects: request.effects.map(site => ({ id: site.id, identity: identity("effect") })),
  environmentPublic: request.environmentPublic.slice(0, 1).map(site => ({ id: site.id, identity: identity("public") })),
}));
"#,
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();

    let missing_manifest = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!missing_manifest.status.success());
    assert!(
        String::from_utf8_lossy(&missing_manifest.stderr).contains("PSENV1102_MANIFEST_REQUIRED")
    );

    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .arg("--environment-manifest")
        .arg("environment.manifest.json")
        .current_dir(&root)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("authority-ran").is_file());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Checked 1 source file(s)"));
    let build = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .arg("--environment-manifest")
        .arg("environment.manifest.json")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(root.join("dist/routes/root/index.html").is_file());
    let environment = fs::read_to_string(root.join("dist/environment.browser.json")).unwrap();
    assert!(environment.contains("PRESOLVE_PUBLIC_NAME"));
    assert!(!environment.contains("DATABASE_URL"));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn decorator_free_v2_layout_slot_compiles_through_the_real_authority_bridge() {
    let root = project_root("v2-layout-slot");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let framework_types = repository.join("framework/packages/presolve/src/index.d.ts");
    let authority_module = repository.join("packages/typescript-authority/src/index.js");
    fs::write(
        root.join("app/layout.tsx"),
        r#"import { Component, slot, type SlotContent } from "presolve";

export class DocsLayout extends Component {
  children: SlotContent = slot();
  render() { return <main><slot /></main>; }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"import { Component } from "presolve";

export class DocsHome extends Component {
  render() { return <article>Decorator-free docs</article>; }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        format!(
            r#"{{"compilerOptions":{{"noEmit":true,"strict":true,"jsx":"preserve","module":"NodeNext","moduleResolution":"NodeNext","paths":{{"presolve":["{}"]}}}}}}"#,
            framework_types.display()
        ),
    )
    .unwrap();
    let executable = root.join("node_modules/.bin/presolve-typescript-authority");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        format!(
            r#"#!/usr/bin/env node
import {{ analyzeV2Authoring }} from "{}";
import {{ readFileSync }} from "node:fs";
process.stdout.write(JSON.stringify(await analyzeV2Authoring(JSON.parse(readFileSync(0, "utf8")))));
"#,
            authority_module.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();

    for command in ["check", "build"] {
        let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
            .arg(command)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{command} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("PSC1001"),
            "a decorator-free layout must not be redirected to the legacy component diagnostic"
        );
    }
    let scoped = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["check", "app/routes/index.tsx", "--format", "json"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        scoped.status.success(),
        "file-scoped check stderr: {}\nstdout: {}",
        String::from_utf8_lossy(&scoped.stderr),
        String::from_utf8_lossy(&scoped.stdout)
    );
    let scoped_json: serde_json::Value =
        serde_json::from_slice(&scoped.stdout).expect("file-scoped check JSON");
    assert_eq!(
        scoped_json["summary"]["compiler_diagnostics"],
        serde_json::json!(0)
    );
    assert!(
        scoped_json["compiler_diagnostics"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "decorator-free file checks must not emit legacy component diagnostics"
    );
    let html = fs::read_to_string(root.join("dist/routes/root/index.html")).unwrap();
    assert!(html.contains("Decorator-free docs"));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn decorator_free_standard_schema_bundles_through_real_authority_and_vite() {
    let root = project_root("v2-standard-schema");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let framework_types = repository.join("framework/packages/presolve/src/index.d.ts");
    let authority_module = repository.join("packages/typescript-authority/src/index.js");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"import { Component, defineForm, field } from "presolve";
import { displayNameSchema } from "./schemas.js";

export class Profile extends Component {
  profile = defineForm({
    fields: {
      displayName: field({ initial: "", validate: [displayNameSchema] }),
    },
  });

  render() {
    return <form form={this.profile}><input bind:value={this.profile.fields.displayName} /></form>;
  }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/schemas.ts"),
        r#"export const displayNameSchema = {
  "~standard": {
    version: 1 as const,
    vendor: "presolve-test",
    validate(value: unknown) {
      const text = typeof value === "string" ? value : "";
      return text.length >= 3
        ? { value: text }
        : { issues: [{ message: "Use at least three characters" }] };
    },
    types: undefined as unknown as { input: string; output: string },
  },
};
"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        format!(
            r#"{{"compilerOptions":{{"noEmit":true,"strict":true,"jsx":"preserve","module":"NodeNext","moduleResolution":"NodeNext","paths":{{"presolve":["{}"]}}}}}}"#,
            framework_types.display()
        ),
    )
    .unwrap();
    let executable = root.join("node_modules/.bin/presolve-typescript-authority");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        format!(
            r#"#!/usr/bin/env node
import {{ analyzeV2Authoring }} from "{}";
import {{ readFileSync }} from "node:fs";
process.stdout.write(JSON.stringify(await analyzeV2Authoring(JSON.parse(readFileSync(0, "utf8")))));
"#,
            authority_module.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink(
        repository.join("packages/vite/node_modules/vite"),
        root.join("node_modules/vite"),
    )
    .unwrap();

    let build = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "Standard Schema build stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let first_forms = fs::read(root.join("dist/routes/root/forms.runtime.json")).unwrap();
    let forms: serde_json::Value = serde_json::from_slice(&first_forms).unwrap();
    assert_eq!(forms["schema_version"], 7);
    assert_eq!(
        forms["standard_schema_module"]["path"],
        "/presolve.validators.js"
    );
    assert_eq!(
        forms["standard_schema_module"]["validators"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    let bundle = fs::read(root.join("dist/presolve.validators.js")).unwrap();
    assert!(String::from_utf8_lossy(&bundle).contains("presolveStandardSchemaValidators"));
    let manifest = fs::read_to_string(root.join("dist/file-routes.manifest.json")).unwrap();
    assert!(manifest.contains("presolve.validators.js"));

    let second = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "second Standard Schema build stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        bundle,
        fs::read(root.join("dist/presolve.validators.js")).unwrap(),
        "Standard Schema publication must be byte deterministic"
    );
    assert_eq!(
        publication_stage_count(&root),
        1,
        "repeated file-route builds must retain only the active atomic release"
    );
    fs::remove_dir_all(root).unwrap();
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
fn environment_command_publishes_only_explicit_public_values() {
    let root = project_root("environment");
    fs::write(
        root.join(".env"),
        "PRESOLVE_PUBLIC_NAME=Presolve\nDATABASE_URL=postgres://secret\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["environment", "--file", ".env"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        manifest["browserValues"]["PRESOLVE_PUBLIC_NAME"],
        "Presolve"
    );
    assert_eq!(
        manifest["serverValueNames"],
        serde_json::json!(["DATABASE_URL"])
    );
    assert!(!String::from_utf8(output.stdout)
        .unwrap()
        .contains("postgres://secret"));
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
fn canonical_application_files_own_the_document_and_global_stylesheet() {
    let root = project_root("canonical-application-files");
    fs::write(
        root.join("app/app.tsx"),
        r#"
@component() class App extends Component {
  @slot() children!: SlotContent;
  render() { return <div class="app-shell"><slot /></div>; }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("app/app.css"),
        ".app-shell { min-height: 100vh; }\n",
    )
    .unwrap();
    fs::write(
        root.join("app/index.html"),
        "<!doctype html>\n<html lang=\"en\">\n<head>\n{{ head }}\n</head>\n<body>\n{{ app }}{{ runtime }}\n</body>\n</html>\n",
    )
    .unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Home</main>; } }"#,
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
    let stylesheet_digest = "36709dfafff32d5ed90c36b3b50c450fe2d484fdb88eb49abcb5f6b17cbff2c8";
    assert!(html.contains(&format!(
        "<link rel=\"stylesheet\" href=\"/app.{stylesheet_digest}.css\">"
    )));
    assert!(html.contains("app-shell"));
    assert!(!html.contains("<main><main"));
    assert_eq!(
        fs::read_to_string(root.join("dist/app.css")).unwrap(),
        ".app-shell { min-height: 100vh; }\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(format!("dist/app.{stylesheet_digest}.css"))).unwrap(),
        ".app-shell { min-height: 100vh; }\n"
    );
    let runtime = fs::read(root.join("dist/routes/root/runtime.js")).unwrap();
    let runtime_digest = format!("{:x}", Sha256::digest(&runtime));
    assert!(html.contains(&format!(
        "<script src=\"./runtime.{runtime_digest}.js\" defer></script>"
    )));
    assert_eq!(
        fs::read(root.join(format!("dist/routes/root/runtime.{runtime_digest}.js"))).unwrap(),
        runtime
    );
    let manifest = fs::read_to_string(root.join("dist/file-routes.manifest.json")).unwrap();
    assert!(manifest.contains(&format!("app.{stylesheet_digest}.css")));
    assert!(manifest.contains(&format!("routes/root/runtime.{runtime_digest}.js")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn default_build_publishes_compiler_joined_route_metadata() {
    let root = project_root("route-metadata");
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Home</main>; } }"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/index.metadata.json"),
        r#"{"title":"Home","description":"Welcome"}"#,
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
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("dist/route-metadata.json")).unwrap()).unwrap();
    assert_eq!(metadata["routes"][0]["path"], "/");
    assert_eq!(metadata["routes"][0]["title"], "Home");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn canonical_route_loader_bundles_executes_caches_and_bootstraps_the_browser() {
    let root = project_root("route-loader");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let framework_types = repository.join("framework/packages/presolve/src/index.d.ts");
    let authority_module = repository.join("packages/typescript-authority/src/index.js");
    let package = root.join("node_modules/post-service");
    fs::create_dir_all(package.join("dist")).unwrap();
    fs::create_dir_all(root.join("app/routes/posts")).unwrap();
    fs::create_dir_all(root.join("app/routes/private")).unwrap();
    fs::create_dir_all(root.join("app/routes/fresh")).unwrap();
    fs::write(
        root.join("app/routes/posts/[slug].tsx"),
        r#"
import { Component, loader, type Resource, type RouteParameters } from "presolve";
import { loadPost } from "post-service";
type PostRecord = { slug: string; title: string; };
type NotFound = { code: "not_found"; };
export class Post extends Component {
  post: Resource<PostRecord, NotFound> = loader<PostRecord, NotFound>(
    async (params: RouteParameters, signal: AbortSignal) => loadPost(params, signal),
  );
  render() { return <article>{this.post.data?.title ?? "Loading"}</article>; }
}
"#,
    )
    .unwrap();
    for (directory, class, field, imported) in [
        ("private", "PrivatePost", "privatePost", "loadPrivate"),
        ("fresh", "FreshPost", "freshPost", "loadFresh"),
    ] {
        fs::write(
            root.join(format!("app/routes/{directory}/[slug].tsx")),
            format!(
                r#"import {{ Component, loader, type Resource, type RouteParameters }} from "presolve";
import {{ {imported} }} from "post-service";
type PostRecord = {{ slug: string; title: string; }};
type NotFound = {{ code: "not_found"; }};
export class {class} extends Component {{
  {field}: Resource<PostRecord, NotFound> = loader<PostRecord, NotFound>(
    async (params: RouteParameters, signal: AbortSignal) => {imported}(params, signal),
  );
  render() {{ return <article />; }}
}}
"#
            ),
        )
        .unwrap();
    }
    fs::write(
        package.join("presolve.contract.json"),
        r#"{"schema_version":1,"package":"post-service","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadFresh":{"kind":"resource","type_signature":"(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>","runtime_module":"dist/load-fresh.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"no_store"},"failure":"typed"}},"loadPost":{"kind":"resource","type_signature":"(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>","runtime_module":"dist/load-post.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"public","max_age_seconds":60},"failure":"typed"}},"loadPrivate":{"kind":"resource","type_signature":"(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>","runtime_module":"dist/load-private.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"private","max_age_seconds":60},"failure":"typed"}}}}"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/load-post.js"),
        r#"const calls = new Map();
export async function loadPost({ slug }, signal) {
  if (slug === 'missing') throw { code: 'not_found' };
  if (slug === 'invalid') return { slug, title: 42 };
  if (slug === 'slow') return await new Promise((resolve, reject) => signal.addEventListener('abort', () => { console.error('PRESOLVE_TEST_LOADER_ABORTED'); reject(new Error('aborted')); }, { once: true }));
  calls.set(slug, (calls.get(slug) ?? 0) + 1);
  return { slug, title: `Post:${slug}:${calls.get(slug)}` };
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/load-private.js"),
        r#"const calls = new Map();
export async function loadPrivate({ slug }, signal) {
  if (signal.aborted) throw new DOMException('Aborted', 'AbortError');
  calls.set(slug, (calls.get(slug) ?? 0) + 1);
  return { slug, title: `Private:${slug}:${calls.get(slug)}` };
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/load-fresh.js"),
        r#"const calls = new Map();
export async function loadFresh({ slug }, signal) {
  if (signal.aborted) throw new DOMException('Aborted', 'AbortError');
  calls.set(slug, (calls.get(slug) ?? 0) + 1);
  return { slug, title: `Fresh:${slug}:${calls.get(slug)}` };
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("index.d.ts"),
        r#"import type { RouteParameters } from "presolve";
export interface PostRecord { slug: string; title: string; }
export interface NotFound { code: "not_found"; }
export declare function loadFresh(params: RouteParameters, signal: AbortSignal): Promise<PostRecord>;
export declare function loadPost(params: RouteParameters, signal: AbortSignal): Promise<PostRecord>;
export declare function loadPrivate(params: RouteParameters, signal: AbortSignal): Promise<PostRecord>;
"#,
    )
    .unwrap();
    fs::write(
        package.join("package.json"),
        r#"{"name":"post-service","version":"1.0.0","type":"module","types":"index.d.ts"}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        format!(
            r#"{{"compilerOptions":{{"noEmit":true,"strict":true,"jsx":"preserve","module":"NodeNext","moduleResolution":"NodeNext","paths":{{"presolve":["{}"]}}}}}}"#,
            framework_types.display()
        ),
    )
    .unwrap();
    let executable = root.join("node_modules/.bin/presolve-typescript-authority");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        format!(
            r#"#!/usr/bin/env node
import {{ analyzeV2Authoring }} from "{}";
import {{ readFileSync }} from "node:fs";
process.stdout.write(JSON.stringify(await analyzeV2Authoring(JSON.parse(readFileSync(0, "utf8")))));
"#,
            authority_module.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink(
        repository.join("packages/vite/node_modules/vite"),
        root.join("node_modules/vite"),
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
    assert!(!plan.contains("@loader"));
    let plan: serde_json::Value = serde_json::from_str(&plan).unwrap();
    assert_eq!(plan["schema_version"], 2);
    let loader = &plan["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["path"] == "/posts/:slug")
        .unwrap()["loaders"][0];
    assert_eq!(loader["parameters"][0]["name"], "slug");
    assert_eq!(loader["parameters"][0]["segment_index"], 1);
    assert_eq!(loader["normalization"]["percent_decoding"], "strict_utf8");
    assert_eq!(loader["data_codec"]["kind"], "object_codec");
    assert_eq!(loader["error_codec"]["kind"], "object_codec");
    assert!(loader["resource_activation_id"]
        .as_str()
        .unwrap()
        .contains("resource-activation"));
    let resources: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("dist/routes/segment-posts/parameter-slug/resources.runtime.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(resources["schema_version"], 4);
    assert_eq!(resources["server_bootstraps"].as_array().unwrap().len(), 1);
    assert_eq!(
        resources["server_bootstraps"][0]["loader_capability_id"],
        loader["id"]
    );
    assert!(resources["declarations"][0]["endpoint"]
        .get("runtime_location")
        .is_none());

    let deploy = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-loaders"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        deploy.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&deploy.stderr)
    );
    let adapter = root.join(".presolve/node");
    assert!(adapter.join("presolve.route-loaders.mjs").is_file());
    let deployment: serde_json::Value =
        serde_json::from_slice(&fs::read(adapter.join("deployment.plan.json")).unwrap()).unwrap();
    assert_eq!(deployment["schemaVersion"], 3);
    assert_eq!(deployment["routeLoaders"].as_array().unwrap().len(), 3);
    assert_eq!(
        deployment["routeLoaderRegistry"]["path"],
        "presolve.route-loaders.mjs"
    );
    let first_plan = fs::read(adapter.join("deployment.plan.json")).unwrap();
    let first_host = fs::read(adapter.join("server.mjs")).unwrap();
    let first_registry = fs::read(adapter.join("presolve.route-loaders.mjs")).unwrap();
    let deterministic = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-loaders"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        deterministic.status.success(),
        "deterministic deploy stderr: {}",
        String::from_utf8_lossy(&deterministic.stderr)
    );
    assert_eq!(
        fs::read(adapter.join("deployment.plan.json")).unwrap(),
        first_plan,
        "Node deployment plan changed across identical preparations"
    );
    assert_eq!(
        fs::read(adapter.join("server.mjs")).unwrap(),
        first_host,
        "Node host changed across identical preparations"
    );
    assert_eq!(
        fs::read(adapter.join("presolve.route-loaders.mjs")).unwrap(),
        first_registry,
        "route-loader registry changed across identical preparations"
    );

    let index_path = root.join("dist/routes/segment-posts/parameter-slug/index.html");
    let index = fs::read_to_string(&index_path).unwrap();
    fs::write(
        &index_path,
        index.replace(
            "</body>",
            r##"<script>
const waitFor = (predicate, label) => new Promise((resolve, reject) => { const deadline = Date.now() + 4000; const tick = () => predicate() ? resolve() : Date.now() > deadline ? reject(new Error(`Timed out waiting for ${label}`)) : setTimeout(tick, 20); tick(); });
(async () => {
  await waitFor(() => ["ready", "error"].includes(document.documentElement.dataset.presolveRuntime), "runtime readiness");
  if (document.documentElement.dataset.presolveRuntime !== "ready") throw new Error("runtime failed to boot");
  const bootstrap = JSON.parse(document.querySelector("#presolve-resource-bootstrap")?.textContent ?? "null");
  const resource = [...window.__PRESOLVE__.store.resources.values()][0];
  if (bootstrap?.schema_version !== 1 || bootstrap.values?.[0]?.data?.title !== "Post:browser:1") throw new Error(`bootstrap mismatch: ${JSON.stringify(bootstrap)}`);
  if (resource?.state !== "ready" || resource?.data?.title !== "Post:browser:1") throw new Error(`resource mismatch: ${JSON.stringify(resource)}`);
  if (document.querySelector("article")?.textContent !== "Post:browser:1") throw new Error(`rendered data mismatch: ${document.querySelector("article")?.textContent}`);
  if (window.__PRESOLVE__.diagnostics.length !== 0) throw new Error(`runtime diagnostics: ${JSON.stringify(window.__PRESOLVE__.diagnostics)}`);
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_NODE_ROUTE_LOADER_BROWSER_PASS</div>");
})().catch((error) => document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_NODE_ROUTE_LOADER_BROWSER_FAIL: ${error.message}</div>`));
</script></body>"##,
        ),
    )
    .unwrap();
    let syntax = Command::new("node")
        .arg("--check")
        .arg(adapter.join("server.mjs"))
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "Node release syntax stderr: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut host = Command::new("node")
        .arg(adapter.join("server.mjs"))
        .env("PORT", port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let hello = raw_http_request(
        port,
        "GET /posts/hello/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(hello.starts_with("HTTP/1.1 200 OK"), "{hello}");
    assert!(hello.contains("public, max-age=60"));
    assert!(hello.contains(r#""title":"Post:hello:1""#));
    let cached = raw_http_request(
        port,
        "GET /posts/hello/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(cached.contains(r#""title":"Post:hello:1""#));
    let typed_failure = raw_http_request(
        port,
        "GET /posts/missing/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(typed_failure.starts_with("HTTP/1.1 200 OK"));
    assert!(typed_failure.contains(r#""state":"failed""#));
    assert!(typed_failure.contains(r#""code":"not_found""#));
    let invalid = raw_http_request(
        port,
        "GET /posts/invalid/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(invalid.starts_with("HTTP/1.1 500 Internal Server Error"));
    assert!(invalid.contains("PSNODE2016_ROUTE_LOADER_EXECUTION_FAILED"));
    let private_first = raw_http_request(
        port,
        "GET /private/account/ HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer a\r\nConnection: close\r\n\r\n",
    );
    assert!(private_first.contains("private, max-age=60"));
    assert!(private_first.contains("Vary: Authorization, Cookie"));
    assert!(private_first.contains(r#""title":"Private:account:1""#));
    let private_cached = raw_http_request(
        port,
        "GET /private/account/ HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer a\r\nConnection: close\r\n\r\n",
    );
    assert!(private_cached.contains(r#""title":"Private:account:1""#));
    let private_partition = raw_http_request(
        port,
        "GET /private/account/ HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer b\r\nConnection: close\r\n\r\n",
    );
    assert!(private_partition.contains(r#""title":"Private:account:2""#));
    let fresh_first = raw_http_request(
        port,
        "GET /fresh/news/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(fresh_first.contains("Cache-Control: no-store"));
    assert!(fresh_first.contains(r#""title":"Fresh:news:1""#));
    let fresh_second = raw_http_request(
        port,
        "GET /fresh/news/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(fresh_second.contains(r#""title":"Fresh:news:2""#));
    let malformed = raw_http_request(
        port,
        "GET /posts/%ZZ/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(malformed.starts_with("HTTP/1.1 400 Bad Request"));

    let profile = adapter.join("chrome-route-loader-profile");
    fs::create_dir_all(&profile).unwrap();
    let mut chrome_arguments = vec![
        "--headless=new".to_string(),
        "--disable-gpu".to_string(),
        "--no-first-run".to_string(),
        "--disable-background-networking".to_string(),
        "--disable-component-update".to_string(),
        "--disable-default-apps".to_string(),
        "--disable-extensions".to_string(),
        "--disable-sync".to_string(),
        "--virtual-time-budget=5000".to_string(),
        "--dump-dom".to_string(),
        format!("--user-data-dir={}", profile.display()),
        format!("http://127.0.0.1:{port}/posts/browser/"),
    ];
    if std::env::var_os("CI").is_some() {
        chrome_arguments.insert(0, "--no-sandbox".to_string());
        chrome_arguments.insert(1, "--disable-dev-shm-usage".to_string());
    }
    let chrome = run_chrome_with_timeout(chrome_bin(), &chrome_arguments, Duration::from_secs(20));
    assert!(
        String::from_utf8_lossy(&chrome.stdout).contains("PRESOLVE_NODE_ROUTE_LOADER_BROWSER_PASS"),
        "browser route-loader probe failed\nstatus: {}\nstdout: {}\nstderr: {}",
        chrome.status,
        String::from_utf8_lossy(&chrome.stdout),
        String::from_utf8_lossy(&chrome.stderr)
    );

    let mut disconnected = TcpStream::connect(("127.0.0.1", port)).unwrap();
    disconnected
        .write_all(b"GET /posts/slow/ HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    drop(disconnected);
    thread::sleep(Duration::from_millis(100));
    host.kill().unwrap();
    let output = host.wait_with_output().unwrap();
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PRESOLVE_TEST_LOADER_ABORTED"),
        "client disconnect did not abort the active loader capability\nhost stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let shutdown_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let shutdown_port = shutdown_listener.local_addr().unwrap().port();
    drop(shutdown_listener);
    let shutdown_host = Command::new("node")
        .arg(adapter.join("server.mjs"))
        .env("PORT", shutdown_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut active = loop {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", shutdown_port)) {
            break stream;
        }
        thread::sleep(Duration::from_millis(25));
    };
    active
        .write_all(
            b"GET /posts/slow/ HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n",
        )
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    let terminated = Command::new("kill")
        .args(["-TERM", &shutdown_host.id().to_string()])
        .status()
        .unwrap();
    assert!(terminated.success());
    thread::sleep(Duration::from_millis(100));
    drop(active);
    let shutdown_output = shutdown_host.wait_with_output().unwrap();
    assert!(shutdown_output.status.success());
    assert!(
        String::from_utf8_lossy(&shutdown_output.stderr).contains("PRESOLVE_TEST_LOADER_ABORTED"),
        "host shutdown did not abort the active loader capability: {}",
        String::from_utf8_lossy(&shutdown_output.stderr)
    );
    let package_types = fs::read_to_string(package.join("index.d.ts")).unwrap();
    fs::write(
        package.join("index.d.ts"),
        package_types.replace(
            "loadPost(params: RouteParameters, signal: AbortSignal)",
            "loadPost(params: any, signal: any)",
        ),
    )
    .unwrap();
    let unproven = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!unproven.status.success());
    assert!(
        String::from_utf8_lossy(&unproven.stderr)
            .contains("V2 TypeScript authority returned incompatible evidence"),
        "non-exact TypeScript route loader was not rejected: {}",
        String::from_utf8_lossy(&unproven.stderr)
    );
    fs::write(package.join("index.d.ts"), &package_types).unwrap();
    let package_contract = fs::read_to_string(package.join("presolve.contract.json")).unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        package_contract.replace("dist/load-post.js", "dist/missing-load-post.js"),
    )
    .unwrap();
    let missing_runtime = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-loaders"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!missing_runtime.status.success());
    assert!(
        String::from_utf8_lossy(&missing_runtime.stderr)
            .contains("PSDISC1012_PACKAGE_RUNTIME_MISSING"),
        "missing loader runtime was not rejected: {}",
        String::from_utf8_lossy(&missing_runtime.stderr)
    );
    fs::write(package.join("presolve.contract.json"), &package_contract).unwrap();
    fs::write(
        package.join("dist/load-post.js"),
        "export const wrongExport = true;\n",
    )
    .unwrap();
    let missing_export = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-loaders"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!missing_export.status.success());
    assert!(
        String::from_utf8_lossy(&missing_export.stderr)
            .contains("PSNODE1025_ROUTE_LOADER_BUNDLE_FAILED"),
        "missing named loader export was not rejected: {}",
        String::from_utf8_lossy(&missing_export.stderr)
    );
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
  @serverAction("savePost") save(): void {}
  render() { return <form />; }
}
"#,
    )
    .unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        r#"{"schema_version":1,"package":"post-service","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"savePost":{"kind":"server_action","type_signature":"(FormData, AbortSignal) -> Promise<ServerActionResult>","runtime_module":"dist/save-post.js","resume_policy":"cold_fallback","server_action":{"input":"form_data","response":"json","failure":"typed"}}}}"#,
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
    let node = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        node.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    let deployment = fs::read_to_string(root.join(".presolve/node/deployment.plan.json")).unwrap();
    assert!(deployment.contains("\"execution\": \"node\""));
    assert!(deployment.contains("\"serverActionCount\": 1"));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn canonical_form_server_action_bundles_and_executes_through_the_node_host() {
    let root = project_root("canonical-node-server-action");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let framework_types = repository.join("framework/packages/presolve/src/index.d.ts");
    let authority_module = repository.join("packages/typescript-authority/src/index.js");
    let package = root.join("node_modules/post-service");
    fs::create_dir_all(package.join("dist")).unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"import { Component, defineForm, field, required } from "presolve";
import { redirectPost as navigatePost, savePost as persistPost } from "post-service";

export class Post extends Component {
  post = defineForm({
    serialization: "form-data",
    fields: { title: field({ initial: "", validate: [required()] }) },
    submit: async ({ formData, signal }) => persistPost(formData, signal),
  });
  redirect = defineForm({
    serialization: "form-data",
    fields: { destination: field({ initial: "/" }) },
    submit: async ({ formData, signal }) => navigatePost(formData, signal),
  });
  render() {
    return <main><form form={this.post}><input name="title" bind:value={this.post.fields.title} /><button type="submit">Save</button></form><form form={this.redirect}><input name="destination" bind:value={this.redirect.fields.destination} /><button type="submit">Continue</button></form></main>;
  }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("app/routes/about.tsx"),
        r#"import { Component } from "presolve";
export class About extends Component { render() { return <main>About Presolve</main>; } }
"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        format!(
            r#"{{"compilerOptions":{{"noEmit":true,"strict":true,"jsx":"preserve","module":"NodeNext","moduleResolution":"NodeNext","paths":{{"presolve":["{}"]}}}}}}"#,
            framework_types.display()
        ),
    )
    .unwrap();
    fs::write(
        package.join("package.json"),
        r#"{"name":"post-service","version":"1.0.0","type":"module","types":"index.d.ts"}"#,
    )
    .unwrap();
    fs::write(
        package.join("index.d.ts"),
        "export declare function savePost(data: FormData, signal: AbortSignal): Promise<{ saved: string; aborted: boolean }>;\nexport declare function redirectPost(data: FormData, signal: AbortSignal): Promise<{ location: `/${string}` }>;\n",
    )
    .unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        r#"{"schema_version":1,"package":"post-service","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"redirectPost":{"kind":"server_action","type_signature":"(FormData, AbortSignal) -> Promise<ServerActionResult>","runtime_module":"dist/redirect-post.js","resume_policy":"cold_fallback","server_action":{"input":"form_data","response":"redirect","failure":"typed"}},"savePost":{"kind":"server_action","type_signature":"(FormData, AbortSignal) -> Promise<ServerActionResult>","runtime_module":"dist/save-post.js","resume_policy":"cold_fallback","server_action":{"input":"form_data","response":"json","failure":"typed"}}}}"#,
    )
    .unwrap();
    fs::write(
        package.join("dist/save-post.js"),
        "let browserCalls = 0; export async function savePost(data, signal) { const title = String(data.get('title')); if (title === 'invalid') throw { status: 422, code: 'TITLE_INVALID', message: 'Title is invalid', issues: [{ path: ['title'] }] }; if (title === 'slow') return await new Promise((resolve, reject) => { signal.addEventListener('abort', () => { console.error('PRESOLVE_TEST_ACTION_ABORTED'); reject(new Error('aborted')); }, { once: true }); }); if (title === 'Browser') browserCalls += 1; return { saved: title, aborted: signal.aborted, browserCalls }; }\n",
    )
    .unwrap();
    fs::write(
        package.join("dist/redirect-post.js"),
        "export async function redirectPost(data, signal) { if (signal.aborted) throw new Error('aborted'); return { location: String(data.get('destination')) }; }\n",
    )
    .unwrap();
    let executable = root.join("node_modules/.bin/presolve-typescript-authority");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        format!(
            r#"#!/usr/bin/env node
import {{ analyzeV2Authoring }} from "{}";
import {{ readFileSync }} from "node:fs";
process.stdout.write(JSON.stringify(await analyzeV2Authoring(JSON.parse(readFileSync(0, "utf8")))));
"#,
            authority_module.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink(
        repository.join("packages/vite/node_modules/vite"),
        root.join("node_modules/vite"),
    )
    .unwrap();

    let deploy = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-actions"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        deploy.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&deploy.stderr)
    );
    let adapter = root.join(".presolve/node");
    assert!(adapter.join("presolve.server-actions.mjs").is_file());
    let deployment: serde_json::Value =
        serde_json::from_slice(&fs::read(adapter.join("deployment.plan.json")).unwrap()).unwrap();
    assert_eq!(deployment["schemaVersion"], 3);
    let server_actions = deployment["serverActions"].as_array().unwrap();
    assert_eq!(server_actions.len(), 2);
    let request_path = server_actions
        .iter()
        .find(|action| action["response"] == "json")
        .and_then(|action| action["requestPath"].as_str())
        .unwrap();
    let redirect_path = server_actions
        .iter()
        .find(|action| action["response"] == "redirect")
        .and_then(|action| action["requestPath"].as_str())
        .unwrap();
    assert!(request_path.starts_with("/_presolve/actions/"));
    assert_eq!(
        deployment["serverActionRegistry"]["path"],
        "presolve.server-actions.mjs"
    );
    let first_forms = fs::read(root.join("dist/routes/root/forms.runtime.json")).unwrap();
    let forms: serde_json::Value = serde_json::from_slice(&first_forms).unwrap();
    assert_eq!(forms["schema_version"], 7);
    assert_eq!(
        forms["server_action_capabilities"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(forms["server_action_capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action["request_path"] == request_path));
    assert!(forms.get("submission_capability_module").is_none());
    let first_plan = fs::read(adapter.join("deployment.plan.json")).unwrap();
    let first_host = fs::read(adapter.join("server.mjs")).unwrap();
    let first_registry = fs::read(adapter.join("presolve.server-actions.mjs")).unwrap();
    let deterministic = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-actions"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        deterministic.status.success(),
        "deterministic deploy stderr: {}",
        String::from_utf8_lossy(&deterministic.stderr)
    );
    assert_eq!(
        fs::read(adapter.join("deployment.plan.json")).unwrap(),
        first_plan,
        "Node deployment plan changed across identical preparations"
    );
    assert_eq!(
        fs::read(adapter.join("server.mjs")).unwrap(),
        first_host,
        "Node host changed across identical preparations"
    );
    assert_eq!(
        fs::read(adapter.join("presolve.server-actions.mjs")).unwrap(),
        first_registry,
        "server-action registry changed across identical preparations"
    );
    assert_eq!(
        fs::read(root.join("dist/routes/root/forms.runtime.json")).unwrap(),
        first_forms,
        "Forms server-action artifact changed across identical preparations"
    );
    let index_path = root.join("dist/routes/root/index.html");
    let index = fs::read_to_string(&index_path).unwrap();
    fs::write(
        &index_path,
        index.replace(
            "</body>",
            r#"<script>
const waitFor = (predicate, label) => new Promise((resolve, reject) => { const deadline = Date.now() + 4000; const tick = () => predicate() ? resolve() : Date.now() > deadline ? reject(new Error(`Timed out waiting for ${label}`)) : setTimeout(tick, 20); tick(); });
(async () => {
  await waitFor(() => ["ready", "error"].includes(document.documentElement.dataset.presolveRuntime), "runtime readiness");
  if (document.documentElement.dataset.presolveRuntime !== "ready") throw new Error("runtime failed to boot");
  const forms = [...document.querySelectorAll("form")];
  const form = forms[0];
  const input = form.querySelector('input[name="title"]');
  const instances = [...window.__PRESOLVE__.store.formInstances.values()];
  const instance = instances.find(entry => entry.definition.debug_name === "post");
  const invalidLocal = new Event("submit", { bubbles: true, cancelable: true });
  form.dispatchEvent(invalidLocal);
  await waitFor(() => instance.submission === "Invalid", "client validation rejection");
  if (!invalidLocal.defaultPrevented) throw new Error("invalid submission did not remain compiler-owned");
  window.__PRESOLVE_FORMS__.resetForm(instance.instance.id);
  input.value = "invalid";
  input.dispatchEvent(new Event("input", { bubbles: true }));
  form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  await waitFor(() => instance.submission === "Failed", "typed server failure");
  if (!window.__PRESOLVE__.diagnostics.some(entry => entry.code === "PSR_FORM_SERVER_ACTION_REJECTED" && entry.detail?.status === 422)) throw new Error("typed server failure was not normalized");
  window.__PRESOLVE_FORMS__.resetForm(instance.instance.id);
  input.value = "slow";
  input.dispatchEvent(new Event("input", { bubbles: true }));
  form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  await waitFor(() => instance.submission === "Submitting", "active server action");
  window.__PRESOLVE_FORMS__.resetForm(instance.instance.id);
  await waitFor(() => instance.submission === "Idle", "cancelled server action reset");
  input.value = "Browser";
  input.dispatchEvent(new Event("input", { bubbles: true }));
  const submit = new Event("submit", { bubbles: true, cancelable: true });
  form.dispatchEvent(submit);
  form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  await waitFor(() => instance.submission === "Completed" || instance.submission === "Failed", "server action response");
  if (!submit.defaultPrevented || instance.submission !== "Completed" || instance.submission_result?.saved !== "Browser" || instance.submission_result?.browserCalls !== 1) throw new Error(`server action mismatch: ${JSON.stringify(instance.submission_result)}`);
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_NODE_SERVER_ACTION_BROWSER_PASS</div>");
})().catch((error) => document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_NODE_SERVER_ACTION_BROWSER_FAIL: ${error.message}</div>`));
</script></body>"#,
        ),
    )
    .unwrap();
    let syntax = Command::new("node")
        .arg("--check")
        .arg(adapter.join("server.mjs"))
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "Node release syntax stderr: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut host = Command::new("node")
        .arg(adapter.join("server.mjs"))
        .env("PORT", port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let page = raw_http_request(
        port,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(page.starts_with("HTTP/1.1 200 OK"));
    assert!(page.contains("Save"));
    let about_page = raw_http_request(
        port,
        "GET /about HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(about_page.starts_with("HTTP/1.1 200 OK"));
    assert!(about_page.contains("About Presolve"));
    let profile = adapter.join("chrome-server-action-profile");
    fs::create_dir_all(&profile).unwrap();
    let mut chrome_arguments = vec![
        "--headless=new".to_string(),
        "--disable-gpu".to_string(),
        "--no-first-run".to_string(),
        "--disable-background-networking".to_string(),
        "--disable-component-update".to_string(),
        "--disable-default-apps".to_string(),
        "--disable-extensions".to_string(),
        "--disable-sync".to_string(),
        "--virtual-time-budget=5000".to_string(),
        "--dump-dom".to_string(),
        format!("--user-data-dir={}", profile.display()),
        format!("http://127.0.0.1:{port}/"),
    ];
    if std::env::var_os("CI").is_some() {
        chrome_arguments.insert(0, "--no-sandbox".to_string());
        chrome_arguments.insert(1, "--disable-dev-shm-usage".to_string());
    }
    let chrome = run_chrome_with_timeout(chrome_bin(), &chrome_arguments, Duration::from_secs(20));
    assert!(
        String::from_utf8_lossy(&chrome.stdout)
            .contains("PRESOLVE_NODE_SERVER_ACTION_BROWSER_PASS"),
        "browser server-action probe failed\nstatus: {}\nstdout: {}\nstderr: {}",
        chrome.status,
        String::from_utf8_lossy(&chrome.stdout),
        String::from_utf8_lossy(&chrome.stderr)
    );
    let method = raw_http_request(
        port,
        &format!(
            "GET {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(method.starts_with("HTTP/1.1 405 Method Not Allowed"));
    let body = "title=Compiler";
    let foreign = raw_http_request(
        port,
        &format!(
            "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: https://example.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(foreign.starts_with("HTTP/1.1 403 Forbidden"));
    assert!(foreign.contains("PSNODE2004_ACTION_ORIGIN_REJECTED"));
    let unsupported = raw_http_request(
        port,
        &format!(
            "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(unsupported.starts_with("HTTP/1.1 415 Unsupported Media Type"));
    let multipart_body = "--presolve-boundary\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nMultipart\r\n--presolve-boundary--\r\n";
    let multipart = raw_http_request(
        port,
        &format!(
            "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: multipart/form-data; boundary=presolve-boundary\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{multipart_body}",
            multipart_body.len()
        ),
    );
    assert!(multipart.starts_with("HTTP/1.1 200 OK"));
    assert!(multipart.contains(r#""saved":"Multipart""#));
    let oversized = raw_http_request(
        port,
        &format!(
            "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 8388609\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(oversized.starts_with("HTTP/1.1 413 Payload Too Large"));
    assert!(oversized.contains("PSNODE2008_ACTION_BODY_TOO_LARGE"));
    let invalid_body = "title=invalid";
    let invalid = raw_http_request(
        port,
        &format!(
            "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{invalid_body}",
            invalid_body.len()
        ),
    );
    assert!(invalid.starts_with("HTTP/1.1 422 Unprocessable Entity"));
    assert!(invalid.contains("TITLE_INVALID"));
    let redirect_body = "destination=%2F";
    let redirect = raw_http_request(
        port,
        &format!(
            "POST {redirect_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{redirect_body}",
            redirect_body.len()
        ),
    );
    assert!(redirect.starts_with("HTTP/1.1 303 See Other"));
    assert!(redirect.to_ascii_lowercase().contains("location: /\r\n"));
    let request = format!(
        "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = raw_http_request(port, &request);
    let slow_body = "title=slow";
    let mut disconnected = TcpStream::connect(("127.0.0.1", port)).unwrap();
    disconnected
        .write_all(
            format!(
                "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{slow_body}",
                slow_body.len()
            )
            .as_bytes(),
        )
        .unwrap();
    drop(disconnected);
    thread::sleep(Duration::from_millis(100));
    host.kill().unwrap();
    let output = host.wait_with_output().unwrap();
    assert!(
        response.starts_with("HTTP/1.1 200 OK"),
        "response: {response}\nhost stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(response.contains(r#""saved":"Compiler""#));
    assert!(response.contains(r#""aborted":false"#));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PRESOLVE_TEST_ACTION_ABORTED"),
        "client disconnect did not abort the active package capability\nhost stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let shutdown_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let shutdown_port = shutdown_listener.local_addr().unwrap().port();
    drop(shutdown_listener);
    let shutdown_host = Command::new("node")
        .arg(adapter.join("server.mjs"))
        .env("PORT", shutdown_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = raw_http_request(
        shutdown_port,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    let mut active = TcpStream::connect(("127.0.0.1", shutdown_port)).unwrap();
    active
        .write_all(
            format!(
                "POST {request_path} HTTP/1.1\r\nHost: 127.0.0.1:{shutdown_port}\r\nOrigin: http://127.0.0.1:{shutdown_port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{slow_body}",
                slow_body.len()
            )
            .as_bytes(),
        )
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    let terminated = Command::new("kill")
        .args(["-TERM", &shutdown_host.id().to_string()])
        .status()
        .unwrap();
    assert!(terminated.success());
    thread::sleep(Duration::from_millis(100));
    drop(active);
    let shutdown_output = shutdown_host.wait_with_output().unwrap();
    assert!(shutdown_output.status.success());
    assert!(
        String::from_utf8_lossy(&shutdown_output.stderr).contains("PRESOLVE_TEST_ACTION_ABORTED"),
        "host shutdown did not abort the active package capability: {}",
        String::from_utf8_lossy(&shutdown_output.stderr)
    );
    fs::write(
        package.join("index.d.ts"),
        "export declare function savePost(data: any, signal: any): Promise<{ saved: string; aborted: boolean }>;\nexport declare function redirectPost(data: FormData, signal: AbortSignal): Promise<{ location: `/${string}` }>;\n",
    )
    .unwrap();
    let unproven = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!unproven.status.success());
    assert!(
        String::from_utf8_lossy(&unproven.stderr).contains("was not proven as an imported"),
        "non-exact TypeScript server action was not rejected: {}",
        String::from_utf8_lossy(&unproven.stderr)
    );
    fs::write(
        package.join("index.d.ts"),
        "export declare function savePost(data: FormData, signal: AbortSignal): Promise<{ saved: string; aborted: boolean }>;\nexport declare function redirectPost(data: FormData, signal: AbortSignal): Promise<{ location: `/${string}` }>;\n",
    )
    .unwrap();
    let package_contract = fs::read_to_string(package.join("presolve.contract.json")).unwrap();
    fs::write(
        package.join("presolve.contract.json"),
        package_contract.replace("dist/save-post.js", "dist/missing-save-post.js"),
    )
    .unwrap();
    let missing_runtime = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-actions"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!missing_runtime.status.success());
    assert!(
        String::from_utf8_lossy(&missing_runtime.stderr)
            .contains("PSDISC1012_PACKAGE_RUNTIME_MISSING"),
        "missing server runtime was not rejected: {}",
        String::from_utf8_lossy(&missing_runtime.stderr)
    );
    fs::write(package.join("presolve.contract.json"), &package_contract).unwrap();
    fs::write(
        package.join("dist/save-post.js"),
        "export const wrongExport = true;\n",
    )
    .unwrap();
    let missing_export = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-actions"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!missing_export.status.success());
    assert!(
        String::from_utf8_lossy(&missing_export.stderr)
            .contains("PSNODE1018_SERVER_ACTION_BUNDLE_FAILED"),
        "missing named server export was not rejected: {}",
        String::from_utf8_lossy(&missing_export.stderr)
    );
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
    let node = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "node", "--prepare", "--name", "presolve-docs"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        node.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    let node_adapter = root.join(".presolve/node");
    let node_plan = fs::read_to_string(node_adapter.join("deployment.plan.json")).unwrap();
    assert!(node_plan.contains("\"provider\": \"node\""));
    assert!(node_plan.contains("\"execution\": \"static\""));
    let syntax = Command::new("node")
        .arg("--check")
        .arg(node_adapter.join("server.mjs"))
        .output()
        .unwrap();
    assert!(
        syntax.status.success(),
        "Node release syntax stderr: {}",
        String::from_utf8_lossy(&syntax.stderr)
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut host = Command::new("node")
        .arg(node_adapter.join("server.mjs"))
        .env("PORT", port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut response = Vec::new();
    for _ in 0..120 {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .write_all(b"GET /about/ HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
            stream.read_to_end(&mut response).unwrap();
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    host.kill().unwrap();
    host.wait().unwrap();
    let response = String::from_utf8(response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("About"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_styles_and_public_files_join_the_atomic_publication_and_deployment_inventory() {
    let root = project_root("static-publication-inputs");
    fs::create_dir_all(root.join("styles")).unwrap();
    fs::create_dir_all(root.join("public/brand")).unwrap();
    fs::write(
        root.join("styles/site.css"),
        "body { color: rebeccapurple; }\n",
    )
    .unwrap();
    fs::write(
        root.join("public/brand/logo.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\" />\n",
    )
    .unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main><link rel="stylesheet" href="/styles/site.css" />Static assets</main>; } }"#,
    )
    .unwrap();

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
    assert_eq!(
        fs::read_to_string(root.join("dist/styles/site.css")).unwrap(),
        "body { color: rebeccapurple; }\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("dist/brand/logo.svg")).unwrap(),
        "<svg xmlns=\"http://www.w3.org/2000/svg\" />\n"
    );
    let manifest = fs::read_to_string(root.join("dist/file-routes.manifest.json")).unwrap();
    assert!(manifest.contains("styles/site.css"));
    assert!(manifest.contains("brand/logo.svg"));

    let cloudflare = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "deploy",
            "cloudflare",
            "--prepare",
            "--name",
            "presolve-static-inputs",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        cloudflare.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&cloudflare.stderr)
    );
    let cloudflare_plan =
        fs::read_to_string(root.join(".presolve/cloudflare/deployment.plan.json")).unwrap();
    assert!(cloudflare_plan.contains("styles/site.css"));
    assert!(cloudflare_plan.contains("brand/logo.svg"));

    let node = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args([
            "deploy",
            "node",
            "--prepare",
            "--name",
            "presolve-static-inputs",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        node.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut host = Command::new("node")
        .arg(root.join(".presolve/node/server.mjs"))
        .env("PORT", port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut response = Vec::new();
    for _ in 0..120 {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .write_all(b"GET /styles/site.css HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .unwrap();
            stream.read_to_end(&mut response).unwrap();
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    host.kill().unwrap();
    host.wait().unwrap();
    let response = String::from_utf8(response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("rebeccapurple"));
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

#[cfg(unix)]
#[test]
fn fresh_scaffold_needs_no_configuration_source_list_or_component_identity() {
    let root = std::env::temp_dir().join(format!(
        "presolve-fresh-scaffold-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let application = root.join("hello-presolve");
    let scaffold = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/create-presolve/bin/create-presolve.mjs");
    let created = Command::new("node")
        .arg(scaffold)
        .arg(&application)
        .output()
        .unwrap();
    assert!(
        created.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&created.stderr)
    );
    let executable = application.join("node_modules/.bin/presolve-typescript-authority");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        r#"#!/usr/bin/env node
import { readFileSync } from "node:fs";
const request = JSON.parse(readFileSync(0, "utf8"));
const identity = name => ({ name, flags: 32, declarationModules: ["presolve"] });
process.stdout.write(JSON.stringify({
  schemaVersion: 13,
  diagnostics: [],
  components: request.components.map(site => ({ id: site.id, identity: identity("Component") })),
  states: request.states.map(site => ({ id: site.id, identity: identity("state") })),
  actions: request.actions.map(site => ({ id: site.id, identity: identity("action") })),
  effects: request.effects.map(site => ({ id: site.id, identity: identity("effect") })),
  slots: request.slots.map(site => ({ id: site.id, identity: identity("slot") })),
  environmentPublic: request.environmentPublic.map(site => ({ id: site.id, identity: identity("public") })),
}));
"#,
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let check = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("check")
        .current_dir(&application)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    let build = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .arg("build")
        .current_dir(&application)
        .output()
        .unwrap();
    assert!(build.status.success());
    let deploy = Command::new(env!("CARGO_BIN_EXE_presolve"))
        .args(["deploy", "cloudflare", "--prepare"])
        .current_dir(&application)
        .output()
        .unwrap();
    assert!(
        deploy.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&deploy.stderr)
    );
    assert!(application.join("dist/routes/root/index.html").is_file());
    let html =
        fs::read_to_string(application.join("dist/routes/root/index.html")).expect("starter HTML");
    assert!(html.contains("Ship the page. Resume the behavior."));
    assert!(html.contains("Interactive proof"));
    assert!(html.contains("width=device-width"));
    assert!(html.contains("href=\"/favicon.svg\""));
    assert!(html.contains("href=\"/app."));
    assert!(!html.contains("<main><main"));
    assert!(application.join("dist/app.css").is_file());
    assert!(application.join("dist/favicon.svg").is_file());
    let css = fs::read_to_string(application.join("dist/app.css")).expect("starter CSS");
    assert!(css.contains("@media (min-width: 48rem)"));
    assert!(css.contains("prefers-reduced-motion"));
    let component_artifact =
        fs::read_to_string(application.join("dist/routes/root/component.runtime.json"))
            .expect("starter component artifact");
    assert!(component_artifact.contains("increment"));
    assert!(component_artifact.contains("action_batch_id"));
    assert!(application
        .join(".presolve/cloudflare/deployment.plan.json")
        .is_file());
    assert!(!application.join("presolve.json").exists());
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

#[test]
fn dev_rebuilds_routes_and_hot_swaps_css_from_compiler_publications() {
    let root = project_root("dev-live-update");
    fs::write(
        root.join("app/index.html"),
        "<!doctype html><html><head>{{ head }}</head><body>{{ app }}{{ runtime }}</body></html>",
    )
    .unwrap();
    fs::write(root.join("app/app.css"), "body { color: red; }\n").unwrap();
    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Before edit</main>; } }"#,
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

    let initial = (0..200)
        .find_map(|_| {
            let response = development_get(port, "/");
            if response
                .as_ref()
                .is_some_and(|value| value.contains("Before edit"))
            {
                response
            } else {
                thread::sleep(Duration::from_millis(25));
                None
            }
        })
        .expect("development page became available");
    assert!(initial.contains("Cache-Control: no-store"));
    assert!(initial.contains("/__presolve/dev-client.js?revision=0"));
    assert!(development_get(port, "/__presolve/dev-client.js")
        .is_some_and(|response| response.contains("document.currentScript.src")
            && response.contains("/__presolve/dev-state")));

    fs::write(root.join("app/app.css"), "body { color: blue; }\n").unwrap();
    let style_update = (0..240)
        .find_map(|_| {
            let response = development_get(port, "/__presolve/dev-state")?;
            let body = response.split("\r\n\r\n").nth(1)?;
            let update: serde_json::Value = serde_json::from_str(body).ok()?;
            if update["revision"].as_u64().is_some_and(|value| value > 0)
                && update["updateKind"] == "style-update"
                && update["status"] == "ready"
            {
                Some(update)
            } else {
                thread::sleep(Duration::from_millis(25));
                None
            }
        })
        .expect("CSS edit produced a style update");
    let style_revision = style_update["revision"].as_u64().unwrap();
    assert!(
        development_get(port, "/app.css").is_some_and(|response| response.contains("color: blue"))
    );
    thread::sleep(Duration::from_millis(400));
    let settled_style_state = development_get(port, "/__presolve/dev-state").unwrap();
    let settled_style_state: serde_json::Value = serde_json::from_str(
        settled_style_state
            .split("\r\n\r\n")
            .nth(1)
            .expect("development state body"),
    )
    .unwrap();
    assert_eq!(settled_style_state["revision"], style_revision);
    assert_eq!(settled_style_state["updateKind"], "style-update");

    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>After edit</main>; } }"#,
    )
    .unwrap();
    let semantic_update = (0..240)
        .find_map(|_| {
            let response = development_get(port, "/__presolve/dev-state")?;
            let body = response.split("\r\n\r\n").nth(1)?;
            let update: serde_json::Value = serde_json::from_str(body).ok()?;
            if update["revision"]
                .as_u64()
                .is_some_and(|value| value > style_revision)
                && update["updateKind"] == "full-reload"
                && update["status"] == "ready"
            {
                Some(update)
            } else {
                thread::sleep(Duration::from_millis(25));
                None
            }
        })
        .expect("TSX edit produced a safe full reload");
    let semantic_revision = semantic_update["revision"].as_u64().unwrap();
    assert!(development_get(port, "/").is_some_and(|response| response.contains("After edit")));

    fs::write(
        root.join("app/routes/index.tsx"),
        "@component() class Home extends Component { render(",
    )
    .unwrap();
    let failed_update = (0..240)
        .find_map(|_| {
            let response = development_get(port, "/__presolve/dev-state")?;
            let body = response.split("\r\n\r\n").nth(1)?;
            let update: serde_json::Value = serde_json::from_str(body).ok()?;
            if update["revision"]
                .as_u64()
                .is_some_and(|value| value > semantic_revision)
                && update["updateKind"] == "diagnostic-update"
                && update["status"] == "error"
                && update["error"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            {
                Some(update)
            } else {
                thread::sleep(Duration::from_millis(25));
                None
            }
        })
        .expect("invalid edit produced a development diagnostic");
    let failed_revision = failed_update["revision"].as_u64().unwrap();
    assert!(development_get(port, "/").is_some_and(|response| response.contains("After edit")));

    fs::write(
        root.join("app/routes/index.tsx"),
        r#"@component() class Home extends Component { render() { return <main>Recovered</main>; } }"#,
    )
    .unwrap();
    (0..240)
        .find_map(|_| {
            let response = development_get(port, "/__presolve/dev-state")?;
            let body = response.split("\r\n\r\n").nth(1)?;
            let update: serde_json::Value = serde_json::from_str(body).ok()?;
            if update["revision"]
                .as_u64()
                .is_some_and(|value| value > failed_revision)
                && update["updateKind"] == "full-reload"
                && update["status"] == "ready"
            {
                Some(())
            } else {
                thread::sleep(Duration::from_millis(25));
                None
            }
        })
        .expect("valid edit recovered the development server");
    assert!(development_get(port, "/").is_some_and(|response| response.contains("Recovered")));

    child.kill().unwrap();
    child.wait().unwrap();
    fs::remove_dir_all(root).unwrap();
}
