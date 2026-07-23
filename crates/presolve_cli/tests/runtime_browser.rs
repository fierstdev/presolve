use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard, OnceLock, PoisonError,
};
use std::thread;
use std::time::{Duration, Instant};

static BROWSER_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn browser_test_guard() -> MutexGuard<'static, ()> {
    BROWSER_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize repo root")
}

fn presolve_cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_presolve"))
}

fn chrome_bin() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PRESOLVE_CHROME") {
        let path = PathBuf::from(path);

        if path.is_file() {
            return Some(path);
        }
    }

    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/opt/google/chrome/chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];

    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn run_chrome_probe(chrome: PathBuf, user_data_dir: &str, probe_url: &str) -> Output {
    run_chrome_probe_with_timeout(chrome, user_data_dir, probe_url, Duration::from_secs(20))
}

fn run_chrome_probe_with_timeout(
    chrome: PathBuf,
    user_data_dir: &str,
    probe_url: &str,
    timeout: Duration,
) -> Output {
    let mut args = vec![
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
    ];

    if std::env::var_os("CI").is_some() {
        args.push("--no-sandbox".to_string());
        args.push("--disable-dev-shm-usage".to_string());
    }

    args.push(user_data_dir.to_string());
    args.push(probe_url.to_string());

    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_chrome_with_timeout(chrome, &arg_refs, timeout)
}

#[test]
fn component_runtime_consumes_only_compiler_plans_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/component-runtime");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0065-component-runtime/input/RuntimeComponents.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build component runtime fixture");
    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_component_runtime_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);
    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_COMPONENT_RUNTIME_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout tail:\n{}\nstderr:\n{}",
        output.status,
        stdout
            .get(stdout.len().saturating_sub(4_000)..)
            .unwrap_or(&stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn phase_k_production_artifact_runs_under_csp_and_rejects_malformed_boot_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/phase-k-production");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean Phase K browser output");
    }
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0047-computed-diamond/input/ComputedDiamond.tsx",
            "--out",
            out_dir.to_str().expect("Phase K browser output UTF-8"),
            "--production",
        ])
        .output()
        .expect("failed to build Phase K production browser fixture");
    assert!(
        output.status.success(),
        "Phase K production build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_phase_k_production_probe_pages(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let run = |page: &str, profile: &str| {
        let profile_dir = out_dir.join(profile);
        fs::create_dir_all(&profile_dir).expect("failed to create Phase K Chrome profile");
        run_chrome_probe(
            chrome.clone(),
            &format!("--user-data-dir={}", profile_dir.to_string_lossy()),
            &format!("http://127.0.0.1:{}/{page}", server.port),
        )
    };
    let production = run("production-csp.html", "chrome-production");
    let malformed = run("production-malformed.html", "chrome-malformed");
    server.stop();

    assert!(
        String::from_utf8_lossy(&production.stdout)
            .contains("PRESOLVE_PHASE_K_PRODUCTION_BROWSER_PASS"),
        "production CSP probe failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        production.status,
        String::from_utf8_lossy(&production.stdout),
        String::from_utf8_lossy(&production.stderr)
    );
    assert!(
        String::from_utf8_lossy(&malformed.stdout)
            .contains("PRESOLVE_PHASE_K_MALFORMED_BROWSER_PASS"),
        "malformed production probe failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        malformed.status,
        String::from_utf8_lossy(&malformed.stdout),
        String::from_utf8_lossy(&malformed.stderr)
    );
}

fn write_phase_k_production_probe_pages(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("Phase K production page");
    let csp = index
        .replace(
            "<head>",
            "<head>\n    <meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'self'; object-src 'none'; base-uri 'none'\">",
        )
        .replace(
            "</body>",
            "    <script src=\"./phase-k-production-probe.js\" defer></script>\n  </body>",
        );
    fs::write(out_dir.join("production-csp.html"), csp).expect("write Phase K production CSP page");

    let module_names = fs::read_dir(out_dir.join("production"))
        .expect("Phase K production module directory")
        .map(|entry| {
            entry
                .expect("Phase K module entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    let module_names = serde_json::to_string(&module_names).expect("module filename JSON");
    let probe = format!(
        r#"const fail=(message)=>{{throw new Error(message)}};
const waitFor=(predicate,label)=>new Promise((resolve,reject)=>{{const deadline=Date.now()+3000;const tick=()=>{{if(predicate()){{resolve();return}}if(Date.now()>deadline){{reject(new Error(`Timed out waiting for ${{label}}`));return}}setTimeout(tick,20)}};tick()}});
(async()=>{{
  await waitFor(()=>["ready","error"].includes(document.documentElement.dataset.presolveRuntime),"production runtime");
  const runtime=window.__PRESOLVE__;
  if(document.documentElement.dataset.presolveRuntime==="error")fail(`production runtime failed: ${{JSON.stringify(runtime.diagnostics)}}`);
  if(runtime.production===null||runtime.production.table_kinds.length!==6)fail("packed ordinal tables were not installed exactly");
  if(new Set(runtime.production.table_kinds).size!==runtime.production.table_kinds.length)fail("duplicate production table installation");
  const before=Object.values(runtime.store).filter(value=>value instanceof Map||value instanceof Set).map(value=>value.size).join(",");
  const host=document.querySelector("main")??document.body;
  for(let cycle=0;cycle<100;cycle+=1){{const clone=host.cloneNode(true);document.body.appendChild(clone);clone.remove();await Promise.resolve()}}
  const after=Object.values(runtime.store).filter(value=>value instanceof Map||value instanceof Set).map(value=>value.size).join(",");
  if(after!==before)fail("compiler-owned registry counts grew across 100 create/remove cycles");
  const sources=await Promise.all(["runtime.js",...{module_names}.map(name=>`production/${{name}}`)].map(path=>fetch(path).then(response=>response.text())));
  if(sources.some(source=>source.includes("eval(")||source.includes("Function(")))fail("CSP-unsafe dynamic code was emitted");
  document.body.insertAdjacentHTML("beforeend","<div>PRESOLVE_PHASE_K_PRODUCTION_BROWSER_PASS</div>");
}})().catch(error=>{{document.body.insertAdjacentHTML("beforeend",`<div>PRESOLVE_PHASE_K_PRODUCTION_BROWSER_FAIL: ${{error.message}}</div>`);console.error(error)}});
"#
    );
    fs::write(out_dir.join("phase-k-production-probe.js"), probe)
        .expect("write Phase K production probe");

    let malformed = index
        .replacen("\"schemaVersion\":1", "\"schemaVersion\":2", 1)
        .replace(
            "</body>",
            "    <script src=\"./phase-k-malformed-probe.js\" defer></script>\n  </body>",
        );
    fs::write(out_dir.join("production-malformed.html"), malformed)
        .expect("write malformed production page");
    fs::write(
        out_dir.join("phase-k-malformed-probe.js"),
        r#"const wait=setInterval(()=>{if(document.documentElement.dataset.presolveRuntime!=="error")return;clearInterval(wait);const runtime=window.__PRESOLVE__;if(runtime.production!==undefined&&runtime.production!==null){document.body.insertAdjacentHTML("beforeend","<div>PRESOLVE_PHASE_K_MALFORMED_BROWSER_FAIL</div>");return}document.body.insertAdjacentHTML("beforeend","<div>PRESOLVE_PHASE_K_MALFORMED_BROWSER_PASS</div>")},20);"#,
    )
    .expect("write malformed production probe");
}

#[test]
fn repeated_component_state_and_computed_updates_stay_instance_qualified_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/state-instance-storage");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean State instance browser output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create State instance browser output");
    let input = out_dir.join("RepeatedCounter.tsx");
    fs::write(
        &input,
        r#"
@component("x-repeated-counter") class RepeatedCounter extends Component {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  @action() increment() { this.count += 1; }
  render() { return <button onClick={this.increment}>{this.count}</button>; }
}
@component("x-repeated-page") @route("/") class RepeatedPage extends Component {
  render() { return <main><RepeatedCounter /><RepeatedCounter /></main>; }
}"#,
    )
    .expect("failed to write repeated State browser source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build repeated State browser source");
    assert!(
        output.status.success(),
        "repeated State build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_state_instance_storage_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_STATE_INSTANCE_STORAGE_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn phase_j_rejects_component_v2_while_legacy_v2_cold_boot_remains_accepted() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/state-instance-compatibility");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean State compatibility output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create State compatibility output");
    let input = out_dir.join("StaticPage.tsx");
    fs::write(
        &input,
        r#"
@component("x-static-page") @route("/") class StaticPage extends Component {
  render() { return <main>Static</main>; }
}"#,
    )
    .expect("failed to write State compatibility source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build State compatibility source");
    assert!(
        output.status.success(),
        "State compatibility build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_state_instance_compatibility_probe_pages(&out_dir);

    for (index, (file_name, marker)) in [
        (
            "phase-j-component-v2.html",
            "PRESOLVE_PHASE_J_COMPONENT_V2_REJECTION_PASS",
        ),
        (
            "legacy-component-v2-cold.html",
            "PRESOLVE_LEGACY_COMPONENT_V2_COLD_PASS",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let server = StaticServer::start(out_dir.clone());
        let chrome = chrome_bin().expect("headless Chrome was not found");
        let profile_dir = out_dir.join(format!("chrome-profile-{index}"));
        fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
        let output = run_chrome_probe(
            chrome,
            &format!("--user-data-dir={}", profile_dir.display()),
            &format!("http://127.0.0.1:{}/{file_name}", server.port),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        server.stop();
        assert!(
            stdout.contains(marker),
            "browser probe did not pass for {file_name}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn explicit_form_hosts_submit_only_through_compiler_emitted_records() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/form-submission-host");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean Forms browser output");
    }
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "framework/tests/forms-types/src/FormHost.tsx",
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build Forms browser source");
    assert!(
        output.status.success(),
        "Forms build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_form_submission_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_FORM_SUBMISSION_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn resume_bootstrap_registry_accepts_and_atomically_falls_back_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-bootstrap");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean resume bootstrap output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create resume bootstrap output");
    let input = out_dir.join("ResumeCounter.tsx");
    fs::write(
        &input,
        r#"
@component("resume-counter") class ResumeCounter {
  count = state(1);
  @action() increment() { this.count += 1; }
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
}"#,
    )
    .expect("failed to write resume bootstrap source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build resume bootstrap source");
    assert!(
        output.status.success(),
        "resume bootstrap build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_resume_bootstrap_probe_pages(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    for (index, (page, marker)) in [
        ("cold.html", "PRESOLVE_RESUME_COLD_PASS"),
        ("accepted.html", "PRESOLVE_RESUME_ACCEPTED_PASS"),
        ("build-mismatch.html", "PRESOLVE_RESUME_BUILD_FALLBACK_PASS"),
        (
            "protocol-mismatch.html",
            "PRESOLVE_RESUME_ARTIFACT_FALLBACK_PASS",
        ),
        (
            "malformed-snapshot.html",
            "PRESOLVE_RESUME_SNAPSHOT_FALLBACK_PASS",
        ),
        ("value-mismatch.html", "PRESOLVE_RESUME_VALUE_FALLBACK_PASS"),
    ]
    .into_iter()
    .enumerate()
    {
        let profile_dir = out_dir.join(format!("chrome-profile-{index}"));
        fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
        let output = run_chrome_probe(
            chrome.clone(),
            &format!("--user-data-dir={}", profile_dir.display()),
            &format!("http://127.0.0.1:{}/{page}", server.port),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(marker),
            "resume bootstrap probe failed for {page}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    server.stop();
}

#[test]
fn resume_restores_repeated_state_and_recomputes_computed_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-state-computed");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean resume State output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create resume State output");
    let input = out_dir.join("RepeatedComputed.tsx");
    fs::write(
        &input,
        r#"
@component("resume-child") class ResumeChild {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  @action() increment() { this.count += 1; }
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
}
@component("resume-parent") @route("/") class ResumeParent {
  render() { return <main><ResumeChild /><ResumeChild /></main>; }
}"#,
    )
    .expect("failed to write repeated resume source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build repeated resume source");
    assert!(
        output.status.success(),
        "repeated resume build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_resume_state_computed_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_RESUME_STATE_COMPUTED_PASS"),
        "resume State/Computed probe failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn resume_restores_exact_nested_context_bindings_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-context");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean resume Context output");
    }
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0064-component-instance-context/input/InstanceContext.tsx",
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build resume Context fixture");
    assert!(
        output.status.success(),
        "resume Context build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_resume_context_probe_pages(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_RESUME_CONTEXT_PASS"),
        "resume Context probe failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn resume_context_binding_mismatch_falls_back_without_reselection() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-context-mismatch");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean Context mismatch output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create Context mismatch output");
    let input = out_dir.join("ResumeContextFallback.tsx");
    fs::write(
        &input,
        r#"
@component("resume-context-fallback")
class ResumeContextFallback extends Component {
  @context() shared!: string;
  @provide(ResumeContextFallback.shared) provided: string = "cold";
  @consume(ResumeContextFallback.shared) selected!: string;
  render() { return <main />; }
}"#,
    )
    .expect("failed to write Context mismatch source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build Context mismatch fixture");
    assert!(output.status.success(), "Context mismatch fixture failed");
    write_resume_context_mismatch_probe(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_RESUME_CONTEXT_FALLBACK_PASS"),
        "resume Context fallback probe failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn resume_restores_components_and_structural_state_without_dom_reconstruction() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-structure");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean resume structure output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create resume structure output");
    let input = out_dir.join("ResumeStructure.tsx");
    let source = fs::read_to_string(
        repo_root.join("fixtures/0065-component-runtime/input/StructuralComponents.tsx"),
    )
    .expect("failed to read structural fixture")
    .replace("  toggle()", "  @action() toggle()")
    .replace("  reconcile()", "  @action() reconcile()")
    .replace("  trim()", "  @action() trim()");
    fs::write(&input, source).expect("failed to write resume structure source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build resume structure fixture");
    assert!(output.status.success(), "resume structure fixture failed");
    write_resume_structure_probe_pages(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    for (index, (page, marker)) in [
        ("probe.html", "PRESOLVE_RESUME_STRUCTURE_PASS"),
        (
            "state-mismatch.html",
            "PRESOLVE_RESUME_STRUCTURE_FALLBACK_PASS",
        ),
        (
            "missing-anchor.html",
            "PRESOLVE_RESUME_ANCHOR_FALLBACK_PASS",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let profile_dir = out_dir.join(format!("chrome-profile-{index}"));
        fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
        let output = run_chrome_probe(
            chrome.clone(),
            &format!("--user-data-dir={}", profile_dir.display()),
            &format!("http://127.0.0.1:{}/{page}", server.port),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(marker),
            "resume structure probe failed for {page}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    server.stop();
}

#[test]
fn resume_restores_exact_slot_bindings_without_component_initialization() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-slots");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean resume Slot output");
    }
    fs::create_dir_all(&out_dir).expect("failed to create resume Slot output");
    let input = out_dir.join("ResumeSlots.tsx");
    let source = fs::read_to_string(
        repo_root.join("fixtures/0065-component-runtime/input/RuntimeComponents.tsx"),
    )
    .expect("failed to read Slot fixture")
    .replace("  increment()", "  @action() increment()");
    fs::write(&input, source).expect("failed to write resume Slot source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            input.to_str().expect("input UTF-8"),
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build resume Slot fixture");
    assert!(output.status.success(), "resume Slot fixture failed");
    write_resume_slot_probe_page(&out_dir);

    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
    let probe_url = format!("file://{}", out_dir.join("probe.html").display());
    let output = run_chrome_probe_with_timeout(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &probe_url,
        Duration::from_secs(45),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("PRESOLVE_RESUME_SLOT_PASS"),
        "resume Slot probe failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn resume_restores_compiler_owned_form_state_and_rejects_active_submission() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/resume-forms");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean resume Forms output");
    }
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "framework/tests/forms-resume-types/src/ResumeForms.tsx",
            "--out",
            out_dir.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("failed to build resume Forms fixture");
    assert!(output.status.success(), "resume Forms fixture failed");
    write_resume_form_probe_pages(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    for (index, (page, marker)) in [
        ("probe.html", "PRESOLVE_RESUME_FORMS_PASS"),
        ("submitting.html", "PRESOLVE_RESUME_FORMS_FALLBACK_PASS"),
    ]
    .into_iter()
    .enumerate()
    {
        let profile_dir = out_dir.join(format!("chrome-profile-{index}"));
        fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile");
        let output = run_chrome_probe(
            chrome.clone(),
            &format!("--user-data-dir={}", profile_dir.display()),
            &format!("http://127.0.0.1:{}/{page}", server.port),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(marker),
            "resume Forms probe failed for {page}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    server.stop();
}

fn write_resume_state_computed_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let resume_json =
        fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest");
    let manifest: serde_json::Value =
        serde_json::from_str(&resume_json).expect("resume manifest JSON");
    let mut snapshot = resume_bootstrap_snapshot(&manifest);
    let state_slots = manifest["slot_schemas"]
        .as_array()
        .expect("slot schemas")
        .iter()
        .filter(|slot| slot["restore_phase"] == "R3")
        .map(|slot| slot["slot_id"].as_str().expect("State slot").to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut next = [7, 11].into_iter();
    for boundary in snapshot["boundaries"]
        .as_array_mut()
        .expect("snapshot boundaries")
    {
        for record in boundary["values"].as_array_mut().expect("snapshot values") {
            if state_slots.contains(record["slotId"].as_str().expect("snapshot slot")) {
                record["value"] = serde_json::Value::from(next.next().expect("State value"));
            }
        }
    }
    assert!(
        next.next().is_none(),
        "expected exactly two repeated State slots"
    );
    let probe = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&snapshot).expect("snapshot JSON")
        ),
        r#"
if (runtime.resume.mode !== "resume") fail("snapshot was not resumed");
const states = runtime.components.filter((component) => component.state.count !== undefined).map((component) => component.state.count).sort((a, b) => a - b);
if (JSON.stringify(states) !== "[7,11]") fail("repeated State slots were not restored distinctly");
const computed = runtime.computed.map((entry) => entry.value).sort((a, b) => a - b);
if (JSON.stringify(computed) !== "[14,22]") fail("Computed slots did not recompute from exact repeated State");
if (runtime.computed.some((entry) => entry.dirty)) fail("Computed slots remained dirty");
if (runtime.resume_recomputation_runs.length !== 2 || new Set(runtime.resume_recomputation_runs).size !== 2) fail("Computed recomputation did not execute exactly once per instance");
if (runtime.initial_effect_runs.length !== 0) fail("Effects ran during R3-R5");
if (runtime.diagnostics.some((diagnostic) => diagnostic.fatal)) fail("resume reported a fatal diagnostic");"#,
        "PRESOLVE_RESUME_STATE_COMPUTED_PASS",
    );
    fs::write(out_dir.join("probe.html"), probe).expect("resume State probe");
}

fn write_resume_slot_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let resume_json =
        fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest");
    let manifest: serde_json::Value =
        serde_json::from_str(&resume_json).expect("resume manifest JSON");
    let component_json =
        fs::read_to_string(out_dir.join("component.runtime.json")).expect("component artifact");
    let component: serde_json::Value =
        serde_json::from_str(&component_json).expect("component artifact JSON");
    let mut snapshot = resume_bootstrap_snapshot(&manifest);
    set_snapshot_state_to_compiled_initials(&manifest, &component, &mut snapshot);
    let probe = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&snapshot).expect("snapshot JSON")
        ),
        r#"
