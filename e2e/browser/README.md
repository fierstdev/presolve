# Browser e2e

The permanent browser gate is the Rust integration harness in:

```text
crates/presolve_cli/tests/runtime_browser.rs
```

It builds the double-binding counter fixture, serves the generated output with a tiny in-test static server, opens the page in real Chrome, and fails on unexpected generated-runtime console errors.

Run it directly:

```sh
cargo test -p presolve-cli --features browser-tests --test runtime_browser -- --nocapture
```

Or through the project aliases:

```sh
pnpm test:e2e
just e2e
```

`pnpm test:e2e:headed` and `just e2e-headed` currently run the same deterministic
Chrome dump-DOM harness.

Set `PRESOLVE_CHROME=/path/to/chrome` when Chrome is not in a standard location.
