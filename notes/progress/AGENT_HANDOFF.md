EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: chore: clear verification gates
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:42:36 PDT

Last completed slice

* Slice: Verification cleanup after 6E-D
* Summary: Cleared the stale local verification exceptions for `just e2e` and clippy with warnings-as-errors.
* Key files: crates/ezc_cli/src/main.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_parser_spike/src/main.rs, justfile
* New behavior: `cargo clippy --workspace --all-targets -- -D warnings` passes, and `just e2e` was run directly through the repository recipe.
* Tests added or changed: no fixture behavior changed; helper refactors and clippy annotations only.
* Fixtures added or changed: None

Current in-progress slice

* Slice: 6F-A - Decrement mutation operation
* Status: Not started
* Completed: None
* Remaining: Recognize `this.<field>--`, add `StateOperation::Decrement`, serialize the operation through the manifest, and execute it in the browser runtime.

Verification

* cargo fmt --all --check: pass
* cargo test -p ezc_parser: pass
* cargo test -p ezc_core: pass
* cargo test -p ezc_cli --test explain: pass
* cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* CI=true cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* cargo clippy --workspace --all-targets -- -D warnings: pass
* cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: pass

Architecture decisions made

* Decision: Keep browser e2e gates sequential when running multiple commands locally.
* Reason: The browser tests use deterministic Chrome profile directories under `target/ezc-browser-test`, and parallel browser-suite invocations can contend on Chrome `SingletonLock` files.
* Tradeoff: Sequential verification is a little slower but avoids false failures unrelated to compiler/runtime behavior.
* Follow-up: If parallel browser-suite execution becomes necessary, make the profile root unique per process.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment actions, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 6F-A by adding decrement support for `this.<field>--`: parser extraction, component action conversion, manifest serialization, runtime execution, fixture coverage, and a browser assertion.

Useful commands

* `cargo fmt --all --check`
* `cargo test -p ezc_parser`
* `cargo test -p ezc_core`
* `cargo test -p ezc_cli`
* `cargo test -p ezc_cli --test runtime_browser -- --nocapture`
* `cargo clippy --workspace --all-targets -- -D warnings`
* `pnpm test:e2e`
* `just e2e`
* `cargo test --workspace`
* `cargo run -p ezc_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/ezc-manual/double-binding-counter`

Changed but uncommitted files

* None