if (runtime.resume?.mode !== "resume") fail("Slot snapshot was not resumed: " + JSON.stringify(runtime.diagnostics));
const artifact = runtime.store.componentArtifact;
const expected = new Map(artifact.slot_binding_programs.map((binding) => [binding.binding, binding]));
if (runtime.store.slotBindings.size !== expected.size || expected.size !== 2) fail("exact Slot bindings were not installed");
for (const [bindingId, binding] of expected) {
  const installed = runtime.store.slotBindings.get(bindingId);
  if (installed?.caller_instance !== binding.caller_instance
    || installed?.callee_instance !== binding.callee_instance
    || installed?.content_owner_instance !== binding.content_owner_instance) {
    fail("Slot caller, callee, or content owner was reselected");
  }
}
if (runtime.resume_registry.component_records.size !== runtime.component_instance_tree.length) fail("component runtime records were incomplete");
if (runtime.component_initialization_runs.length !== 0 || runtime.slot_binding_runs.length !== 0) fail("resume executed component or Slot initialization");
if (runtime.initial_effect_runs.length !== 0) fail("resume executed cold effects");
if (document.querySelector("main").innerHTML !== window.__presolveInitialHtml) fail("resume reconstructed component DOM");"#,
        "PRESOLVE_RESUME_SLOT_PASS",
    )
    .replace(
        "window.__PRESOLVE_RESUME_SNAPSHOT__",
        "window.__presolveInitialHtml = document.querySelector(\"main\").innerHTML;\nwindow.__PRESOLVE_RESUME_SNAPSHOT__",
    );
    fs::write(out_dir.join("probe.html"), probe).expect("resume Slot probe");
}

fn write_resume_form_probe_pages(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let resume_json =
        fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest");
    let manifest: serde_json::Value =
        serde_json::from_str(&resume_json).expect("resume manifest JSON");
    let component_json =
        fs::read_to_string(out_dir.join("component.runtime.json")).expect("component artifact");
    let component: serde_json::Value =
        serde_json::from_str(&component_json).expect("component artifact JSON");
    let mut snapshot = resume_bootstrap_snapshot(&manifest);
    set_snapshot_state_to_compiled_initials(&manifest, &component, &mut snapshot);
    for record in snapshot["boundaries"]
        .as_array_mut()
        .expect("boundaries")
        .iter_mut()
        .flat_map(|boundary| {
            boundary["values"]
                .as_array_mut()
                .expect("values")
                .iter_mut()
        })
    {
        let schema = manifest["slot_schemas"]
            .as_array()
            .expect("slot schemas")
            .iter()
            .find(|schema| schema["slot_id"] == record["slotId"])
            .expect("slot schema");
        record["value"] = match schema["restore_phase"].as_str() {
            Some("R11") => serde_json::Value::String("resumed-name".to_string()),
            Some("R12") => serde_json::Value::Bool(true),
            Some("R13") => match schema["semantic_type"].as_str() {
                Some("boolean") => serde_json::Value::Bool(true),
                _ => serde_json::Value::Array(Vec::new()),
            },
            Some("R14") => serde_json::Value::String("Completed".to_string()),
            _ => record["value"].clone(),
        };
    }
    let snapshot_json = serde_json::to_string(&snapshot).expect("snapshot JSON");
    let probe = resume_bootstrap_probe_page(
        &index,
        &format!("window.__PRESOLVE_RESUME_SNAPSHOT__ = {snapshot_json};"),
        r#"
if (runtime.resume?.mode !== "resume") fail("Form snapshot was not resumed: " + JSON.stringify({resume: runtime.resume, diagnostics: runtime.diagnostics}));
if (runtime.forms.length !== 1 || runtime.forms[0].submission !== "Completed" || runtime.forms[0].aggregate_valid !== true) fail("stable Form state was not restored");
const instance = runtime.store.formInstances.values().next().value;
const field = instance.fields.values().next().value;
if (field.value !== "resumed-name" || field.dirty !== true || field.touched !== true || field.validation.length !== 0) fail("Field slots were not restored exactly");
if (document.querySelector("input").value !== "resumed-name") fail("exact Form control was not synchronized");
if (!window.__PRESOLVE_FORMS__.resetForm(instance.instance.id)) fail("reset plan was not installed");
if (field.value !== "" || field.dirty || field.touched || field.validation.length !== 0 || document.querySelector("input").value !== "") fail("reset did not use compiled initial Form values");"#,
        "PRESOLVE_RESUME_FORMS_PASS",
    );
    fs::write(out_dir.join("probe.html"), probe).expect("resume Forms probe");

    let mut submitting = snapshot;
    let record = submitting["boundaries"]
        .as_array_mut()
        .expect("boundaries")
        .iter_mut()
        .flat_map(|boundary| {
            boundary["values"]
                .as_array_mut()
                .expect("values")
                .iter_mut()
        })
        .find(|record| {
            manifest["slot_schemas"]
                .as_array()
                .expect("slot schemas")
                .iter()
                .find(|schema| schema["slot_id"] == record["slotId"])
                .is_some_and(|schema| schema["restore_phase"] == "R14")
        })
        .expect("submission slot");
    record["value"] = serde_json::Value::String("Submitting".to_string());
    let submitting_page = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&submitting).expect("submitting snapshot")
        ),
        r#"
if (runtime.resume.mode !== "cold" || runtime.resume.failure !== "UnstableFormSubmission") fail("active Form submission did not cold fall back");
if (runtime.resume_registry !== null || runtime.forms[0].submission !== "Idle") fail("active Form submission retained resume state");"#,
        "PRESOLVE_RESUME_FORMS_FALLBACK_PASS",
    );
    fs::write(out_dir.join("submitting.html"), submitting_page).expect("submitting Forms probe");
}

fn write_resume_context_probe_pages(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let resume_json =
        fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest");
    let manifest: serde_json::Value =
        serde_json::from_str(&resume_json).expect("resume manifest JSON");
    let mut snapshot = resume_bootstrap_snapshot(&manifest);
    let values = ["restored-default", "restored-light", "restored-nearest"];
    let mut expected = serde_json::Map::new();
    let mut next = values.into_iter();
    for boundary in snapshot["boundaries"]
        .as_array_mut()
        .expect("snapshot boundaries")
    {
        for record in boundary["values"].as_array_mut().expect("snapshot values") {
            let slot_id = record["slotId"].as_str().expect("snapshot slot");
            let schema = manifest["slot_schemas"]
                .as_array()
                .expect("slot schemas")
                .iter()
                .find(|schema| schema["slot_id"] == slot_id)
                .expect("slot schema");
            if schema["restore_phase"] != "R6" {
                continue;
            }
            let value = next.next().expect("Context restore value");
            record["value"] = serde_json::Value::String(value.to_string());
            expected.insert(
                schema["existing_storage_slot_id"]
                    .as_str()
                    .expect("runtime slot")
                    .to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }
    assert!(next.next().is_none(), "expected three Context value slots");
    let prelude = format!(
        "window.__PRESOLVE_RESUME_SNAPSHOT__ = {}; window.__J13_EXPECTED__ = {};",
        serde_json::to_string(&snapshot).expect("snapshot JSON"),
        serde_json::to_string(&expected).expect("expected Context JSON")
    );
    let probe = resume_bootstrap_probe_page(
        &index,
        &prelude,
        r#"
if (runtime.resume?.mode !== "resume") fail("Context snapshot was not resumed: " + JSON.stringify({resume: runtime.resume, diagnostics: runtime.diagnostics}));
if (runtime.context_initial_source_runs.length !== 0) fail("Context evaluator ran during R6-R7");
if (runtime.context_slots.length !== 3) fail("restored Context slots were incomplete");
if (runtime.context_consumer_bindings.length !== 3) fail("exact Consumer bindings were incomplete");
if (runtime.resume_registry.context_bindings.size !== 3) fail("Context registry was incomplete");
const expected = new Map(Object.entries(window.__J13_EXPECTED__));
for (const [slot, value] of runtime.context_slots) {
  if (expected.get(slot) !== value) fail("Context value did not use its exact instance slot");
}
const componentArtifact = JSON.parse(document.getElementById("presolve-component-runtime").textContent);
for (const binding of componentArtifact.instance_context_bindings) {
  const installedSlot = runtime.store.contextConsumerBindings.get(binding.consumer_instance);
  const installed = runtime.resume_registry.context_bindings.get(binding.consumer_instance);
  if (installedSlot !== binding.runtime_slot || installed.value_slot_id !== binding.runtime_slot) fail("Consumer slot was reselected");
  if (installed.selected_source !== binding.selected_source) fail("Provider source was reselected");
  if (installed.provider_instance_id !== binding.provider_source) fail("Provider instance was reselected");
  if (runtime.store.contextSlots.get(installedSlot) !== expected.get(installedSlot)) fail("Consumer did not observe the exact restored Provider slot");
}
if (runtime.context_failures.length !== 0) fail("Context restore reported failures");
if (runtime.initial_effect_runs.length !== 0) fail("Effect ran during Context restore");"#,
        "PRESOLVE_RESUME_CONTEXT_PASS",
    );
    fs::write(out_dir.join("probe.html"), probe).expect("resume Context probe");
}

fn write_resume_context_mismatch_probe(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest"),
    )
    .expect("resume manifest JSON");
    let snapshot = resume_bootstrap_snapshot(&manifest);
    let mismatch = replace_json_script(&index, "presolve-resume-runtime", |value| {
        let instruction = value["restore_programs"]
            .as_array_mut()
            .expect("restore programs")
            .iter_mut()
            .flat_map(|program| {
                program["instructions"]
                    .as_array_mut()
                    .expect("restore instructions")
                    .iter_mut()
            })
            .find(|record| record["phase"] == "R7")
            .expect("R7 instruction");
        instruction["instruction"]["selected_source"] =
            serde_json::Value::String("context-source:invalid".to_string());
    });
    let probe = resume_bootstrap_probe_page(
        &mismatch,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&snapshot).expect("snapshot JSON")
        ),
        r#"
if (runtime.resume.mode !== "cold" || runtime.resume.failure !== "ResumeArtifactMismatch") fail("Context mismatch did not fall back atomically");
if (runtime.resume_registry !== null) fail("Context mismatch retained a partial registry");
if (runtime.context_initial_source_runs.length !== 1 || runtime.context_slots.length !== 1) fail("cold fallback did not initialize Context once");
if (runtime.context_consumer_bindings.length !== 1) fail("cold fallback reselected or lost Consumer binding");"#,
        "PRESOLVE_RESUME_CONTEXT_FALLBACK_PASS",
    );
    fs::write(out_dir.join("probe.html"), probe).expect("Context fallback probe");
}

fn write_resume_structure_probe_pages(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest"),
    )
    .expect("resume manifest JSON");
    let component: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(out_dir.join("component.runtime.json")).expect("component artifact"),
    )
    .expect("component artifact JSON");
    let mut snapshot = resume_bootstrap_snapshot(&manifest);
    set_snapshot_state_to_compiled_initials(&manifest, &component, &mut snapshot);
    let snapshot_json = serde_json::to_string(&snapshot).expect("snapshot JSON");

    let probe = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {snapshot_json}; window.__J14_DOM__ = document.querySelector('main').innerHTML;"
        ),
        r#"
