use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize repo root")
}

fn ezc_cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ezc_cli"))
}

fn chrome_bin() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("EDGEZERO_CHROME") {
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
    let mut args = vec![
        "--headless=new".to_string(),
        "--disable-gpu".to_string(),
        "--no-first-run".to_string(),
        "--disable-background-networking".to_string(),
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
    run_chrome_with_timeout(chrome, &arg_refs, Duration::from_secs(5))
}

#[test]
fn double_binding_counter_increments_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/double-binding-counter");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn decrement_counter_decrements_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/decrement-counter");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_DECREMENT_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn add_subtract_assign_counter_updates_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/add-subtract-assign");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_ADD_SUBTRACT_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn direct_assignment_counter_resets_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/direct-assignment");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_DIRECT_ASSIGN_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn boolean_toggle_flips_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/boolean-toggle");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_BOOLEAN_TOGGLE_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn multi_step_action_runs_all_steps_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/multi-step-action");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_MULTI_STEP_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn string_state_initializes_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/string-greeting");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_STRING_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn boolean_state_initializes_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/boolean-flags");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_BOOLEAN_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn null_state_initializes_in_a_real_browser() {
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/null-selection");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_NULL_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

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

  if (document.documentElement.dataset.ezRuntime !== "ready") {
    fail("runtime was not ready");
  }

  if (!Array.isArray(window.__EDGEZERO__.missingAnchors) || window.__EDGEZERO__.missingAnchors.length !== 0) {
    fail("missing anchors were present");
  }

  if (window.__EDGEZERO__.components[0].state.count !== 2) {
    fail("debug state did not update to 2");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  if (!(window.__EDGEZERO__.store.components instanceof Map)) {
    fail("runtime store components was not a Map");
  }

  if (!(window.__EDGEZERO__.store.bindingsByField instanceof Map)) {
    fail("runtime store bindingsByField was not a Map");
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_decrement_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("decrement button was not found");
  if (!document.body.textContent.includes("Count:2")) fail("initial count was not 2");

  button.click();
  await waitFor(() => document.body.textContent.includes("Count:1"), "count 1");

  if (window.__EDGEZERO__.components[0].state.count !== 1) {
    fail("debug state did not update to 1");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_DECREMENT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_DECREMENT_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  if (buttons.length !== 2) fail("expected two step buttons");
  if (!document.body.textContent.includes("Add:4")) fail("initial add text was not 4");
  if (!document.body.textContent.includes("Subtract:4")) fail("initial subtract text was not 4");

  const actions = window.__EDGEZERO__.manifest.components[0].actions;
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

  if (window.__EDGEZERO__.components[0].state.count !== 3) {
    fail("debug state did not update to 3");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_ADD_SUBTRACT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_ADD_SUBTRACT_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("reset button was not found");
  if (!document.body.textContent.includes("Count:5")) fail("initial count was not 5");

  const action = window.__EDGEZERO__.manifest.components[0].actions[0];
  if (action.operation !== "assign" || action.operand !== "0") {
    fail("manifest did not preserve assign operand");
  }

  button.click();
  await waitFor(() => document.body.textContent.includes("Count:0"), "count 0");

  if (window.__EDGEZERO__.components[0].state.count !== "0") {
    fail("debug state did not preserve assigned literal");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_DIRECT_ASSIGN_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_DIRECT_ASSIGN_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("toggle button was not found");
  if (!document.body.textContent.includes("Enabled:false")) fail("initial enabled text was not false");

  const action = window.__EDGEZERO__.manifest.components[0].actions[0];
  if (action.operation !== "toggle" || Object.prototype.hasOwnProperty.call(action, "operand")) {
    fail("manifest did not preserve closed toggle operation");
  }

  button.click();
  await waitFor(() => document.body.textContent.includes("Enabled:true"), "enabled true");
  if (window.__EDGEZERO__.components[0].state.enabled !== true) {
    fail("debug state did not update to true");
  }

  button.click();
  await waitFor(() => document.body.textContent.includes("Enabled:false"), "enabled false");
  if (window.__EDGEZERO__.components[0].state.enabled !== false) {
    fail("debug state did not update to false");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_BOOLEAN_TOGGLE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_BOOLEAN_TOGGLE_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const button = document.querySelector("button");
  if (button === null) fail("multi-step button was not found");
  if (!document.body.textContent.includes("Count:1Enabled:false")) {
    fail("initial multi-step text did not render");
  }

  const actions = window.__EDGEZERO__.manifest.components[0].actions;
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

  const state = window.__EDGEZERO__.components[0].state;
  if (state.count !== 9 || state.enabled !== true) {
    fail(`debug state did not reflect ordered plan: ${JSON.stringify(state)}`);
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_MULTI_STEP_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_MULTI_STEP_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn write_string_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
};

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  if (!document.body.textContent.includes("Name:Austin & <Zero>")) {
    fail("string binding text was not rendered");
  }

  if (window.__EDGEZERO__.components[0].state.name !== "Austin & <Zero>") {
    fail("debug state did not preserve string value");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_STRING_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_STRING_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  if (!document.body.textContent.includes("Enabled:true")) {
    fail("true boolean binding text was not rendered");
  }

  if (!document.body.textContent.includes("Disabled:false")) {
    fail("false boolean binding text was not rendered");
  }

  const state = window.__EDGEZERO__.components[0].state;
  if (state.enabled !== true) {
    fail("enabled state did not preserve boolean true");
  }

  if (state.disabled !== false) {
    fail("disabled state did not preserve boolean false");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_BOOLEAN_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_BOOLEAN_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const edgezeroConsoleErrors = [];
const originalConsoleError = console.error.bind(console);
console.error = (...args) => {
  const message = args.map((arg) => {
    if (arg instanceof Error) return arg.message;
    if (typeof arg === "string") return arg;
    return JSON.stringify(arg);
  }).join(" ");

  edgezeroConsoleErrors.push(message);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const paragraph = document.querySelector("p");
  if (paragraph === null) {
    fail("null fixture paragraph was not found");
  }

  if (paragraph.textContent !== "Selection:") {
    fail(`null binding text was not empty: ${paragraph.textContent}`);
  }

  const manifestNode = window.__EDGEZERO__.manifest.components[0].template.nodes[1];
  if (manifestNode.initial_value !== null) {
    fail("manifest did not preserve null initial value");
  }

  const state = window.__EDGEZERO__.components[0].state;
  if (state.selection !== null) {
    fail("selection state did not preserve null");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_NULL_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_NULL_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
}

fn run_chrome_with_timeout(chrome: PathBuf, args: &[&str], timeout: Duration) -> Output {
    let mut child = Command::new(chrome)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run headless Chrome");

    let started = Instant::now();

    loop {
        if child
            .try_wait()
            .expect("failed to poll headless Chrome")
            .is_some()
        {
            return child
                .wait_with_output()
                .expect("failed to collect headless Chrome output");
        }

        if started.elapsed() > timeout {
            child
                .kill()
                .expect("failed to stop timed-out headless Chrome");
            return child
                .wait_with_output()
                .expect("failed to collect timed-out headless Chrome output");
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
