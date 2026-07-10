EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: ci: allow Chrome e2e on GitHub runners
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:26:45 PDT

Last completed slice

* Slice: CI repair after 6E-C
* Summary: GitHub Actions Chrome e2e failed because hosted runners report `No usable sandbox!`; the browser harness now adds CI-only Chrome sandbox flags.
* Key files: crates/ezc_cli/tests/runtime_browser.rs
* New behavior: Chrome browser probes use shared launch argument construction and add `--no-sandbox` plus `--disable-dev-shm-usage` only when `CI` is present.
* Tests added or changed: no new tests; the existing browser suite was run locally with and without `CI=true`.
* Fixtures added or changed: None

Current in-progress slice

* Slice: 6E-D - Typed serializable value model
* Status: Not started
* Completed: None
* Remaining: Rename/refine the temporary initial-state scalar type into the roadmap's compiler-owned `SerializableValue { Null, Boolean(bool), Number(String), String(String) }` model and ensure parser, manifest, HTML, and browser tests continue to assert end-to-end type retention.

Verification

* cargo fmt --all --check: pass
* cargo test -p ezc_parser: pass
* cargo test -p ezc_core: pass
* cargo test -p ezc_cli --test explain: pass
* cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* CI=true cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: not run locally because `just` is not installed in this shell
* Known failures: `cargo clippy --workspace --all-targets -- -D warnings` still fails on pre-existing warnings outside this slice, so the new CI workflow intentionally does not add a clippy gate yet.

Architecture decisions made

* Decision: Add Chrome `--no-sandbox` only under `CI`.
* Reason: GitHub runners need the flag for the installed Chrome binary, while local browser tests should keep stricter default sandbox behavior where available.
* Tradeoff: The GitHub hosted environment still needs a push to confirm the repair end to end.
* Follow-up: If hosted CI still fails, inspect the next logs before changing runtime behavior; the compiler/browser probes passed locally.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment actions, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still scalar-only until later array/object slices.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 6E-D by renaming/refining `ParsedStateInitialValue` and `StateInitialValue` toward a compiler-owned `SerializableValue` model, preserving the current JSON semantics for null/boolean/number/string fixtures and keeping the browser gate green.

Useful commands

* `cargo fmt --all --check`
* `cargo test -p ezc_parser`
* `cargo test -p ezc_core`
* `cargo test -p ezc_cli`
* `cargo test -p ezc_cli --test runtime_browser -- --nocapture`
* `pnpm test:e2e`
* `cargo test --workspace`
* `cargo run -p ezc_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/ezc-manual/double-binding-counter`

Changed but uncommitted files

* None