if (runtime.resume === undefined) fail("structural boot failed: " + JSON.stringify({ diagnostics: runtime.diagnostics, resume: window.__PRESOLVE_RESUME__.debugEvidence() }));
if (runtime.resume.mode !== "resume") fail("structural snapshot was not resumed");
if (runtime.resume_registry.component_records.size !== 5) fail("Component runtime records were incomplete");
if (runtime.resume_registry.structural_records.size !== 2) fail("structural runtime records were incomplete");
if ([...runtime.resume_registry.structural_records.values()].some((record) => record.selection_value === undefined)) fail("structural selection was not restored");
if (runtime.store.resumeAnchors.size !== runtime.resume_registry.definitions.anchors.size) fail("exact resume anchors were not verified");
if (runtime.component_initialization_runs.length !== 0) fail("Component initialization ran during R8");
if (runtime.initial_effect_runs.length !== 0) fail("Effect ran during R8-R10");
if (document.querySelector("main").innerHTML !== window.__J14_DOM__) fail("resume reconstructed the DOM");"#,
        "PRESOLVE_RESUME_STRUCTURE_PASS",
    );
    fs::write(out_dir.join("probe.html"), probe).expect("resume structure probe");

    let mut mismatch = snapshot.clone();
    let record = mismatch["boundaries"]
        .as_array_mut()
        .expect("boundaries")
        .iter_mut()
        .flat_map(|boundary| {
            boundary["values"]
                .as_array_mut()
                .expect("values")
                .iter_mut()
        })
        .find(|record| {
            manifest["slot_schemas"]
                .as_array()
                .expect("slot schemas")
                .iter()
                .find(|slot| slot["slot_id"] == record["slotId"])
                .is_some_and(|slot| slot["semantic_type"] == "boolean")
        })
        .expect("boolean structural State");
    record["value"] = serde_json::Value::Bool(false);
    let mismatch_page = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&mismatch).expect("mismatch snapshot")
        ),
        &structural_fallback_assertions("StructuralStateMismatch"),
        "PRESOLVE_RESUME_STRUCTURE_FALLBACK_PASS",
    );
    fs::write(out_dir.join("state-mismatch.html"), mismatch_page)
        .expect("structural mismatch probe");

    let first_anchor = manifest["anchors"]
        .as_array()
        .expect("anchors")
        .iter()
        .find(|anchor| {
            matches!(
                anchor["kind"].as_str(),
                Some("structural_start" | "structural_end")
            )
        })
        .and_then(|anchor| anchor["anchor_id"].as_str())
        .expect("structural anchor ID");
    let missing = index.replace(&format!("<!--presolve-r-end:{first_anchor}-->"), "");
    let missing = missing.replace(&format!("<!--presolve-r-start:{first_anchor}-->"), "");
    assert_ne!(missing, index, "resume anchor comment was not emitted");
    let missing_page = resume_bootstrap_probe_page(
        &missing,
        &format!("window.__PRESOLVE_RESUME_SNAPSHOT__ = {snapshot_json};"),
        &structural_fallback_assertions("MissingAnchor"),
        "PRESOLVE_RESUME_ANCHOR_FALLBACK_PASS",
    );
    fs::write(out_dir.join("missing-anchor.html"), missing_page).expect("missing anchor probe");
}

fn set_snapshot_state_to_compiled_initials(
    manifest: &serde_json::Value,
    component: &serde_json::Value,
    snapshot: &mut serde_json::Value,
) {
    let initial_by_slot = component["instances"]
        .as_array()
        .expect("component instances")
        .iter()
        .flat_map(|instance| {
            instance["state_slots"]
                .as_array()
                .expect("State slots")
                .iter()
        })
        .map(|slot| {
            (
                slot["slot_id"].as_str().expect("State slot"),
                slot["initial_value"].clone(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for record in snapshot["boundaries"]
        .as_array_mut()
        .expect("snapshot boundaries")
        .iter_mut()
        .flat_map(|boundary| {
            boundary["values"]
                .as_array_mut()
                .expect("snapshot values")
                .iter_mut()
        })
    {
        let slot_id = record["slotId"].as_str().expect("snapshot slot");
        let schema = manifest["slot_schemas"]
            .as_array()
            .expect("slot schemas")
            .iter()
            .find(|schema| schema["slot_id"] == slot_id)
            .expect("slot schema");
        if schema["restore_phase"] != "R3" {
            continue;
        }
        let existing = schema["existing_storage_slot_id"]
            .as_str()
            .expect("existing slot");
        let mut value = initial_by_slot
            .get(existing)
            .expect("compiled State initial")
            .clone();
        if schema["semantic_type"] == "number" {
            let number = value
                .as_str()
                .expect("serialized numeric State initial")
                .parse::<f64>()
                .expect("numeric State initial");
            value = serde_json::Value::Number(
                serde_json::Number::from_f64(number).expect("finite numeric State initial"),
            );
        }
        record["value"] = value;
    }
}

fn structural_fallback_assertions(failure: &str) -> String {
    format!(
        r#"
if (runtime.resume.mode !== "cold" || runtime.resume.failure !== "{failure}") fail("structural rejection did not select one cold boot");
if (runtime.resume_registry !== null) fail("structural rejection retained a partial registry");
if (runtime.component_initialization_runs.length === 0) fail("cold fallback did not initialize Components");"#
    )
}

fn write_resume_bootstrap_probe_pages(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("built page");
    let resume_json =
        fs::read_to_string(out_dir.join("resume.runtime.json")).expect("resume manifest");
    let manifest: serde_json::Value =
        serde_json::from_str(&resume_json).expect("resume manifest JSON");
    let snapshot = resume_bootstrap_snapshot(&manifest);

    let cold = resume_bootstrap_probe_page(
        &index,
        "",
        r#"
if (runtime.resume === undefined) fail("runtime boot failed: " + JSON.stringify(runtime.diagnostics));
if (runtime.resume.mode !== "cold" || runtime.resume.failure !== null) fail("no-snapshot boot was not cold");
if (!(runtime.store?.components instanceof Map) || runtime.components[0].state.count !== 1) fail("cold runtime did not initialize");
if (runtime.resume_registry !== null) fail("cold boot retained a resume registry");
document.querySelector("button").click();
await waitFor(() => runtime.components[0].state.count === 2, "cold action");"#,
        "PRESOLVE_RESUME_COLD_PASS",
    );
    fs::write(out_dir.join("cold.html"), cold).expect("cold probe");

    let accepted = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&snapshot).expect("snapshot JSON")
        ),
        r#"
if (runtime.resume.mode !== "resume" || runtime.resume.failure !== null) fail("valid snapshot was not accepted");
if (!(runtime.resume_registry?.boundary_records instanceof Map)) fail("closed registry was not allocated");
if (runtime.resume_registry.boundary_records.size !== runtime.resume_registry.definitions.boundaries.size) fail("boundary records were incomplete");
if (runtime.resume_registry.slot_values.size !== 1) fail("restored State registry was incomplete");
if (!(runtime.store?.components instanceof Map) || runtime.components[0].state.count !== 7) fail("snapshot State was not restored");
if (document.querySelector("button").textContent !== "7") fail("R16 did not patch the exact text binding before Ready");
if (runtime.initial_effect_runs.length !== 0) fail("resume path executed cold authored initialization");
const eventId = runtime.resume_registry.definitions.events.keys().next().value;
const activation = await window.__PRESOLVE_RESUME__.activateByEvent(eventId);
if (activation.status !== "active") fail("event activation API did not load the exact chunk");
document.querySelector("button").click();
await waitFor(() => runtime.components[0].state.count === 8, "activated action");
if (window.__PRESOLVE_RESUME__.captureSnapshot().failure !== "NotQuiescent") fail("capture API contract was absent");
let doubleFailure = null;
try { await window.__PRESOLVE_RESUME__.bootstrapResume(); } catch (error) { doubleFailure = error.failure; }
if (doubleFailure !== "DoubleBootstrap") fail("double bootstrap was not rejected");"#,
        "PRESOLVE_RESUME_ACCEPTED_PASS",
    );
    fs::write(out_dir.join("accepted.html"), accepted).expect("accepted probe");

    let mut wrong_build = snapshot.clone();
    wrong_build["buildId"] = serde_json::Value::String(
        "resume-build:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
    );
    let rejected = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&wrong_build).expect("wrong build snapshot JSON")
        ),
        &fallback_assertions("BuildIdMismatch"),
        "PRESOLVE_RESUME_BUILD_FALLBACK_PASS",
    );
    fs::write(out_dir.join("build-mismatch.html"), rejected).expect("build fallback probe");

    let protocol_page = replace_json_script(&index, "presolve-resume-runtime", |value| {
        value["runtime_protocol_version"] = serde_json::Value::from(2);
    });
    let protocol_page = resume_bootstrap_probe_page(
        &protocol_page,
        "",
        &fallback_assertions("RuntimeProtocolMismatch"),
        "PRESOLVE_RESUME_ARTIFACT_FALLBACK_PASS",
    );
    fs::write(out_dir.join("protocol-mismatch.html"), protocol_page)
        .expect("artifact fallback probe");

    write_resume_value_mismatch_probe(out_dir, &index, &snapshot);

    let mut malformed = snapshot;
    malformed["unknown"] = serde_json::Value::Bool(true);
    let malformed_page = resume_bootstrap_probe_page(
        &index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&malformed).expect("malformed snapshot JSON")
        ),
        &fallback_assertions("SnapshotSchemaMismatch"),
        "PRESOLVE_RESUME_SNAPSHOT_FALLBACK_PASS",
    );
    fs::write(out_dir.join("malformed-snapshot.html"), malformed_page)
        .expect("snapshot fallback probe");
}

fn write_resume_value_mismatch_probe(out_dir: &Path, index: &str, snapshot: &serde_json::Value) {
    let mut mismatch = snapshot.clone();
    let value = mismatch["boundaries"]
        .as_array_mut()
        .expect("boundaries")
        .iter_mut()
        .flat_map(|boundary| {
            boundary["values"]
                .as_array_mut()
                .expect("values")
                .iter_mut()
        })
        .next()
        .expect("retained value");
    value["value"] = serde_json::Value::String("not-a-number".to_string());
    let page = resume_bootstrap_probe_page(
        index,
        &format!(
            "window.__PRESOLVE_RESUME_SNAPSHOT__ = {};",
            serde_json::to_string(&mismatch).expect("mismatch snapshot JSON")
        ),
        &fallback_assertions("ValueTypeMismatch"),
        "PRESOLVE_RESUME_VALUE_FALLBACK_PASS",
    );
    fs::write(out_dir.join("value-mismatch.html"), page).expect("value fallback probe");
}

