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
    run_chrome_with_timeout(chrome, &arg_refs, Duration::from_secs(20))
}

#[test]
fn double_binding_counter_increments_in_a_real_browser() {
    let _guard = browser_test_guard();
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
    let _guard = browser_test_guard();
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
    let _guard = browser_test_guard();
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
    let _guard = browser_test_guard();
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
    let _guard = browser_test_guard();
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
    let _guard = browser_test_guard();
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
fn dynamic_attributes_update_in_a_real_browser() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/dynamic-attributes");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_DYNAMIC_ATTRIBUTES_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/fragments");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_FRAGMENTS_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/conditional-rendering");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_CONDITIONAL_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/logical-and-conditional");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_LOGICAL_AND_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/keyed-list-reconciliation");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_KEYED_LIST_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/object-keyed-list-reconciliation");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_OBJECT_KEYED_LIST_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/dynamic-list-item-behavior");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
        stdout.contains("EDGEZERO_DYNAMIC_LIST_ITEM_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/runtime-contract");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        .expect("failed to run ezc_cli build");

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
            "EZR_MISSING_MANIFEST",
            "EDGEZERO_MISSING_MANIFEST_DIAGNOSTIC_PASS",
        ),
        (
            "invalid-json.html",
            Some("{"),
            "EZR_INVALID_MANIFEST_JSON",
            "EDGEZERO_INVALID_JSON_DIAGNOSTIC_PASS",
        ),
        (
            "unsupported-schema.html",
            Some(r#"{"schema_version":999,"components":[]}"#),
            "EZR_UNSUPPORTED_SCHEMA",
            "EDGEZERO_UNSUPPORTED_SCHEMA_DIAGNOSTIC_PASS",
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
    let _guard = browser_test_guard();
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
    let _guard = browser_test_guard();
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

#[test]
fn computed_values_execute_once_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/computed-runtime-execution");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_COMPUTED_RUNTIME_BROWSER_TEST_PASS"),
        "browser probe did not pass\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn initial_effects_execute_once_from_compiler_generated_runtime_programs() {
    let _guard = browser_test_guard();
    let repo_root = repo_root();
    let out_dir = repo_root.join("target/ezc-browser-test/initial-effect-runtime");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_INITIAL_EFFECT_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/completed-action-effect-runtime");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_COMPLETED_ACTION_EFFECT_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/context-runtime-matrix");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_CONTEXT_RUNTIME_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/context-source-failure");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_CONTEXT_FAILURE_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/computed-batched-invalidation");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_COMPUTED_BATCH_BROWSER_TEST_PASS"),
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
    let out_dir = repo_root.join("target/ezc-browser-test/computed-diamond");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("failed to clean previous browser test output");
    }

    let output = Command::new(ezc_cli_bin())
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
        stdout.contains("EDGEZERO_COMPUTED_DIAMOND_BROWSER_TEST_PASS"),
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");
  const computed = window.__EDGEZERO__.computed;
  if (!Array.isArray(computed) || computed.length !== 2) fail("computed cache records were missing");
  const doubled = computed.find((entry) => entry.computed.endsWith("/computed:doubled"));
  const label = computed.find((entry) => entry.computed.endsWith("/computed:label"));
  if (doubled?.value !== 2 || label?.value !== 3) fail("computed programs did not evaluate in compiler order");
  if (doubled?.dirty !== false || label?.dirty !== false) fail("computed caches remained dirty after execution");
  if (!(window.__EDGEZERO__.store.computedCaches instanceof Map)) fail("computed cache store was not a Map");
  if (window.__EDGEZERO__.diagnostics.length !== 0) fail("computed execution reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_COMPUTED_RUNTIME_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_COMPUTED_RUNTIME_BROWSER_TEST_FAIL: ${error.message}</div>`);
  console.error(error);
});
</script>
</body>"#,
    );

    fs::write(out_dir.join("probe.html"), probe).expect("failed to write browser probe page");
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");
  if (document.title !== "EdgeZero initial effect") fail("effect did not update document title");
  if (localStorage.getItem("edgezero-effect-initial") !== "ready") fail("effect did not update local storage");
  const runs = window.__EDGEZERO__.initial_effect_runs;
  if (!Array.isArray(runs) || runs.length !== 1) fail("initial effect did not execute exactly once");
  const run = runs[0];
  if (!run.effect.endsWith("/effect:report") || run.effect_batch_index !== 0) fail("initial effect debug evidence was not deterministic");
  const operations = run.capability_operations.map((operation) => operation.runtime_lowering).join("|");
  if (operations !== "builtin.browser.console.log|builtin.browser.document.title.assign|builtin.browser.local_storage.set_item") {
    fail("effect capability dispatch order did not match the compiler program");
  }
  if (window.__EDGEZERO__.computed.find((entry) => entry.computed.endsWith("/computed:doubled"))?.value !== 4) {
    fail("effect did not observe initialized computed state");
  }
  if (window.__EDGEZERO__.diagnostics.length !== 0) fail("initial effect execution reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_INITIAL_EFFECT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_INITIAL_EFFECT_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");
  const manifestEvent = window.__EDGEZERO__.manifest.components[0].template.events[0];
  if (manifestEvent.kind !== "action" || !manifestEvent.method_id || !manifestEvent.action_batch_id) {
    fail("template manifest did not carry canonical action activation identities");
  }
  document.querySelector("button")?.click();
  await waitFor(() => window.__EDGEZERO__.completed_action_effect_runs.length === 1, "completed action effect");
  if (document.title !== "EdgeZero after action") fail("completed action effect did not synchronize title");
  const run = window.__EDGEZERO__.completed_action_effect_runs[0];
  if (run.action_batch_id !== manifestEvent.action_batch_id || run.effect_batch_index !== 0) {
    fail("runtime did not consume the exact compiler action batch plan");
  }
  if (run.capability_operations.length !== 3) fail("completed action effect did not preserve capability program");
  if (window.__EDGEZERO__.initial_effect_runs.length !== 1) fail("initial effect plan was replayed after action");
  if (window.__EDGEZERO__.computed.find((entry) => entry.computed.endsWith("/computed:doubled"))?.value !== 6) {
    fail("completed action effect ran before compiler computed flush");
  }
  if (window.__EDGEZERO__.store.activeActionBatch !== null) fail("runtime retained an active action batch");
  if (window.__EDGEZERO__.diagnostics.length !== 0) fail("completed action effect reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_COMPLETED_ACTION_EFFECT_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_COMPLETED_ACTION_EFFECT_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");
  const runtime = window.__EDGEZERO__;
  const artifactElement = document.getElementById("ez-context-runtime");
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
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_CONTEXT_RUNTIME_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_CONTEXT_RUNTIME_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
const contextArtifactElement = document.getElementById("ez-context-runtime");
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
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");
  const runtime = window.__EDGEZERO__;
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
  const artifact = JSON.parse(document.getElementById("ez-context-runtime").textContent);
  if (artifact.sources.length !== 1 || artifact.consumers.length !== 2) fail("runtime reconstructed or reselected Context bindings");
  if (runtime.store.contextConsumerBindings.size !== 2) fail("runtime discarded direct Consumer slot bindings");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_CONTEXT_FAILURE_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_CONTEXT_FAILURE_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
