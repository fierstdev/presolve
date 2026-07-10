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
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];

    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

#[test]
#[ignore = "requires a local headless Chrome browser"]
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

    let output = run_chrome_with_timeout(
        chrome,
        &[
            "--headless=new",
            "--disable-gpu",
            "--no-first-run",
            "--disable-background-networking",
            "--virtual-time-budget=5000",
            "--dump-dom",
            &user_data_dir,
            &probe_url,
        ],
        Duration::from_secs(5),
    );

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

fn write_probe_page(out_dir: &Path) {
    let index = fs::read_to_string(out_dir.join("index.html")).expect("failed to read built page");
    let probe = index.replace(
        "</body>",
        r#"<script>
const fail = (message) => {
  throw new Error(message);
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
  if (button === null) fail("button not found");
  if (!document.body.textContent.includes("Count:0")) fail("initial count was not 0");
  if (!document.body.textContent.includes("Mirror:0")) fail("initial mirror was not 0");

  button.click();
  await waitFor(() => document.body.textContent.includes("Count:1"), "count 1");
  if (!document.body.textContent.includes("Mirror:1")) fail("mirror was not 1");

  button.click();
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