fn resume_bootstrap_snapshot(manifest: &serde_json::Value) -> serde_json::Value {
    let slots = manifest["slot_schemas"].as_array().expect("slot schemas");
    let boundaries = manifest["boundaries"]
        .as_array()
        .expect("resume boundaries")
        .iter()
        .map(|boundary| {
            let boundary_id = boundary["boundary_id"].as_str().expect("boundary ID");
            let values = slots
                .iter()
                .filter(|slot| {
                    slot["owner_boundary_id"] == boundary_id && slot["restore_phase"] != "R5"
                })
                .map(|slot| {
                    let slot_id = slot["slot_id"].as_str().expect("slot ID");
                    serde_json::json!({
                        "valueRecordId": format!("resume-value:{slot_id}"),
                        "slotId": slot_id,
                        "value": resume_bootstrap_value(&slot["value_codec"])
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "boundaryId": boundary["boundary_id"],
                "schemaId": boundary["schema_id"],
                "values": values
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schemaVersion": 1,
        "buildId": manifest["build_id"],
        "snapshotId": format!("resume-snapshot:{}", manifest["build_id"].as_str().expect("build ID")),
        "manifestVersion": 6,
        "capturedAt": null,
        "boundaries": boundaries
    })
}

fn resume_bootstrap_value(codec: &serde_json::Value) -> serde_json::Value {
    match codec["kind"].as_str().expect("codec kind") {
        "null_codec" | "nullable_codec" => serde_json::Value::Null,
        "boolean_codec" => serde_json::Value::Bool(false),
        "number_codec" => serde_json::Value::from(7),
        "string_codec" => serde_json::Value::String(String::new()),
        "array_codec" => serde_json::Value::Array(Vec::new()),
        "object_codec" => serde_json::Value::Object(
            codec["value"]
                .as_array()
                .expect("object properties")
                .iter()
                .map(|property| {
                    (
                        property["name"]
                            .as_str()
                            .expect("property name")
                            .to_string(),
                        resume_bootstrap_value(&property["codec"]),
                    )
                })
                .collect(),
        ),
        kind => panic!("unsupported resume codec {kind}"),
    }
}

fn fallback_assertions(failure: &str) -> String {
    format!(
        r#"
if (runtime.resume.mode !== "cold" || runtime.resume.failure !== "{failure}") fail("resume failure did not select one cold boot");
if (!(runtime.store?.components instanceof Map) || runtime.components[0].state.count !== 1) fail("cold fallback did not initialize exactly once");
if (runtime.resume_registry !== null) fail("failed resume retained a partial registry");
const evidence = window.__PRESOLVE_RESUME__.debugEvidence()[0];
if (evidence.failure !== "{failure}" || (evidence.boundary_ids ?? []).length !== 0) fail("fallback debug evidence retained partial state");"#
    )
}

fn resume_bootstrap_probe_page(
    index: &str,
    prelude: &str,
    assertions: &str,
    marker: &str,
) -> String {
    let runtime_tag = r#"<script src="./runtime.js" defer></script>"#;
    let page = index.replace(
        runtime_tag,
        &format!("<script>{prelude}</script>\n{runtime_tag}"),
    );
    let split = marker.len() / 2;
    let marker_start = &marker[..split];
    let marker_end = &marker[split..];
    let probe = format!(
        r#"<script>
const fail = (message) => {{ throw new Error(message); }};
const waitFor = async (predicate, label) => {{
  for (let attempt = 0; attempt < 1000; attempt += 1) {{
    if (predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 10));
  }}
  fail(`Timed out waiting for ${{label}}`);
}};
(async () => {{
  await waitFor(() => window.__PRESOLVE__ !== undefined, "runtime");
  const runtime = window.__PRESOLVE__;
  {assertions}
  document.body.insertAdjacentHTML("beforeend", "<div>" + "{marker_start}" + "{marker_end}" + "</div>");
}})().catch((error) => {{
  document.body.insertAdjacentHTML("beforeend", `<pre>PRESOLVE_RESUME_BOOTSTRAP_FAIL: ${{error.message}}</pre>`);
}});
</script>
</body>"#
    );
    page.replace("</body>", &probe)
}

#[test]
fn component_structural_programs_preserve_host_dom_identity_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/component-structural-runtime");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0065-component-runtime/input/StructuralComponents.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build structural component fixture");
    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_component_structural_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);
    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_COMPONENT_STRUCTURAL_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[allow(clippy::too_many_lines)]
fn write_component_structural_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (document.documentElement.dataset.presolveRuntime === "error") {
      const diagnostic = window.__PRESOLVE__?.diagnostics?.map(({ code, message }) => `${code}: ${message}`).join(" | ") ?? "runtime error";
      reject(new Error(`Runtime failed before ${label}: ${diagnostic}`));
      return;
    }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const runtime = window.__PRESOLVE__;
  const artifact = JSON.parse(document.getElementById("presolve-component-runtime").textContent);
  if (artifact.schema_version !== 2 || artifact.structural_programs.length !== 2) fail("structural component programs were missing");
  if (runtime.store.componentRegions.size !== artifact.structural_programs.length) fail("closed structural region table diverged from the artifact");
  if (!artifact.structural_programs.every((program) => JSON.stringify(runtime.store.componentRegions.get(program.region)) === JSON.stringify(program))) {
    fail("runtime structural regions were not keyed by compiler IDs");
  }
  const serialized = JSON.stringify(artifact).toLowerCase();
  if (serialized.includes("virtual_dom") || serialized.includes("vdom")) fail("component artifact introduced virtual DOM authority");

  const manifest = runtime.manifest.components.find((component) => component.name === "StructuralPage");
  const conditional = manifest.template.nodes.find((node) => node.kind === "conditional");
  const list = manifest.template.nodes.find((node) => node.kind === "list");
  if (conditional === undefined || list === undefined) fail("structural host plans were missing");
  const buttons = [...document.querySelectorAll("main button")];
  if (buttons.length !== 3) fail("structural controls were missing");
  const main = document.querySelector("main");
  const listParent = document.querySelector("main ul");
  const row = (key) => document.querySelector(`[data-presolve-node='${list.item_root}:${key}']`);
  const a = row("a");
  const b = row("b");
  const c = row("c");
  if (main === null || listParent === null || a === null || b === null || c === null) fail("initial structural DOM was incomplete");

  buttons[0].click();
  await waitFor(() => document.querySelector("main aside") !== null, "conditional subtree swap");
  if (document.querySelector("main div") !== null) fail("outgoing conditional subtree survived");
  if (document.querySelector("main") !== main || document.querySelector("main ul") !== listParent) fail("unaffected sibling identity changed");
  if (row("a") !== a || row("b") !== b || row("c") !== c) fail("conditional swap recreated unaffected keyed rows");

  buttons[1].click();
  await waitFor(() => row("d") !== null && row("b") === null, "keyed component reconciliation");
  const d = row("d");
  if (row("a") !== a || row("c") !== c) fail("moved keyed component rows were recreated");
  if (d === null) fail("new keyed component row was not created");

  buttons[2].click();
  await waitFor(() => row("a") === null && row("c") === null, "nested keyed destruction");
  if (row("d") !== d) fail("retained keyed component row was recreated");
  if (document.querySelector("main") !== main || document.querySelector("main ul") !== listParent) fail("unaffected host identity changed after list destruction");
  if (runtime.component_failures.length !== 0) fail("structural component runtime reported failures");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_COMPONENT_STRUCTURAL_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_COMPONENT_STRUCTURAL_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_state_instance_storage_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (document.documentElement.dataset.presolveRuntime === "error") {
      reject(new Error(`Runtime failed before ${label}`));
      return;
    }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const runtime = window.__PRESOLVE__;
  const artifact = JSON.parse(document.getElementById("presolve-component-runtime").textContent);
  const instances = artifact.instances.filter((instance) => instance.component.endsWith("/component:x-repeated-counter"));
  if (instances.length !== 2) fail("repeated component instances were not planned exactly");
  const firstState = instances[0].state_slots[0];
  const secondState = instances[1].state_slots[0];
  const firstComputed = instances[0].computed_slots[0];
  const secondComputed = instances[1].computed_slots[0];
  if (firstState.storage_id !== secondState.storage_id || firstState.slot_id === secondState.slot_id) {
    fail("one declaration storage did not receive two exact instance slots");
  }
  if (Number(runtime.store.storageValues.get(firstState.slot_id)) !== 1
    || Number(runtime.store.storageValues.get(secondState.slot_id)) !== 1) {
    fail("cold boot did not initialize both State instance slots independently");
  }
  if (runtime.store.computedCaches.get(firstComputed.cache_slot_id) !== 2
    || runtime.store.computedCaches.get(secondComputed.cache_slot_id) !== 2) {
    fail("cold boot did not evaluate both computed instance caches");
  }
  const buttons = [...document.querySelectorAll("button")];
  if (buttons.length !== 2 || buttons[0].textContent !== "1" || buttons[1].textContent !== "1") {
    fail("repeated State bindings did not initialize independently");
  }
  buttons[0].click();
  await waitFor(() => buttons[0].textContent === "2", "first instance update");
  if (buttons[1].textContent !== "1") fail("first action updated the second instance binding");
  if (Number(runtime.store.storageValues.get(firstState.slot_id)) !== 2
    || Number(runtime.store.storageValues.get(secondState.slot_id)) !== 1) {
    fail("first action crossed the State instance storage boundary");
  }
  if (runtime.store.computedCaches.get(firstComputed.cache_slot_id) !== 4
    || runtime.store.computedCaches.get(secondComputed.cache_slot_id) !== 2) {
    fail("computed invalidation crossed the component instance boundary");
  }
  if (runtime.store.computedDirtySlots.get(firstComputed.dirty_slot_id) !== false
    || runtime.store.computedDirtySlots.get(secondComputed.dirty_slot_id) !== false) {
    fail("computed dirty flags did not settle independently");
  }
  const exactStateSlotIds = new Set([firstState.slot_id, secondState.slot_id]);
  if ([...runtime.store.storageValues.keys()].some((key) => !exactStateSlotIds.has(key))) {
    fail("schema-v3 runtime exposed a declaration-level State storage key");
  }
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_STATE_INSTANCE_STORAGE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_STATE_INSTANCE_STORAGE_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );
    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn replace_json_script(
    page: &str,
    element_id: &str,
    transform: impl FnOnce(&mut serde_json::Value),
) -> String {
    let id = format!("id=\"{element_id}\"");
    let id_start = page.find(&id).expect("JSON script id was missing");
    let content_start = page[id_start..]
        .find('>')
        .map(|offset| id_start + offset + 1)
        .expect("JSON script opening tag was malformed");
    let content_end = page[content_start..]
        .find("</script>")
        .map(|offset| content_start + offset)
        .expect("JSON script closing tag was missing");
    let mut value: serde_json::Value =
        serde_json::from_str(&page[content_start..content_end]).expect("JSON script was invalid");
    transform(&mut value);
    let json = serde_json::to_string_pretty(&value).expect("JSON script should serialize");
    format!(
        "{}{}\n{}",
        &page[..content_start],
        json,
        &page[content_end..]
    )
}

fn downgrade_component_artifact_to_v2(value: &mut serde_json::Value) {
    value["schema_version"] = serde_json::json!(2);
    for instance in value["instances"]
        .as_array_mut()
        .expect("component instances should be an array")
    {
        instance
            .as_object_mut()
            .expect("component instance should be an object")
            .remove("state_slots");
    }
}

fn write_state_instance_compatibility_probe_pages(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");

    let rejected = replace_json_script(
        &index,
        "presolve-component-runtime",
        downgrade_component_artifact_to_v2,
    );
    let rejected = rejected.replace(
        "</body>",
        r#"<script>
const wait = setInterval(() => {
  if (document.documentElement.dataset.presolveRuntime !== "error") return;
  clearInterval(wait);
  const codes = window.__PRESOLVE__?.diagnostics?.map((diagnostic) => diagnostic.code) ?? [];
  if (codes.includes("PSR_UNSUPPORTED_SCHEMA")) {
    document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_PHASE_J_COMPONENT_V2_REJECTION_PASS</div>");
  }
}, 20);
</script>
</body>"#,
    );
    fs::write(out_dir.join("phase-j-component-v2.html"), rejected)
        .expect("failed to write Phase J v2 rejection probe");

    let legacy = replace_json_script(&index, "presolve-template-manifest", |value| {
        value["schema_version"] = serde_json::json!(3);
        value["ordinary_targets"] = serde_json::json!([]);
        value["ordinary_bindings"] = serde_json::json!([]);
        value["ordinary_events"] = serde_json::json!([]);
    });
    let legacy = replace_json_script(
        &legacy,
        "presolve-component-runtime",
        downgrade_component_artifact_to_v2,
    );
    let legacy = legacy.replace(
        "</body>",
        r#"<script>
const wait = setInterval(() => {
  if (document.documentElement.dataset.presolveRuntime !== "ready") return;
  clearInterval(wait);
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_LEGACY_COMPONENT_V2_COLD_PASS</div>");
}, 20);
</script>
</body>"#,
    );
    fs::write(out_dir.join("legacy-component-v2-cold.html"), legacy)
        .expect("failed to write legacy v2 cold probe");
}