const computedValue = (name) => window.__EDGEZERO__.computed
  .find((entry) => entry.computed.endsWith(`/computed:${name}`))?.value;
(async () => {
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");
  if (computedValue("doubled") !== 0 || computedValue("label") !== 1) fail("initial computed caches were incorrect");
  document.querySelector("button")?.click();
  await waitFor(() => computedValue("doubled") === 4 && computedValue("label") === 5, "batched computed values");
  if (window.__EDGEZERO__.components[0].state.count !== 2) fail("multi-step action did not finish both state writes");
  if (window.__EDGEZERO__.computed_update_runs !== 1) fail("multi-step action triggered more than one computed update run");
  if (window.__EDGEZERO__.computed.some((entry) => entry.dirty)) fail("computed values remained dirty after the batch");
  if (window.__EDGEZERO__.diagnostics.length !== 0) fail("computed batching reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_COMPUTED_BATCH_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_COMPUTED_BATCH_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
const computedValue = (name) => window.__EDGEZERO__.computed
  .find((entry) => entry.computed.endsWith(`/computed:${name}`))?.value;
(async () => {
  await waitFor(() => window.__EDGEZERO__?.ready === true, "runtime readiness");
  if (computedValue("doubled") !== 2 || computedValue("tripled") !== 3 || computedValue("total") !== 5) {
    fail("initial computed diamond values were incorrect");
  }
  document.querySelector("button")?.click();
  await waitFor(() => computedValue("total") === 10, "computed diamond update");
  if (computedValue("doubled") !== 4 || computedValue("tripled") !== 6) fail("diamond prerequisites did not refresh");
  if (window.__EDGEZERO__.computed_update_runs !== 1) fail("diamond action did not flush one batch");
  if (window.__EDGEZERO__.computed.some((entry) => entry.dirty)) fail("computed diamond caches remained dirty");
  if (window.__EDGEZERO__.diagnostics.length !== 0) fail("computed diamond reported diagnostics");
  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_COMPUTED_DIAMOND_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<pre>EDGEZERO_COMPUTED_DIAMOND_BROWSER_TEST_FAIL: ${error.message}</pre>`);
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

  if (!Array.isArray(window.__EDGEZERO__.diagnostics) || window.__EDGEZERO__.diagnostics.length !== 0) {
    fail("unexpected diagnostics were present");
  }

  if (window.__EDGEZERO__.runtime_version !== "0.0.0") {
    fail("runtime version was not exposed");
  }

  if (window.__EDGEZERO__.supported_schema_version !== 2) {
    fail("supported schema version was not exposed");
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

fn write_dynamic_attributes_probe_page(out_dir: &Path) {
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
  if (button === null) fail("dynamic attribute button was not found");
  if (button.hasAttribute("disabled")) fail("disabled attribute should be omitted initially");
  if (button.disabled !== false) fail("disabled property should be false initially");
  if (button.getAttribute("title") !== "Ready") fail("title attribute was not initialized");
  if (!document.body.textContent.includes("Status:Ready")) fail("initial status text was not Ready");

  const nodes = window.__EDGEZERO__.manifest.components[0].template.nodes;
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

  const state = window.__EDGEZERO__.components[0].state;
  if (state.disabled !== true || state.label !== "Locked") {
    fail(`debug state did not reflect dynamic attribute update: ${JSON.stringify(state)}`);
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_DYNAMIC_ATTRIBUTES_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_DYNAMIC_ATTRIBUTES_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

  const elements = Array.from(document.body.children)
    .filter((element) => ["H1", "P", "SPAN"].includes(element.tagName));
  const tags = elements.map((element) => element.tagName.toLowerCase());
  if (JSON.stringify(tags) !== JSON.stringify(["h1", "p", "span"])) {
    fail(`fragment siblings did not render in order: ${tags.join(",")}`);
  }

  if (document.querySelector("[data-ez-node='n0'], [data-ez-node='n2']") !== null) {
    fail("fragment IDs should not render as wrapper DOM nodes");
  }

  const h1 = document.querySelector("h1");
  const paragraph = document.querySelector("p");
  const span = document.querySelector("span");
  if (h1?.dataset.ezNode !== "n1") fail("heading node identity was not preserved");
  if (paragraph?.dataset.ezNode !== "n3") fail("paragraph node identity was not preserved");
  if (span?.dataset.ezNode !== "n5") fail("span node identity was not preserved");
  if (!document.body.textContent.includes("TitleStatus:ReadyDone")) {
    fail("fragment text content did not render expected sibling output");
  }

  const nodes = window.__EDGEZERO__.manifest.components[0].template.nodes;
  const nodeIds = nodes.map((node) => node.id);
  if (JSON.stringify(nodeIds) !== JSON.stringify(["n1", "n3", "n4", "n5"])) {
    fail(`fragment manifest should omit fragment nodes: ${nodeIds.join(",")}`);
  }

  if (window.__EDGEZERO__.components[0].state.label !== "Ready") {
    fail("fragment component state did not initialize");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_FRAGMENTS_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_FRAGMENTS_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
  if (button === null) {
    fail("conditional fixture button was not found");
  }

  if (button.textContent.trim() !== "On") {
    fail(`initial conditional branch was not On: ${button.textContent}`);
  }

  if (document.querySelector("[data-ez-node='n4']") === null) {
    fail("true branch node identity was not present initially");
  }

  const manifestNode = window.__EDGEZERO__.manifest.components[0].template.nodes[1];
  if (manifestNode.kind !== "conditional" || manifestNode.start !== "n2" || manifestNode.end !== "n3") {
    fail("conditional manifest node did not expose stable boundaries");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "Off", "false branch");

  if (document.querySelector("[data-ez-node='n4']") !== null) {
    fail("true branch node should have been removed after toggle");
  }

  if (document.querySelector("[data-ez-node='n5']") === null) {
    fail("false branch node identity was not present after toggle");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "On", "true branch");

  if (document.querySelector("[data-ez-node='n4']") === null) {
    fail("true branch node identity was not restored after second toggle");
  }

  if (window.__EDGEZERO__.store.elementsByNode.get("n4") !== document.querySelector("[data-ez-node='n4']")) {
    fail("runtime element index did not refresh after conditional replacement");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_CONDITIONAL_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_CONDITIONAL_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const listLabels = () => [...document.querySelectorAll("ol li")].map((item) => item.textContent);

(async () => {
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  if (buttons.length !== 2) fail("expected list reconciliation controls");

  if (JSON.stringify(listLabels()) !== JSON.stringify(["0:North", "1:South"])) {
    fail(`initial list contents were unexpected: ${listLabels().join(" | ")}`);
  }

  const north = document.querySelector("[data-ez-node='n7:North']");
  const south = document.querySelector("[data-ez-node='n7:South']");
  if (north === null || south === null) fail("initial keyed list nodes were not found");

  const listNode = window.__EDGEZERO__.manifest.components[0].template.nodes.find(
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

  if (document.querySelector("[data-ez-node='n7:North']") !== north) {
    fail("North node was recreated instead of retained during movement");
  }
  if (document.querySelector("[data-ez-node='n7:South']") !== south) {
    fail("South node was recreated instead of retained during movement");
  }
  const east = document.querySelector("[data-ez-node='n7:East']");
  if (east === null) {
    fail("East node was not inserted during reconciliation");
  }
  if (window.__EDGEZERO__.store.elementsByNode.get("n7:North") !== north) {
    fail("runtime element index did not retain the North node");
  }
  if (JSON.stringify(window.__EDGEZERO__.components[0].state.labels) !== JSON.stringify(["South", "East", "North"])) {
    fail("debug state did not retain the reconciled array");
  }

  buttons[1].click();
  await waitFor(
    () => JSON.stringify(listLabels()) === JSON.stringify(["0:East"]),
    "trimmed list"
  );

  if (document.querySelector("[data-ez-node='n7:East']") !== east) {
    fail("East node was recreated instead of retained during deletion");
  }
  if (document.querySelector("[data-ez-node='n7:North']") !== null || document.querySelector("[data-ez-node='n7:South']") !== null) {
    fail("stale keyed list nodes were not removed during deletion");
  }
  if (JSON.stringify(window.__EDGEZERO__.components[0].state.labels) !== JSON.stringify(["East"])) {
    fail("debug state did not retain the trimmed array");
  }
  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_KEYED_LIST_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_KEYED_LIST_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const listLabels = () => [...document.querySelectorAll("ol li")].map((item) => item.textContent);

(async () => {
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const buttons = document.querySelectorAll("button");
  if (buttons.length !== 2) fail("expected object list reconciliation controls");

  if (JSON.stringify(listLabels()) !== JSON.stringify(["0:North(west)", "1:South(east)"])) {
    fail(`initial object list contents were unexpected: ${listLabels().join(" | ")}`);
  }

  const north = document.querySelector("[data-ez-node='n7:north']");
  const south = document.querySelector("[data-ez-node='n7:south']");
  if (north === null || south === null) fail("initial object keyed list nodes were not found");

  const listNode = window.__EDGEZERO__.manifest.components[0].template.nodes.find(
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

  if (document.querySelector("[data-ez-node='n7:north']") !== north) {
    fail("North object node was recreated instead of retained during movement");
  }
  if (document.querySelector("[data-ez-node='n7:south']") !== south) {
    fail("South object node was recreated instead of retained during movement");
  }
  const east = document.querySelector("[data-ez-node='n7:east']");
  if (east === null || east.textContent !== "1:East(central)") {
    fail("East object node did not receive member bindings during insertion");
  }
  if (window.__EDGEZERO__.store.elementsByNode.get("n7:north") !== north) {
    fail("runtime element index did not retain the North object node");
  }
  if (JSON.stringify(window.__EDGEZERO__.components[0].state.items.map((item) => item.id)) !== JSON.stringify(["south", "east", "north"])) {
    fail("debug state did not retain the reconciled object array");
  }

  buttons[1].click();
  await waitFor(
    () => JSON.stringify(listLabels()) === JSON.stringify(["0:East(central)"]),
    "trimmed object list"
  );

  if (document.querySelector("[data-ez-node='n7:east']") !== east) {
    fail("East object node was recreated instead of retained during deletion");
  }
  if (document.querySelector("[data-ez-node='n7:north']") !== null || document.querySelector("[data-ez-node='n7:south']") !== null) {
    fail("stale object keyed list nodes were not removed during deletion");
  }
  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_OBJECT_KEYED_LIST_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_OBJECT_KEYED_LIST_BROWSER_TEST_FAIL: ${error.message}</div>`);
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

const itemButtons = () => [...document.querySelectorAll("ol li button")];

(async () => {
  await waitFor(() => document.documentElement.dataset.ezRuntime === "ready", "runtime ready");

  const refresh = document.querySelector("section > button");
  const selections = document.querySelector("p");
  const north = document.querySelector("[data-ez-node='n8:north']");
  const south = document.querySelector("[data-ez-node='n8:south']");
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

  if (document.querySelector("[data-ez-node='n8:north']") !== north) {
    fail("North root was replaced instead of refreshed in place");
  }
  if (document.querySelector("[data-ez-node='n8:south']") !== south) {
    fail("South root was replaced instead of refreshed in place");
  }
  if (north.title !== "central" || north.dataset.label !== "Northern") {
    fail("retained North attributes did not refresh");
  }
  if (south.title !== "mountain" || south.dataset.label !== "Southern") {
    fail("retained South attributes did not refresh");
  }

  const east = document.querySelector("[data-ez-node='n8:east']");
  if (east === null || east.title !== "coastal" || east.dataset.label !== "East") {
    fail("inserted East attributes were not materialized");
  }

  itemButtons()[1].click();
  await waitFor(() => selections.textContent === "2", "inserted list item event");
  itemButtons()[0].click();
  await waitFor(() => selections.textContent === "3", "retained list item event");

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_DYNAMIC_LIST_ITEM_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_DYNAMIC_LIST_ITEM_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
  if (button === null) {
    fail("logical-and fixture button was not found");
  }

  if (button.textContent.trim() !== "On") {
    fail(`initial logical-and branch was not On: ${button.textContent}`);
  }

  const manifestNode = window.__EDGEZERO__.manifest.components[0].template.nodes[1];
  if (manifestNode.kind !== "conditional" || manifestNode.when_false_html !== "") {
    fail("logical-and manifest node did not expose an empty false branch");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "", "empty false branch");

  if (document.querySelector("[data-ez-node='n4']") !== null) {
    fail("true branch node should have been removed for logical-and false branch");
  }

  button.click();
  await waitFor(() => button.textContent.trim() === "On", "restored true branch");

  if (document.querySelector("[data-ez-node='n4']") === null) {
    fail("true branch node identity was not restored for logical-and true branch");
  }

  if (edgezeroConsoleErrors.some((message) => message.includes("[EdgeZero]"))) {
    fail(`unexpected EdgeZero console error: ${edgezeroConsoleErrors.join(" | ")}`);
  }

  document.body.dataset.browserTest = "pass";
  document.body.insertAdjacentHTML("beforeend", "<div>EDGEZERO_LOGICAL_AND_BROWSER_TEST_PASS</div>");
})().catch((error) => {
  document.body.dataset.browserTest = "fail";
  document.body.insertAdjacentHTML("beforeend", `<div>EDGEZERO_LOGICAL_AND_BROWSER_TEST_FAIL: ${error.message}</div>`);
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
        page.push_str("<script type=\"application/json\" id=\"ez-template-manifest\">\n");
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
    page.push_str("  await waitFor(() => document.documentElement.dataset.ezRuntime === \"error\" && window.__EDGEZERO__, \"runtime error\");\n");
    page.push_str("  if (window.__EDGEZERO__.runtime_version !== \"0.0.0\") fail(\"runtime version was not exposed\");\n");
    page.push_str("  if (window.__EDGEZERO__.supported_schema_version !== 2) fail(\"supported schema version was not exposed\");\n");
    page.push_str("  const diagnostics = window.__EDGEZERO__.diagnostics;\n");
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
    page.push_str("  document.body.insertAdjacentHTML(\"beforeend\", `<div>EDGEZERO_RUNTIME_CONTRACT_DIAGNOSTIC_FAIL: ${error.message}</div>`);\n");
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
