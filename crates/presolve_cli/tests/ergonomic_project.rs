use std::fs;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

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
  schemaVersion: 9,
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
    let forms: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("dist/routes/root/forms.runtime.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(forms["schema_version"], 5);
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
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"/app.css?v=36709dfafff32d5ed90c36b3b50c450fe2d484fdb88eb49abcb5f6b17cbff2c8\">"
    ));
    assert!(html.contains("app-shell"));
    assert!(!html.contains("<main><main"));
    assert_eq!(
        fs::read_to_string(root.join("dist/app.css")).unwrap(),
        ".app-shell { min-height: 100vh; }\n"
    );
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
  @serverAction("savePost") save(): void {}
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
  schemaVersion: 9,
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