fn write_component_runtime_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const runtime = window.__PRESOLVE__;
  const artifact = JSON.parse(document.getElementById("presolve-component-runtime").textContent);
  for (const record of document.querySelectorAll('script[type="application/json"]')) record.remove();
  if (artifact.schema_version !== 3) fail("component artifact schema was not v3");
  if (artifact.instances.length < 6) fail("component instances were not materialized from the plan");
  if (runtime.store.componentInstances.size !== artifact.instances.length) fail("instance plan was not loaded exactly once");
  if (runtime.component_instance_tree.length !== artifact.instances.length) fail("debug instance tree diverged from the compiler artifact");
  if (runtime.component_initialization_runs.join(",") !== artifact.initialization_batches.map((batch) => batch.index).join(",")) {
    fail("initialization batch order diverged from the compiler artifact");
  }
  for (const instance of artifact.instances) {
    if (instance.parent !== null) {
      const parent = artifact.instances.find((candidate) => candidate.instance === instance.parent);
      if (parent === undefined || parent.depth >= instance.depth || parent.initialization_batch >= instance.initialization_batch) {
        fail("parent-before-child order was not preserved");
      }
    }
  }
  const stateSlots = artifact.instances.flatMap((instance) => instance.state_slots);
  if (new Set(stateSlots.map((slot) => slot.slot_id)).size !== stateSlots.length) {
    fail("repeated instances shared State slot identity");
  }
  if (stateSlots.some((slot) => !runtime.store.storageValues.has(slot.slot_id))) {
    fail("cold boot did not initialize every exact State instance slot");
  }
  if ([...runtime.store.storageValues.keys()].some((key) => !stateSlots.some((slot) => slot.slot_id === key))) {
    fail("schema-v3 runtime retained a declaration-level State storage key");
  }
  if (runtime.slot_binding_runs.length !== artifact.slot_binding_programs.length) fail("slot programs were not loaded exactly");
  if (runtime.store.instanceContextBindings.size !== artifact.instance_context_bindings.length) fail("instance Context plans were not loaded exactly");
  if (!artifact.slot_binding_programs.every((binding) => binding.content_owner_instance === binding.caller_instance)) {
    fail("slotted content lost caller ownership");
  }
  const serialized = Object.keys(artifact).join(",").toLowerCase();
  for (const forbidden of ["lookup", "resolve", "traverse", "ancestry", "virtual_dom", "vdom"]) {
    if (serialized.includes(forbidden)) fail(`component artifact contained forbidden runtime authority: ${forbidden}`);
  }
  if (runtime.component_failures.length !== 0) fail("component runtime reported failures");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_COMPONENT_RUNTIME_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_COMPONENT_RUNTIME_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_form_submission_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace("</body>", r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => { const deadline = Date.now() + 3000; const tick = () => predicate() ? resolve() : Date.now() > deadline ? reject(new Error(`Timed out waiting for ${label}`)) : setTimeout(tick, 20); tick(); });
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const forms = JSON.parse(document.getElementById("presolve-forms-runtime").textContent);
  const manifest = JSON.parse(document.getElementById("presolve-template-manifest").textContent);
  if (forms.hosts.length !== 1 || manifest.form_hosts.length !== 1) fail("exact host records were absent");
  const host = document.querySelector("form");
  const event = new Event("submit", { bubbles: true, cancelable: true });
  host.dispatchEvent(event);
  await waitFor(() => window.__PRESOLVE__.components[0].state.submitted === 1, "submit action");
  if (!event.defaultPrevented) fail("compiler host policy did not prevent default");
  if (window.__PRESOLVE__.forms[0].submission !== "Completed") fail("submission did not complete");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("runtime reported Forms diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_FORM_SUBMISSION_BROWSER_TEST_PASS</div>");
})().catch((error) => { document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_FORM_SUBMISSION_BROWSER_TEST_FAIL: ${error.message}</div>`); console.error(error); });
</script>
</body>"#);
    fs::write(out_dir.join("probe.html"), probe).expect("failed to write Forms probe page");
}

#[test]
fn double_binding_counter_increments_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/double-binding-counter");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn framework_counter_increments_through_compiler_artifacts_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/framework-counter");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous framework Counter output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "examples/counter/src/Counter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("framework Counter output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build framework Counter through the compiler");
    assert!(
        output.status.success(),
        "expected framework Counter build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_framework_counter_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);
    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_FRAMEWORK_COUNTER_BROWSER_TEST_PASS"),
        "framework Counter browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn decrement_counter_decrements_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/decrement-counter");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_decrement_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_DECREMENT_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn add_subtract_assign_counter_updates_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/add-subtract-assign");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_add_subtract_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_ADD_SUBTRACT_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn direct_assignment_counter_resets_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/direct-assignment");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_direct_assignment_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_DIRECT_ASSIGN_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn boolean_toggle_flips_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/boolean-toggle");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0012-boolean-toggle/input/ToggleFlag.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_boolean_toggle_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_BOOLEAN_TOGGLE_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn multi_step_action_runs_all_steps_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/multi-step-action");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_multi_step_action_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_MULTI_STEP_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dynamic_attributes_update_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/dynamic-attributes");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_dynamic_attributes_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_DYNAMIC_ATTRIBUTES_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn fragments_preserve_sibling_identity_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/fragments");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0016-fragments/input/FragmentPanel.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_fragments_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_FRAGMENTS_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn conditional_branches_switch_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/conditional-rendering");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_conditional_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_CONDITIONAL_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn logical_and_conditional_switches_to_empty_branch_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/logical-and-conditional");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_logical_and_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_LOGICAL_AND_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn keyed_lists_reconcile_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/keyed-list-reconciliation");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0021-keyed-list-reconciliation/input/KeyedListReconciliation.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_keyed_list_reconciliation_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_KEYED_LIST_BROWSER_TEST_PASS"),
        "browser probe did not pass\\nstatus: {}\\nstdout:\\n{}\\nstderr:\\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn object_keyed_lists_reconcile_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/object-keyed-list-reconciliation");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0025-object-keyed-list-reconciliation/input/ObjectKeyedListReconciliation.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_object_keyed_list_reconciliation_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_OBJECT_KEYED_LIST_BROWSER_TEST_PASS"),
        "browser probe did not pass\\nstatus: {}\\nstdout:\\n{}\\nstderr:\\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dynamic_list_items_refresh_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/dynamic-list-item-behavior");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0026-dynamic-list-item-behavior/input/DynamicListItemBehavior.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\\nstatus: {}\\nstderr:\\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_dynamic_list_item_behavior_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_DYNAMIC_LIST_ITEM_BROWSER_TEST_PASS"),
        "browser probe did not pass\\nstatus: {}\\nstdout:\\n{}\\nstderr:\\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn runtime_contract_diagnostics_report_manifest_boot_failures_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/runtime-contract");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0004-nested-jsx/input/NestedCounter.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let probes = [
        (
            "missing-manifest.html",
            None,
            "PSR_MISSING_MANIFEST",
            "PRESOLVE_MISSING_MANIFEST_DIAGNOSTIC_PASS",
        ),
        (
            "invalid-json.html",
            Some("{"),
            "PSR_INVALID_MANIFEST_JSON",
            "PRESOLVE_INVALID_JSON_DIAGNOSTIC_PASS",
        ),
        (
            "unsupported-schema.html",
            Some(r#"{"schema_version":999,"components":[]}"#),
            "PSR_UNSUPPORTED_SCHEMA",
            "PRESOLVE_UNSUPPORTED_SCHEMA_DIAGNOSTIC_PASS",
        ),
    ];

    for (index, (file_name, manifest_json, expected_code, pass_marker)) in probes.iter().enumerate()
    {
        write_runtime_contract_probe_page(
            &out_dir,
            file_name,
            *manifest_json,
            expected_code,
            pass_marker,
        );

        let server = StaticServer::start(out_dir.clone());
        let chrome = chrome_bin().expect("headless Chrome was not found");
        let profile_dir = out_dir.join(format!("chrome-profile-{index}"));
        fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
        let user_data_dir = format!(
            "--user-data-dir={}",
            profile_dir
                .to_str()
                .expect("Chrome profile path was not valid UTF-8")
        );
        let probe_url = format!("http://127.0.0.1:{}/{}", server.port, file_name);

        let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

        let stdout = String::from_utf8_lossy(&output.stdout);

        server.stop();

        assert!(
            stdout.contains(pass_marker),
            "browser probe did not pass for {file_name}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn string_state_initializes_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/string-greeting");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0006-string-state/input/StringGreeting.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_string_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_STRING_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn boolean_state_initializes_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/boolean-flags");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0007-boolean-state/input/BooleanFlags.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_boolean_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_BOOLEAN_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn null_state_initializes_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/null-selection");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0008-null-state/input/NullSelection.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to run presolve_cli build");

    assert!(
        output.status.success(),
        "expected build to succeed\nstatus: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    write_null_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);

    let stdout = String::from_utf8_lossy(&output.stdout);

    server.stop();

    assert!(
        stdout.contains("PRESOLVE_NULL_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn computed_values_execute_once_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/computed-runtime-execution");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0044-computed-runtime-execution/input/RuntimeComputed.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build computed runtime fixture");
    assert!(output.status.success());

    write_computed_runtime_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_COMPUTED_RUNTIME_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pure_package_contracts_execute_only_the_compiler_lowered_operation_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/pure-package-runtime");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("clean previous pure package browser output");
    }
    fs::create_dir_all(&out_dir).expect("create pure package browser output");
    let source = out_dir.join("PackageComputed.tsx");
    let contract = out_dir.join("value-kit.contract.json");
    fs::write(
        &source,
        r#"
import { identity as visible } from "value-kit";
@component("x-package-computed")
class PackageComputed extends Component {
  count = state(1);
  @computed()
  get visibleCount() { return visible(this.count); }
  render() { return <output>Visible count</output>; }
}

"#,
    )
    .expect("write pure package source");
    fs::write(
        &contract,
        r#"{"schema_version":1,"package":"value-kit","version":"1.0.0","integrity":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","exports":{"identity":{"kind":"pure","type_signature":"<T>(T)->T","runtime_module":"dist/identity.js","resume_policy":"input_only","pure_operation":"identity"}}}"#,
    )
    .expect("write pure package contract");
    let contract_argument = format!("value-kit={}", contract.display());
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            source.to_str().expect("source path"),
            "--package-contract",
            &contract_argument,
            "--out",
            out_dir.to_str().expect("output path"),
        ])
        .output()
        .expect("build pure package runtime fixture");
    assert!(
        output.status.success(),
        "pure package build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    write_pure_package_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("create Chrome profile directory");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir.to_str().expect("Chrome profile path")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);
    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_PURE_PACKAGE_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn template_interpolations_execute_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/template-interpolation-runtime");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("clean previous template browser output");
    }
    fs::create_dir_all(&out_dir).expect("create template browser output");
    let source = out_dir.join("TemplateComputed.tsx");
    fs::write(
        &source,
        r#"
@component("x-template-computed")
class TemplateComputed extends Component {
  count = state(2);
  @computed()
  get label() { return `Count: ${this.count}!`; }
  render() { return <output>Count</output>; }
}
"#,
    )
    .expect("write template source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            source.to_str().expect("source path"),
            "--out",
            out_dir.to_str().expect("output path"),
        ])
        .output()
        .expect("build template runtime fixture");
    assert!(output.status.success());

    write_template_interpolation_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("create Chrome profile directory");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir.to_str().expect("Chrome profile path")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);
    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_TEMPLATE_INTERPOLATION_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn static_index_accesses_execute_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/static-index-runtime");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("clean previous index browser output");
    }
    fs::create_dir_all(&out_dir).expect("create index browser output");
    let source = out_dir.join("IndexedComputed.tsx");
    fs::write(
        &source,
        r#"
@component("x-indexed-computed")
class IndexedComputed extends Component {
  labels = state(["zero", "one"]);
  @computed()
  get selected() { return this.labels[1]; }
  render() { return <output>Indexed</output>; }
}

"#,
    )
    .expect("write index source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            source.to_str().expect("source path"),
            "--out",
            out_dir.to_str().expect("output path"),
        ])
        .output()
        .expect("build index runtime fixture");
    assert!(
        output.status.success(),
        "index build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    write_static_index_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("create Chrome profile directory");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir.to_str().expect("Chrome profile path")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);
    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_STATIC_INDEX_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn boolean_conditional_computed_values_execute_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/conditional-computed-runtime");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("clean conditional browser output");
    }
    fs::create_dir_all(&out_dir).expect("create conditional browser output");
    let source = out_dir.join("ConditionalComputed.tsx");
    fs::write(
        &source,
        r#"
@component("x-conditional-computed")
class ConditionalComputed extends Component {
  enabled: boolean = state(true);
  @computed()
  get label() { return this.enabled ? "enabled" : "disabled"; }
  render() { return <output>Conditional</output>; }
}
"#,
    )
    .expect("write conditional source");
    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            source.to_str().expect("source path"),
            "--out",
            out_dir.to_str().expect("output path"),
        ])
        .output()
        .expect("build conditional runtime fixture");
    assert!(
        output.status.success(),
        "conditional build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    write_boolean_conditional_probe_page(&out_dir);
    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("create Chrome profile directory");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile_dir.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_BOOLEAN_CONDITIONAL_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn optional_member_accesses_execute_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let root = repo_root();
    let out = root.join("target/psc-browser-test/optional-member-runtime");
    if out.exists() {
        fs::remove_dir_all(&out).expect("clean optional output");
    }
    fs::create_dir_all(&out).expect("create optional output");
    let source = out.join("OptionalComputed.tsx");
    fs::write(
        &source,
        r#"
@component("x-optional-computed")
class OptionalComputed extends Component {
  profile = state({ label: "Presolve" });
  @computed()
  get label() { return this.profile?.label; }
  render() { return <output>Optional</output>; }
}
"#,
    )
    .expect("write optional source");
    let build = Command::new(presolve_cli_bin())
        .current_dir(&root)
        .args([
            "build",
            source.to_str().expect("source"),
            "--out",
            out.to_str().expect("out"),
        ])
        .output()
        .expect("build optional fixture");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    write_optional_member_probe_page(&out);
    let server = StaticServer::start(out.clone());
    let chrome = chrome_bin().expect("Chrome");
    let profile = out.join("chrome-profile");
    fs::create_dir_all(&profile).expect("profile");
    let output = run_chrome_probe(
        chrome,
        &format!("--user-data-dir={}", profile.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_OPTIONAL_MEMBER_BROWSER_TEST_PASS"),
        "{}",
        stdout
    );
}

#[test]
fn registered_math_abs_executes_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let root = repo_root();
    let out = root.join("target/psc-browser-test/math-abs-runtime");
    if out.exists() {
        fs::remove_dir_all(&out).expect("clean abs output");
    }
    fs::create_dir_all(&out).expect("create abs output");
    let source = out.join("AbsComputed.tsx");
    fs::write(
        &source,
        r#"
@component("x-abs-computed")
class AbsComputed extends Component {
  count = state(-2);
  @computed()
  get magnitude() { return Math.abs(this.count); }
  render() { return <output>Abs</output>; }
}

"#,
    )
    .expect("write abs source");
    let build = Command::new(presolve_cli_bin())
        .current_dir(&root)
        .args([
            "build",
            source.to_str().expect("source"),
            "--out",
            out.to_str().expect("out"),
        ])
        .output()
        .expect("build abs");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    write_math_abs_probe_page(&out);
    let server = StaticServer::start(out.clone());
    let profile = out.join("chrome-profile");
    fs::create_dir_all(&profile).expect("profile");
    let output = run_chrome_probe(
        chrome_bin().expect("Chrome"),
        &format!("--user-data-dir={}", profile.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_MATH_ABS_BROWSER_TEST_PASS"),
        "{}",
        stdout
    );
}

#[test]
fn registered_math_min_max_execute_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let root = repo_root();
    let out = root.join("target/psc-browser-test/math-min-max-runtime");
    if out.exists() {
        fs::remove_dir_all(&out).expect("clean min max output");
    }
    fs::create_dir_all(&out).expect("create min max output");
    let source = out.join("MinMaxComputed.tsx");
    fs::write(
        &source,
        r#"
@component("x-min-max-computed")
class MinMaxComputed extends Component {
  left = state(-2); right = state(5);
  @computed() get minimum() { return Math.min(this.left, this.right); }
  @computed() get maximum() { return Math.max(this.left, this.right); }
  render() { return <output>MinMax</output>; }
}
"#,
    )
    .expect("write min max source");
    let build = Command::new(presolve_cli_bin())
        .current_dir(&root)
        .args([
            "build",
            source.to_str().expect("source"),
            "--out",
            out.to_str().expect("out"),
        ])
        .output()
        .expect("build min max");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    write_math_min_max_probe_page(&out);
    let server = StaticServer::start(out.clone());
    let profile = out.join("chrome-profile");
    fs::create_dir_all(&profile).expect("profile");
    let output = run_chrome_probe(
        chrome_bin().expect("Chrome"),
        &format!("--user-data-dir={}", profile.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_MATH_MIN_MAX_BROWSER_TEST_PASS"),
        "{}",
        stdout
    );
}

#[test]
fn serializable_record_state_replacement_executes_from_compiler_generated_runtime() {
    let _guard = browser_test_guard();
    let root = repo_root();
    let out = root.join("target/psc-browser-test/record-state-replacement");
    if out.exists() {
        fs::remove_dir_all(&out).expect("clean record output");
    }
    fs::create_dir_all(&out).expect("create record output");
    let source = out.join("RecordState.tsx");
    fs::write(
        &source,
        r#"
@component("x-record-state")
class RecordState extends Component {
  profile = state({ name: "before", roles: ["reader"] });
  @action() replace() { this.profile = { name: "after", roles: ["writer", "admin"] }; }
  render() { return <button onClick={this.replace}>Replace</button>; }
}
"#,
    )
    .expect("write record source");
    let build = Command::new(presolve_cli_bin())
        .current_dir(&root)
        .args([
            "build",
            source.to_str().expect("source"),
            "--out",
            out.to_str().expect("out"),
        ])
        .output()
        .expect("build record");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let index = fs::read_to_string(out.join("index.html")).expect("read record page");
    let probe = index.replace("</body>", r#"<script>(async()=>{const end=Date.now()+3000;while(document.documentElement.dataset.presolveRuntime!=="ready"){if(Date.now()>end)throw new Error("runtime");await new Promise(r=>setTimeout(r,20));}const action=window.__PRESOLVE__.manifest.components[0].actions[0];if(action.operation!=="assign"||action.operand?.name!=="after")throw new Error("record operand");document.querySelector("button").click();await new Promise(r=>setTimeout(r,20));const state=window.__PRESOLVE__.components[0].state.profile;if(state?.name!=="after"||state.roles?.length!==2)throw new Error("record state");document.body.insertAdjacentHTML("beforeend","<div>PRESOLVE_RECORD_STATE_BROWSER_TEST_PASS</div>");})().catch(e=>document.body.insertAdjacentHTML("beforeend",`<div>PRESOLVE_RECORD_STATE_BROWSER_TEST_FAIL:${e.message}</div>`));</script></body>"#);
    fs::write(out.join("probe.html"), probe).expect("write record probe");
    let server = StaticServer::start(out.clone());
    let profile = out.join("chrome-profile");
    fs::create_dir_all(&profile).expect("profile");
    let output = run_chrome_probe(
        chrome_bin().expect("Chrome"),
        &format!("--user-data-dir={}", profile.display()),
        &format!("http://127.0.0.1:{}/probe.html", server.port),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();
    assert!(
        stdout.contains("PRESOLVE_RECORD_STATE_BROWSER_TEST_PASS"),
        "{}",
        stdout
    );
}

#[test]
fn initial_effects_execute_once_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/initial-effect-runtime");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0053-effect-initial-runtime/input/InitialEffectRuntime.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build initial effect runtime fixture");
    assert!(output.status.success());

    write_initial_effect_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_INITIAL_EFFECT_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn completed_action_batches_execute_compiler_planned_effects_once() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/completed-action-effect-runtime");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0053-effect-initial-runtime/input/InitialEffectRuntime.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build completed action effect fixture");
    assert!(output.status.success());

    write_completed_action_effect_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_COMPLETED_ACTION_EFFECT_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn context_sources_bind_and_update_from_compiler_plans_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/context-runtime-matrix");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0059-context-runtime-matrix/input/ContextRuntimeMatrix.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build Context runtime fixture");
    assert!(
        output.status.success(),
        "Context runtime fixture failed to build:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    write_context_runtime_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_CONTEXT_RUNTIME_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn context_source_failure_preserves_compiler_binding_without_reselection_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/context-source-failure");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0059-context-runtime-matrix/input/ContextRuntimeMatrix.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build Context failure fixture");
    assert!(
        output.status.success(),
        "Context failure fixture failed to build:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    write_context_source_failure_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_CONTEXT_FAILURE_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn multi_step_actions_flush_one_compiler_generated_computed_batch() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/computed-batched-invalidation");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0045-computed-batched-invalidation/input/BatchedComputed.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build computed batching fixture");
    assert!(output.status.success());

    write_batched_computed_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_COMPUTED_BATCH_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn diamond_computed_values_recompute_from_compiler_generated_batches() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/psc-browser-test/computed-diamond");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(presolve_cli_bin())
        .current_dir(&repo_root)
        .args([
            "build",
            "fixtures/0047-computed-diamond/input/ComputedDiamond.tsx",
            "--out",
            out_dir
                .to_str()
                .expect("browser test output path was not valid UTF-8"),
        ])
        .output()
        .expect("failed to build computed diamond fixture");
    assert!(output.status.success());

    write_diamond_computed_probe_page(&out_dir);

    let server = StaticServer::start(out_dir.clone());
    let chrome = chrome_bin().expect("headless Chrome was not found");
    let profile_dir = out_dir.join("chrome-profile");
    fs::create_dir_all(&profile_dir).expect("failed to create Chrome profile dir");
    let user_data_dir = format!(
        "--user-data-dir={}",
        profile_dir
            .to_str()
            .expect("Chrome profile path was not valid UTF-8")
    );
    let probe_url = format!("http://127.0.0.1:{}/probe.html", server.port);

    let output = run_chrome_probe(chrome, &user_data_dir, &probe_url);
    let stdout = String::from_utf8_lossy(&output.stdout);
    server.stop();

    assert!(
        stdout.contains("PRESOLVE_COMPUTED_DIAMOND_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_computed_runtime_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const computed = window.__PRESOLVE__.computed;
  if (!Array.isArray(computed) || computed.length !== 2) fail("computed cache records were missing");
  const doubled = computed.find((entry) => entry.computed.endsWith("/computed:doubled"));
  const label = computed.find((entry) => entry.computed.endsWith("/computed:label"));
  if (doubled?.value !== 2 || label?.value !== 3) fail("computed programs did not evaluate in compiler order");
  if (doubled?.dirty !== false || label?.dirty !== false) fail("computed caches remained dirty after execution");
  if (!(window.__PRESOLVE__.store.computedCaches instanceof Map)) fail("computed cache store was not a Map");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("computed execution reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_COMPUTED_RUNTIME_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_COMPUTED_RUNTIME_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_pure_package_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const computed = window.__PRESOLVE__.computed;
  const visible = computed.find((entry) => entry.computed.endsWith("/computed:visibleCount"));
  if (visible?.value !== 1 || visible?.dirty !== false) fail("pure package operation did not execute from compiler artifact");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("pure package execution reported diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_PURE_PACKAGE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_PURE_PACKAGE_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );
    fs::write(out_dir.join("probe.html"), probe).expect("write pure package probe page");
}

fn write_template_interpolation_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const computed = window.__PRESOLVE__.computed;
  const label = computed.find((entry) => entry.computed.endsWith("/computed:label"));
  if (label?.value !== "Count: 2!" || label?.dirty !== false) fail("template interpolation did not execute from compiler artifact");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("template interpolation reported diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_TEMPLATE_INTERPOLATION_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_TEMPLATE_INTERPOLATION_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );
    fs::write(out_dir.join("probe.html"), probe).expect("write template probe page");
}

fn write_static_index_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const computed = window.__PRESOLVE__.computed;
  const selected = computed.find((entry) => entry.computed.endsWith("/computed:selected"));
  if (selected?.value !== "one" || selected?.dirty !== false) fail("static index access did not execute from compiler artifact");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("static index access reported diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_STATIC_INDEX_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_STATIC_INDEX_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );
    fs::write(out_dir.join("probe.html"), probe).expect("write static index probe page");
}

fn write_boolean_conditional_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read built page");
    let probe = index.replace("</body>", r#"<script>
(async () => {
  const deadline = Date.now() + 3000;
  while (document.documentElement.dataset.presolveRuntime !== "ready") {
    if (Date.now() > deadline) throw new Error("timed out waiting for runtime");
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  const label = window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:label"));
  if (label?.value !== "enabled" || label?.dirty !== false) throw new Error("conditional did not select true branch");
  if (window.__PRESOLVE__.diagnostics.length !== 0) throw new Error("conditional reported diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_BOOLEAN_CONDITIONAL_BROWSER_TEST_PASS</div>");
})().catch((error) => { document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_BOOLEAN_CONDITIONAL_BROWSER_TEST_FAIL: ${error.message}</div>`); });
</script></body>"#);
    fs::write(out_dir.join("probe.html"), probe).expect("write conditional probe page");
}

fn write_optional_member_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read optional page");
    let probe = index.replace("</body>", r#"<script>
(async () => {
  const until = Date.now() + 3000;
  while (document.documentElement.dataset.presolveRuntime !== "ready") {
    if (Date.now() > until) throw new Error("runtime not ready");
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  const label = window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:label"));
  if (label?.value !== "Presolve" || label?.dirty !== false) throw new Error("optional member did not evaluate");
  if (window.__PRESOLVE__.diagnostics.length !== 0) throw new Error("optional member diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_OPTIONAL_MEMBER_BROWSER_TEST_PASS</div>");
})().catch((error) => { document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_OPTIONAL_MEMBER_BROWSER_TEST_FAIL: ${error.message}</div>`); });
</script></body>"#);
    fs::write(out_dir.join("probe.html"), probe).expect("write optional probe");
}

fn write_math_abs_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read abs page");
    let probe = index.replace("</body>", r#"<script>
(async () => { const end = Date.now() + 3000; while (document.documentElement.dataset.presolveRuntime !== "ready") { if (Date.now() > end) throw new Error("runtime not ready"); await new Promise((r) => setTimeout(r, 20)); }
const value = window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:magnitude"));
if (value?.value !== 2 || value?.dirty !== false || window.__PRESOLVE__.diagnostics.length !== 0) throw new Error("Math.abs did not execute");
document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_MATH_ABS_BROWSER_TEST_PASS</div>"); })().catch((error) => { document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_MATH_ABS_BROWSER_TEST_FAIL: ${error.message}</div>`); });
</script></body>"#);
    fs::write(out_dir.join("probe.html"), probe).expect("write abs probe");
}

fn write_math_min_max_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("read min max page");
    let probe = index.replace("</body>", r#"<script>
(async () => { const end = Date.now() + 3000; while (document.documentElement.dataset.presolveRuntime !== "ready") { if (Date.now() > end) throw new Error("runtime not ready"); await new Promise((r) => setTimeout(r, 20)); }
const minimum = window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:minimum"));
const maximum = window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:maximum"));
if (minimum?.value !== -2 || maximum?.value !== 5 || minimum?.dirty !== false || maximum?.dirty !== false || window.__PRESOLVE__.diagnostics.length !== 0) throw new Error("Math.min/Math.max did not execute");
document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_MATH_MIN_MAX_BROWSER_TEST_PASS</div>"); })().catch((error) => { document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_MATH_MIN_MAX_BROWSER_TEST_FAIL: ${error.message}</div>`); });
</script></body>"#);
    fs::write(out_dir.join("probe.html"), probe).expect("write min max probe");
}

fn write_initial_effect_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  if (document.title !== "Presolve initial effect") fail("effect did not update document title");
  if (localStorage.getItem("presolve-effect-initial") !== "ready") fail("effect did not update local storage");
  const runs = window.__PRESOLVE__.initial_effect_runs;
  if (!Array.isArray(runs) || runs.length !== 1) fail("initial effect did not execute exactly once");
  const run = runs[0];
  if (!run.effect.endsWith("/effect:report") || run.effect_batch_index !== 0) fail("initial effect debug evidence was not deterministic");
  const operations = run.capability_operations.map((operation) => operation.runtime_lowering).join("|");
  if (operations !== "builtin.browser.console.log|builtin.browser.document.title.assign|builtin.browser.local_storage.set_item") {
    fail("effect capability dispatch order did not match the compiler program");
  }
  if (window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:doubled"))?.value !== 4) {
    fail("effect did not observe initialized computed state");
  }
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("initial effect execution reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_INITIAL_EFFECT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_INITIAL_EFFECT_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_completed_action_effect_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const manifestEvent = window.__PRESOLVE__.manifest.components[0].template.events[0];
  if (manifestEvent.kind !== "action" || !manifestEvent.method_id || !manifestEvent.action_batch_id) {
    fail("template manifest did not carry canonical action activation identities");
  }
  document.querySelector("button")?.click();
  await waitFor(() => window.__PRESOLVE__.completed_action_effect_runs.length === 1, "completed action effect");
  if (document.title !== "Presolve after action") fail("completed action effect did not synchronize title");
  const run = window.__PRESOLVE__.completed_action_effect_runs[0];
  if (run.action_batch_id !== manifestEvent.action_batch_id || run.effect_batch_index !== 0) {
    fail("runtime did not consume the exact compiler action batch plan");
  }
  if (run.capability_operations.length !== 3) fail("completed action effect did not preserve capability program");
  if (window.__PRESOLVE__.initial_effect_runs.length !== 1) fail("initial effect plan was replayed after action");
  if (window.__PRESOLVE__.computed.find((entry) => entry.computed.endsWith("/computed:doubled"))?.value !== 6) {
    fail("completed action effect ran before compiler computed flush");
  }
  if (window.__PRESOLVE__.store.activeActionBatch !== null) fail("runtime retained an active action batch");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("completed action effect reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_COMPLETED_ACTION_EFFECT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_COMPLETED_ACTION_EFFECT_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_context_runtime_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const runtime = window.__PRESOLVE__;
  const artifactElement = document.getElementById("presolve-context-runtime");
  if (artifactElement === null) fail("compiler Context artifact was missing");
  const artifact = JSON.parse(artifactElement.textContent);
  const serializedArtifact = JSON.stringify(artifact);
  for (const forbidden of ["lookup", "resolve", "traverse", "ancestry", "reconstruct"]) {
    if (serializedArtifact.includes(forbidden)) fail(`Context artifact contained forbidden runtime operation: ${forbidden}`);
  }
  if (artifact.sources.length !== 1 || artifact.consumers.length !== 2) fail("Context artifact did not preserve the frozen source and Consumer plans");
  if (runtime.context_initial_source_runs.length !== 1) fail("initial Context source did not execute exactly once");
  if (runtime.context_slots.length !== 1 || runtime.context_slots[0][1] !== 2) fail("initial Context slot value was incorrect");
  if (runtime.context_consumer_bindings.length !== 2) fail("Consumer bindings were missing");
  const slot = runtime.context_consumer_bindings[0][1];
  if (!runtime.context_consumer_bindings.every((binding) => binding[1] === slot)) fail("Consumers did not bind directly to the shared compiler slot");
  if (runtime.context_failures.length !== 0) fail("initial Context execution reported failures");
  if (runtime.initial_effect_runs.length !== 1) fail("cold boot did not execute the effect after Context initialization");

  const buttons = document.querySelectorAll("button");
  buttons[0].click();
  await waitFor(
    () => runtime.store.contextUpdateSourceRuns.length === 1 && runtime.store.completedActionEffectRuns.length === 1,
    "Context update and completed effect"
  );
  if (runtime.store.contextSlots.get(slot) !== 6) fail("Context update did not observe the flushed computed value");
  if (runtime.store.computedUpdateRuns !== 1) fail("completed action did not execute one computed update batch");
  if (runtime.store.contextUpdateSourceRuns[0].action_batch !== artifact.action_updates[0].action_batch) {
    fail("runtime did not consume the exact compiler action-batch Context plan");
  }
  if (runtime.store.completedActionEffectRuns[0].action_batch_id !== artifact.action_updates[0].action_batch) {
    fail("completed effect did not run after the same compiler action batch");
  }
  if (!runtime.context_consumer_bindings.every((binding) => runtime.store.contextConsumerBindings.get(binding[0]) === slot)) {
    fail("Consumer bindings changed during the Context update");
  }

  buttons[1].click();
  await new Promise((resolve) => setTimeout(resolve, 100));
  if (runtime.store.contextUpdateSourceRuns.length !== 1) fail("unrelated action reevaluated the Context source");
  if (runtime.store.completedActionEffectRuns.length !== 1) fail("unrelated action executed the dependent effect");
  if (runtime.store.contextSlots.get(slot) !== 6) fail("unrelated action changed the Context slot");
  if (runtime.diagnostics.length !== 0) fail("Context execution reported runtime diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_CONTEXT_RUNTIME_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_CONTEXT_RUNTIME_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_context_source_failure_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let index = index.replace(
        r#"<script src="./runtime.js"></script>"#,
        r#"<script>
const contextArtifactElement = document.getElementById("presolve-context-runtime");
const contextArtifact = JSON.parse(contextArtifactElement.textContent);
const initialize = contextArtifact.sources[0].program.instructions
  .find((instruction) => instruction.kind === "initialize_context_slot");
initialize.kind = "unsupported_context_instruction";
contextArtifactElement.textContent = JSON.stringify(contextArtifact);
</script>
<script src="./runtime.js"></script>"#,
    );
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const runtime = window.__PRESOLVE__;
  if (runtime.context_initial_source_runs.length !== 0) fail("failed source was recorded as initialized");
  if (runtime.context_slots.length !== 0) fail("failed source populated a Context slot");
  if (runtime.context_consumer_bindings.length !== 2) fail("compiler Consumer bindings were not retained after source failure");
  const slot = runtime.context_consumer_bindings[0][1];
  if (!runtime.context_consumer_bindings.every((binding) => binding[1] === slot)) fail("source failure changed compiler-selected Consumer slots");
  if (!runtime.context_failures.some((failure) => failure.failure === "unsupported-instruction:unsupported_context_instruction")) {
    fail("source failure was not reported from the compiler program evaluator");
  }
  const unavailable = runtime.context_failures.filter((failure) => failure.failure === "source-slot-unavailable");
  if (unavailable.length !== 2) fail("each compiler-bound Consumer did not report the unavailable slot");
  const artifact = JSON.parse(document.getElementById("presolve-context-runtime").textContent);
  if (artifact.sources.length !== 1 || artifact.consumers.length !== 2) fail("runtime reconstructed or reselected Context bindings");
  if (runtime.store.contextConsumerBindings.size !== 2) fail("runtime discarded direct Consumer slot bindings");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_CONTEXT_FAILURE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_CONTEXT_FAILURE_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_batched_computed_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) { resolve(); return; }
    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }
    setTimeout(tick, 20);
  };
  tick();
});
const computedValue = (name) => window.__PRESOLVE__.computed
  .find((entry) => entry.computed.endsWith(`/computed:${name}`))?.value;
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  if (computedValue("doubled") !== 0 || computedValue("label") !== 1) fail("initial computed caches were incorrect");
  document.querySelector("button")?.click();
  await waitFor(() => computedValue("doubled") === 4 && computedValue("label") === 5, "batched computed values");
  if (window.__PRESOLVE__.components[0].state.count !== 2) fail("multi-step action did not finish both state writes");
  if (window.__PRESOLVE__.computed_update_runs !== 1) fail("multi-step action triggered more than one computed update run");
  if (window.__PRESOLVE__.computed.some((entry) => entry.dirty)) fail("computed values remained dirty after the batch");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("computed batching reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_COMPUTED_BATCH_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_COMPUTED_BATCH_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_diamond_computed_probe_page(out_dir: &Path) {
    fs::write(
        out_dir.join("probe.html"),
        r#"<!doctype html>
<html><body><script type="module">
import "./runtime.js";
const fail = (message) => { throw new Error(message); };
const waitFor = async (predicate, label) => {
  const deadline = Date.now() + 3000;
  while (!predicate()) {
    if (Date.now() >= deadline) fail(`timed out waiting for ${label}`);
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
};
const computedValue = (name) => window.__PRESOLVE__.computed
  .find((entry) => entry.computed.endsWith(`/computed:${name}`))?.value;
(async () => {
  await waitFor(() => window.__PRESOLVE__?.ready === true, "runtime readiness");
  if (computedValue("doubled") !== 2 || computedValue("tripled") !== 3 || computedValue("total") !== 5) {
    fail("initial computed diamond values were incorrect");
  }
  document.querySelector("button")?.click();
  await waitFor(() => computedValue("total") === 10, "computed diamond update");
  if (computedValue("doubled") !== 4 || computedValue("tripled") !== 6) fail("diamond prerequisites did not refresh");
  if (window.__PRESOLVE__.computed_update_runs !== 1) fail("diamond action did not flush one batch");
  if (window.__PRESOLVE__.computed.some((entry) => entry.dirty)) fail("computed diamond caches remained dirty");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("computed diamond reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_COMPUTED_DIAMOND_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<pre>PRESOLVE_COMPUTED_DIAMOND_BROWSER_TEST_FAIL: ${error.message}</pre>`);
});
</script></body></html>"#,
    )
    .expect("failed to write computed diamond probe page");
}

fn write_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  const countTarget = document.querySelector("button span");
  if (buttons.length !== 2) fail("expected two buttons");
  if (countTarget === null) fail("nested click target not found");
  if (!document.body.textContent.includes("Count:0")) fail("initial count was not 0");
  if (!document.body.textContent.includes("Mirror:0")) fail("initial mirror was not 0");

  countTarget.click();
  await waitFor(() => document.body.textContent.includes("Count:1"), "count 1");
  if (!document.body.textContent.includes("Mirror:1")) fail("mirror was not 1");

  buttons[1].click();
  await waitFor(() => document.body.textContent.includes("Count:2"), "count 2");
  if (!document.body.textContent.includes("Mirror:2")) fail("mirror was not 2");

  if (document.documentElement.dataset.presolveRuntime !== "ready") {
    fail("runtime was not ready");
  }

  if (!Array.isArray(window.__PRESOLVE__.missingAnchors) || window.__PRESOLVE__.missingAnchors.length !== 0) {
    fail("missing anchors were present");
  }

  if (!Array.isArray(window.__PRESOLVE__.diagnostics) || window.__PRESOLVE__.diagnostics.length !== 0) {
    fail("unexpected diagnostics were present");
  }

  if (window.__PRESOLVE__.runtime_version !== "0.0.0") {
    fail("runtime version was not exposed");
  }

  if (window.__PRESOLVE__.supported_schema_version !== 4) {
    fail("supported schema version was not exposed");
  }

  if (window.__PRESOLVE__.components[0].state.count !== 2) {
    fail("debug state did not update to 2");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  if (!(window.__PRESOLVE__.store.components instanceof Map)) {
    fail("runtime store components was not a Map");
  }

  if (!(window.__PRESOLVE__.store.bindingsByField instanceof Map)) {
    fail("runtime store bindingsByField was not a Map");
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_framework_counter_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html"))
        .expect("failed to read built framework Counter page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => { throw new Error(message); };
const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) return resolve();
    if (Date.now() > deadline) return reject(new Error(`Timed out waiting for ${label}`));
    setTimeout(tick, 20);
  };
  tick();
});
(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");
  const button = document.querySelector("button");
  if (button === null) fail("Counter button was not emitted");
  if (!document.body.textContent.includes("Count: 0")) fail("Counter did not render its initial binding");
  button.click();
  await waitFor(() => document.body.textContent.includes("Count: 1"), "Counter action binding");
  if (window.__PRESOLVE__.components[0].state.count !== 1) fail("compiler state did not update");
  if (window.__PRESOLVE__.diagnostics.length !== 0) fail("runtime reported diagnostics");
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_FRAMEWORK_COUNTER_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_FRAMEWORK_COUNTER_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe)
        .expect("failed to write framework Counter browser probe page");
}

fn write_decrement_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("decrement button was not found");
  if (!document.body.textContent.includes("Count:2")) fail("initial count was not 2");

  button.click();
  await waitFor(() => document.body.textContent.includes("Count:1"), "count 1");

  if (window.__PRESOLVE__.components[0].state.count !== 1) {
    fail("debug state did not update to 1");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_DECREMENT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_DECREMENT_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_add_subtract_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  if (buttons.length !== 2) fail("expected two step buttons");
  if (!document.body.textContent.includes("Add:4")) fail("initial add text was not 4");
  if (!document.body.textContent.includes("Subtract:4")) fail("initial subtract text was not 4");

  const actions = window.__PRESOLVE__.manifest.components[0].actions;
  if (actions[0].operation !== "add_assign" || actions[0].operand !== "2") {
    fail("manifest did not preserve add_assign operand");
  }
  if (actions[1].operation !== "subtract_assign" || actions[1].operand !== "3") {
    fail("manifest did not preserve subtract_assign operand");
  }

  buttons[0].click();
  await waitFor(() => document.body.textContent.includes("Add:6"), "add text 6");
  if (!document.body.textContent.includes("Subtract:6")) fail("subtract text was not 6");

  buttons[1].click();
  await waitFor(() => document.body.textContent.includes("Add:3"), "add text 3");
  if (!document.body.textContent.includes("Subtract:3")) fail("subtract text was not 3");

  if (window.__PRESOLVE__.components[0].state.count !== 3) {
    fail("debug state did not update to 3");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_ADD_SUBTRACT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_ADD_SUBTRACT_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_direct_assignment_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("reset button was not found");
  if (!document.body.textContent.includes("Count:5")) fail("initial count was not 5");

  const action = window.__PRESOLVE__.manifest.components[0].actions[0];
  if (action.operation !== "assign" || action.operand !== "0") {
    fail("manifest did not preserve assign operand");
  }

  button.click();
  await waitFor(() => document.body.textContent.includes("Count:0"), "count 0");

  if (window.__PRESOLVE__.components[0].state.count !== "0") {
    fail("debug state did not preserve assigned literal");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_DIRECT_ASSIGN_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_DIRECT_ASSIGN_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_boolean_toggle_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("toggle button was not found");
  if (!document.body.textContent.includes("Enabled:false")) fail("initial enabled text was not false");

  const action = window.__PRESOLVE__.manifest.components[0].actions[0];
  if (action.operation !== "toggle" || Object.prototype.hasOwnProperty.call(action, "operand")) {
    fail("manifest did not preserve closed toggle operation");
  }

  button.click();
  await waitFor(() => document.body.textContent.includes("Enabled:true"), "enabled true");
  if (window.__PRESOLVE__.components[0].state.enabled !== true) {
    fail("debug state did not update to true");
  }

  button.click();
  await waitFor(() => document.body.textContent.includes("Enabled:false"), "enabled false");
  if (window.__PRESOLVE__.components[0].state.enabled !== false) {
    fail("debug state did not update to false");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_BOOLEAN_TOGGLE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_BOOLEAN_TOGGLE_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_multi_step_action_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("multi-step button was not found");
  if (!document.body.textContent.includes("Count:1Enabled:false")) {
    fail("initial multi-step text did not render");
  }

  const actions = window.__PRESOLVE__.manifest.components[0].actions;
  const operations = actions.map((action) => `${action.method}:${action.operation}:${action.field}`);
  const expected = [
    "apply:add_assign:count",
    "apply:decrement:count",
    "apply:assign:count",
    "apply:increment:count",
    "apply:toggle:enabled"
  ];

  if (JSON.stringify(operations) !== JSON.stringify(expected)) {
    fail(`manifest did not preserve multi-step action order: ${operations.join(",")}`);
  }

  button.click();
  await waitFor(
    () => document.body.textContent.includes("Count:9Enabled:true"),
    "multi-step final text"
  );

  const state = window.__PRESOLVE__.components[0].state;
  if (state.count !== 9 || state.enabled !== true) {
    fail(`debug state did not reflect ordered plan: ${JSON.stringify(state)}`);
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_MULTI_STEP_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_MULTI_STEP_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_dynamic_attributes_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("dynamic attribute button was not found");
  if (button.hasAttribute("disabled")) fail("disabled attribute should be omitted initially");
  if (button.disabled !== false) fail("disabled property should be false initially");
  if (button.getAttribute("title") !== "Ready") fail("title attribute was not initialized");
  if (!document.body.textContent.includes("Status:Ready")) fail("initial status text was not Ready");

  const nodes = window.__PRESOLVE__.manifest.components[0].template.nodes;
  const disabledBinding = nodes.find((node) => node.attribute === "disabled");
  const titleBinding = nodes.find((node) => node.attribute === "title");
  if (disabledBinding?.target !== "attribute" || disabledBinding?.element !== "n0") {
    fail("disabled binding target metadata was not emitted");
  }
  if (titleBinding?.target !== "attribute" || titleBinding?.element !== "n0") {
    fail("title binding target metadata was not emitted");
  }

  button.click();
  await waitFor(
    () => button.disabled === true &&
      button.hasAttribute("disabled") &&
      button.getAttribute("title") === "Locked" &&
      document.body.textContent.includes("Status:Locked"),
    "dynamic attributes locked"
  );

  const state = window.__PRESOLVE__.components[0].state;
  if (state.disabled !== true || state.label !== "Locked") {
    fail(`debug state did not reflect dynamic attribute update: ${JSON.stringify(state)}`);
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_DYNAMIC_ATTRIBUTES_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_DYNAMIC_ATTRIBUTES_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_fragments_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const elements = Array.from(document.body.children)
    .filter((element) => ["H1", "P", "SPAN"].includes(element.tagName));
  const tags = elements.map((element) => element.tagName.toLowerCase());
  if (JSON.stringify(tags) !== JSON.stringify(["h1", "p", "span"])) {
    fail(`fragment siblings did not render in order: ${tags.join(",")}`);
  }

  if (document.querySelector("[data-presolve-node='n0'], [data-presolve-node='n2']") !== null) {
    fail("fragment IDs should not render as wrapper DOM nodes");
  }

  const h1 = document.querySelector("h1");
  const paragraph = document.querySelector("p");
  const span = document.querySelector("span");
  if (h1?.dataset.presolveNode !== "n1") fail("heading node identity was not preserved");
  if (paragraph?.dataset.presolveNode !== "n3") fail("paragraph node identity was not preserved");
  if (span?.dataset.presolveNode !== "n5") fail("span node identity was not preserved");
  if (!document.body.textContent.includes("TitleStatus:ReadyDone")) {
    fail("fragment text content did not render expected sibling output");
  }

  const nodes = window.__PRESOLVE__.manifest.components[0].template.nodes;
  const nodeIds = nodes.map((node) => node.id);
  if (JSON.stringify(nodeIds) !== JSON.stringify(["n1", "n3", "n4", "n5"])) {
    fail(`fragment manifest should omit fragment nodes: ${nodeIds.join(",")}`);
  }

  if (window.__PRESOLVE__.components[0].state.label !== "Ready") {
    fail("fragment component state did not initialize");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_FRAGMENTS_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_FRAGMENTS_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_conditional_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) {
    fail("conditional fixture button was not found");
  }

  if (button.textContent.trim() !== "On") {
    fail(`initial conditional branch was not On: ${button.textContent}`);
  }

  if (document.querySelector("[data-presolve-node='n4']") === null) {
    fail("true branch node identity was not present initially");
  }

  const manifestNode = window.__PRESOLVE__.manifest.components[0].template.nodes[1];
  if (manifestNode.kind !== "conditional" || manifestNode.start !== "n2" || manifestNode.end !== "n3") {
    fail("conditional manifest node did not expose stable boundaries");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "Off", "false branch");

  if (document.querySelector("[data-presolve-node='n4']") !== null) {
    fail("true branch node should have been removed after toggle");
  }

  if (document.querySelector("[data-presolve-node='n5']") === null) {
    fail("false branch node identity was not present after toggle");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "On", "true branch");

  if (document.querySelector("[data-presolve-node='n4']") === null) {
    fail("true branch node identity was not restored after second toggle");
  }

  if (window.__PRESOLVE__.store.elementsByNode.get("n4") !== document.querySelector("[data-presolve-node='n4']")) {
    fail("runtime element index did not refresh after conditional replacement");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_CONDITIONAL_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_CONDITIONAL_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

#[allow(clippy::too_many_lines)]
fn write_keyed_list_reconciliation_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

const listLabels = () => [...document.querySelectorAll("ol li")].map((item) => item.textContent);

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  if (buttons.length !== 2) fail("expected list reconciliation controls");

  if (JSON.stringify(listLabels()) !== JSON.stringify(["0:North", "1:South"])) {
    fail(`initial list contents were unexpected: ${listLabels().join(" | ")}`);
  }

  const north = document.querySelector("[data-presolve-node='n7:North']");
  const south = document.querySelector("[data-presolve-node='n7:South']");
  if (north === null || south === null) fail("initial keyed list nodes were not found");

  const listNode = window.__PRESOLVE__.manifest.components[0].template.nodes.find(
    (node) => node.kind === "list"
  );
  if (
    listNode === undefined ||
    listNode.start !== "n5" ||
    listNode.end !== "n6" ||
    listNode.item_root !== "n7"
  ) {
    fail("list manifest node did not expose stable anchors and item root");
  }

  buttons[0].click();
  await waitFor(
    () => JSON.stringify(listLabels()) === JSON.stringify(["0:South", "1:East", "2:North"]),
    "reconciled list"
  );

  if (document.querySelector("[data-presolve-node='n7:North']") !== north) {
    fail("North node was recreated instead of retained during movement");
  }
  if (document.querySelector("[data-presolve-node='n7:South']") !== south) {
    fail("South node was recreated instead of retained during movement");
  }
  const east = document.querySelector("[data-presolve-node='n7:East']");
  if (east === null) {
    fail("East node was not inserted during reconciliation");
  }
  if (window.__PRESOLVE__.store.elementsByNode.get("n7:North") !== north) {
    fail("runtime element index did not retain the North node");
  }
  if (JSON.stringify(window.__PRESOLVE__.components[0].state.labels) !== JSON.stringify(["South", "East", "North"])) {
    fail("debug state did not retain the reconciled array");
  }

  buttons[1].click();
  await waitFor(
    () => JSON.stringify(listLabels()) === JSON.stringify(["0:East"]),
    "trimmed list"
  );

  if (document.querySelector("[data-presolve-node='n7:East']") !== east) {
    fail("East node was recreated instead of retained during deletion");
  }
  if (document.querySelector("[data-presolve-node='n7:North']") !== null || document.querySelector("[data-presolve-node='n7:South']") !== null) {
    fail("stale keyed list nodes were not removed during deletion");
  }
  if (JSON.stringify(window.__PRESOLVE__.components[0].state.labels) !== JSON.stringify(["East"])) {
    fail("debug state did not retain the trimmed array");
  }
  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_KEYED_LIST_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_KEYED_LIST_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

#[allow(clippy::too_many_lines)]
fn write_object_keyed_list_reconciliation_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

const listLabels = () => [...document.querySelectorAll("ol li")].map((item) => item.textContent);

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  if (buttons.length !== 2) fail("expected object list reconciliation controls");

  if (JSON.stringify(listLabels()) !== JSON.stringify(["0:North(west)", "1:South(east)"])) {
    fail(`initial object list contents were unexpected: ${listLabels().join(" | ")}`);
  }

  const north = document.querySelector("[data-presolve-node='n7:north']");
  const south = document.querySelector("[data-presolve-node='n7:south']");
  if (north === null || south === null) fail("initial object keyed list nodes were not found");

  const listNode = window.__PRESOLVE__.manifest.components[0].template.nodes.find(
    (node) => node.kind === "list"
  );
  if (
    listNode === undefined ||
    listNode.start !== "n5" ||
    listNode.end !== "n6" ||
    listNode.item_root !== "n7" ||
    listNode.key_expression !== "item.id"
  ) {
    fail("object list manifest node did not expose member-key reconciliation metadata");
  }

  buttons[0].click();
  await waitFor(
    () => JSON.stringify(listLabels()) === JSON.stringify(["0:South(east)", "1:East(central)", "2:North(west)"]),
    "reconciled object list"
  );

  if (document.querySelector("[data-presolve-node='n7:north']") !== north) {
    fail("North object node was recreated instead of retained during movement");
  }
  if (document.querySelector("[data-presolve-node='n7:south']") !== south) {
    fail("South object node was recreated instead of retained during movement");
  }
  const east = document.querySelector("[data-presolve-node='n7:east']");
  if (east === null || east.textContent !== "1:East(central)") {
    fail("East object node did not receive member bindings during insertion");
  }
  if (window.__PRESOLVE__.store.elementsByNode.get("n7:north") !== north) {
    fail("runtime element index did not retain the North object node");
  }
  if (JSON.stringify(window.__PRESOLVE__.components[0].state.items.map((item) => item.id)) !== JSON.stringify(["south", "east", "north"])) {
    fail("debug state did not retain the reconciled object array");
  }

  buttons[1].click();
  await waitFor(
    () => JSON.stringify(listLabels()) === JSON.stringify(["0:East(central)"]),
    "trimmed object list"
  );

  if (document.querySelector("[data-presolve-node='n7:east']") !== east) {
    fail("East object node was recreated instead of retained during deletion");
  }
  if (document.querySelector("[data-presolve-node='n7:north']") !== null || document.querySelector("[data-presolve-node='n7:south']") !== null) {
    fail("stale object keyed list nodes were not removed during deletion");
  }
  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_OBJECT_KEYED_LIST_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_OBJECT_KEYED_LIST_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

#[allow(clippy::too_many_lines)]
fn write_dynamic_list_item_behavior_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

const itemButtons = () => [...document.querySelectorAll("ol li button")];

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const refresh = document.querySelector("section > button");
  const selections = document.querySelector("p");
  const north = document.querySelector("[data-presolve-node='n8:north']");
  const south = document.querySelector("[data-presolve-node='n8:south']");
  if (refresh === null || selections === null || north === null || south === null) {
    fail("initial dynamic list controls or roots were missing");
  }
  if (north.title !== "west" || north.dataset.label !== "North") {
    fail("initial list attributes were not rendered from item members");
  }

  itemButtons()[0].click();
  await waitFor(() => selections.textContent === "1", "initial list item event");

  refresh.click();
  await waitFor(
    () => itemButtons().map((button) => button.textContent).join("|") === "0:Northern(central)|1:East(coastal)|2:Southern(mountain)",
    "refreshed list item bindings"
  );

  if (document.querySelector("[data-presolve-node='n8:north']") !== north) {
    fail("North root was replaced instead of refreshed in place");
  }
  if (document.querySelector("[data-presolve-node='n8:south']") !== south) {
    fail("South root was replaced instead of refreshed in place");
  }
  if (north.title !== "central" || north.dataset.label !== "Northern") {
    fail("retained North attributes did not refresh");
  }
  if (south.title !== "mountain" || south.dataset.label !== "Southern") {
    fail("retained South attributes did not refresh");
  }

  const east = document.querySelector("[data-presolve-node='n8:east']");
  if (east === null || east.title !== "coastal" || east.dataset.label !== "East") {
    fail("inserted East attributes were not materialized");
  }

  itemButtons()[1].click();
  await waitFor(() => selections.textContent === "2", "inserted list item event");
  itemButtons()[0].click();
  await waitFor(() => selections.textContent === "3", "retained list item event");

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_DYNAMIC_LIST_ITEM_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_DYNAMIC_LIST_ITEM_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_logical_and_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) {
    fail("logical-and fixture button was not found");
  }

  if (button.textContent.trim() !== "On") {
    fail(`initial logical-and branch was not On: ${button.textContent}`);
  }

  const manifestNode = window.__PRESOLVE__.manifest.components[0].template.nodes[1];
  if (manifestNode.kind !== "conditional" || manifestNode.when_false_html !== "") {
    fail("logical-and manifest node did not expose an empty false branch");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "", "empty false branch");

  if (document.querySelector("[data-presolve-node='n4']") !== null) {
    fail("true branch node should have been removed for logical-and false branch");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "On", "restored true branch");

  if (document.querySelector("[data-presolve-node='n4']") === null) {
    fail("true branch node identity was not restored for logical-and true branch");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_LOGICAL_AND_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_LOGICAL_AND_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_runtime_contract_probe_page(
    out_dir: &Path,
    file_name: &str,
    manifest_json: Option<&str>,
    expected_code: &str,
    pass_marker: &str,
) {
    let mut page = String::new();

    page.push_str("<!doctype html>\n<html lang=\"en\">\n<body>\n");

    if let Some(manifest_json) = manifest_json {
        page.push_str("<script type=\"application/json\" id=\"presolve-template-manifest\">\n");
        page.push_str(manifest_json);
        page.push_str("\n</script>\n");
    }

    page.push_str("<script src=\"./runtime.js\"></script>\n");
    page.push_str("<script>\n");
    page.push_str("const fail = (message) => { throw new Error(message); };\n");
    page.push_str("const waitFor = (predicate, label) => new Promise((resolve, reject) => {\n");
    page.push_str("  const deadline = Date.now() + 3000;\n");
    page.push_str("  const tick = () => {\n");
    page.push_str("    if (predicate()) { resolve(); return; }\n");
    page.push_str("    if (Date.now() > deadline) { reject(new Error(`Timed out waiting for ${label}`)); return; }\n");
    page.push_str("    setTimeout(tick, 20);\n");
    page.push_str("  };\n");
    page.push_str("  tick();\n");
    page.push_str("});\n");
    page.push_str("(async () => {\n");
    page.push_str("  await waitFor(() => document.documentElement.dataset.presolveRuntime === \"error\" && window.__PRESOLVE__, \"runtime error\");\n");
    page.push_str("  if (window.__PRESOLVE__.runtime_version !== \"0.0.0\") fail(\"runtime version was not exposed\");\n");
    page.push_str("  if (window.__PRESOLVE__.supported_schema_version !== 4) fail(\"supported schema version was not exposed\");\n");
    page.push_str("  const diagnostics = window.__PRESOLVE__.diagnostics;\n");
    page.push_str("  if (!Array.isArray(diagnostics) || diagnostics.length === 0) fail(\"diagnostics were not exposed\");\n");
    page.push_str("  if (diagnostics[0].code !== \"");
    page.push_str(expected_code);
    page.push_str("\") fail(`expected ");
    page.push_str(expected_code);
    page.push_str(" but saw ${diagnostics[0].code}`);\n");
    page.push_str("  if (diagnostics[0].fatal !== true) fail(\"diagnostic was not fatal\");\n");
    page.push_str("  document.body.dataset.browserTest = \"pass\";\n");
    page.push_str("  document.body.insertAdjacentHTML(\"beforeend\", \"<div>");
    page.push_str(pass_marker);
    page.push_str("</div>\");\n");
    page.push_str("})().catch((error) => {\n");
    page.push_str("  document.body.dataset.browserTest = \"fail\";\n");
    page.push_str("  document.body.insertAdjacentHTML(\"beforeend\", `<div>PRESOLVE_RUNTIME_CONTRACT_DIAGNOSTIC_FAIL: ${error.message}</div>`);\n");
    page.push_str("  console.error(error);\n");
    page.push_str("});\n");
    page.push_str("</script>\n</body>\n</html>\n");

    fs::write(out_dir.join(file_name), page).expect("failed to write browser probe page");
}

fn write_string_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  if (!document.body.textContent.includes("Name:Austin & <Zero>")) {
    fail("string binding text was not rendered");
  }

  if (window.__PRESOLVE__.components[0].state.name !== "Austin & <Zero>") {
    fail("debug state did not preserve string value");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_STRING_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_STRING_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_boolean_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  if (!document.body.textContent.includes("Enabled:true")) {
    fail("true boolean binding text was not rendered");
  }

  if (!document.body.textContent.includes("Disabled:false")) {
    fail("false boolean binding text was not rendered");
  }

  const state = window.__PRESOLVE__.components[0].state;
  if (state.enabled !== true) {
    fail("enabled state did not preserve boolean true");
  }

  if (state.disabled !== false) {
    fail("disabled state did not preserve boolean false");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_BOOLEAN_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_BOOLEAN_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_null_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const presolveConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  presolveConsoleErrors.push(message);
  originalConsoleError(...args);
};

const waitFor = (predicate, label) => new Promise((resolve, reject) => {
  const deadline = Date.now() + 3000;
  const tick = () => {
    if (predicate()) {
      resolve();
      return;
    }

    if (Date.now() > deadline) {
      reject(new Error(`Timed out waiting for ${label}`));
      return;
    }

    setTimeout(tick, 20);
  };

  tick();
});

(async () => {
  await waitFor(() => document.documentElement.dataset.presolveRuntime === "ready", "runtime ready");

  const paragraph = document.querySelector("p");
  if (paragraph === null) {
    fail("null fixture paragraph was not found");
  }

  if (paragraph.textContent !== "Selection:") {
    fail(`null binding text was not empty: ${paragraph.textContent}`);
  }

  const manifestNode = window.__PRESOLVE__.manifest.components[0].template.nodes[1];
  if (manifestNode.initial_value !== null) {
    fail("manifest did not preserve null initial value");
  }

  const state = window.__PRESOLVE__.components[0].state;
  if (state.selection !== null) {
    fail("selection state did not preserve null");
  }

  if (presolveConsoleErrors.some((message) => message.includes("[Presolve]"))) {
    fail(`unexpected Presolve console error: ${presolveConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>PRESOLVE_NULL_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>PRESOLVE_NULL_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

#[allow(clippy::too_many_lines)]
fn run_chrome_with_timeout(chrome: PathBuf, args: &[&str], timeout: Duration) -> Output {
    let mut child = Command::new(chrome)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run headless Chrome");
    let stdout = child
        .stdout
        .take()
        .expect("headless Chrome stdout was not piped");
    let stderr = child
        .stderr
        .take()
        .expect("headless Chrome stderr was not piped");
    let stdout_output = Arc::new(Mutex::new(Vec::new()));
    let stdout_buffer = Arc::clone(&stdout_output);
    let mut stdout_reader = Some(thread::spawn(move || {
        let mut stdout = stdout;
        let mut chunk = [0_u8; 16 * 1024];
        while let Ok(count) = stdout.read(&mut chunk) {
            if count == 0 {
                break;
            }
            stdout_buffer
                .lock()
                .expect("headless Chrome stdout buffer was poisoned")
                .extend_from_slice(&chunk[..count]);
        }
    }));
    let stderr_output = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::clone(&stderr_output);
    let mut stderr_reader = Some(thread::spawn(move || {
        let mut stderr = stderr;
        let mut chunk = [0_u8; 16 * 1024];
        while let Ok(count) = stderr.read(&mut chunk) {
            if count == 0 {
                break;
            }
            stderr_buffer
                .lock()
                .expect("headless Chrome stderr buffer was poisoned")
                .extend_from_slice(&chunk[..count]);
        }
    }));

    let started = Instant::now();

    loop {
        let dom_complete = stdout_output
            .lock()
            .expect("headless Chrome stdout buffer was poisoned")
            .windows(b"</html>".len())
            .any(|window| window == b"</html>");
        if dom_complete {
            let status = child
                .try_wait()
                .expect("failed to poll headless Chrome")
                .unwrap_or_else(|| {
                    child
                        .kill()
                        .expect("failed to stop headless Chrome after complete DOM output");
                    child
                        .wait()
                        .expect("failed to wait for headless Chrome after complete DOM output")
                });
            thread::sleep(Duration::from_millis(20));
            return Output {
                status,
                stdout: stdout_output
                    .lock()
                    .expect("headless Chrome stdout buffer was poisoned")
                    .clone(),
                stderr: stderr_output
                    .lock()
                    .expect("headless Chrome stderr buffer was poisoned")
                    .clone(),
            };
        }

        if let Some(status) = child.try_wait().expect("failed to poll headless Chrome") {
            stdout_reader
                .take()
                .expect("headless Chrome stdout reader was missing")
                .join()
                .expect("headless Chrome stdout reader panicked");
            stderr_reader
                .take()
                .expect("headless Chrome stderr reader was missing")
                .join()
                .expect("headless Chrome stderr reader panicked");
            return Output {
                status,
                stdout: stdout_output
                    .lock()
                    .expect("headless Chrome stdout buffer was poisoned")
                    .clone(),
                stderr: stderr_output
                    .lock()
                    .expect("headless Chrome stderr buffer was poisoned")
                    .clone(),
            };
        }

        if started.elapsed() > timeout {
            child
                .kill()
                .expect("failed to stop timed-out headless Chrome");
            let status = child
                .wait()
                .expect("failed to wait for timed-out headless Chrome");
            thread::sleep(Duration::from_millis(20));
            return Output {
                status,
                stdout: stdout_output
                    .lock()
                    .expect("headless Chrome stdout buffer was poisoned")
                    .clone(),
                stderr: stderr_output
                    .lock()
                    .expect("headless Chrome stderr buffer was poisoned")
                    .clone(),
            };
        }

        thread::sleep(Duration::from_millis(50));
    }
}

struct StaticServer {
    port: u16,
    stop: Arc<AtomicBool>,
}

impl StaticServer {
    fn start(root: PathBuf) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind static test server");
        let port = listener
            .local_addr()
            .expect("failed to read static server address")
            .port();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);

        thread::spawn(move || {
            listener
                .set_nonblocking(true)
                .expect("failed to make static server nonblocking");

            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => serve_request(stream, &root),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Self { port, stop }
    }

    fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

fn serve_request(mut stream: TcpStream, root: &Path) {
    let mut buffer = [0_u8; 1024];
    let Ok(read) = stream.read(&mut buffer) else {
        return;
    };

    let request = String::from_utf8_lossy(&buffer[..read]);
    let Some(path) = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
    else {
        write_response(&mut stream, "400 Bad Request", "text/plain", b"bad request");
        return;
    };

    let relative_path = path.trim_start_matches('/');
    let file_name = if relative_path.is_empty() {
        "index.html"
    } else {
        relative_path
    };

    if file_name.contains("..") {
        write_response(&mut stream, "403 Forbidden", "text/plain", b"forbidden");
        return;
    }

    let file_path = root.join(file_name);

    match fs::read(&file_path) {
        Ok(body) => {
            let content_type = match file_path
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                Some("json") => "application/json; charset=utf-8",
                _ => "application/octet-stream",
            };
            write_response(&mut stream, "200 OK", content_type, &body);
        }
        Err(_) => {
            write_response(&mut stream, "404 Not Found", "text/plain", b"not found");
        }
    }
}

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );

    let _ = stream.write_all(headers.as_bytes());
    let _ = stream.write_all(body);
}
