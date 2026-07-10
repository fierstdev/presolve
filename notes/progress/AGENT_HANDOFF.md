EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: stabilize serializable value model
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:31:32 PDT

Last completed slice

* Slice: 6E-D - Typed serializable value model
* Summary: Renamed/refined the temporary initial-state scalar types into the roadmap's serializable value model.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_cli/src/main.rs
* New behavior: Parser state initializers now expose `ParsedSerializableValue`, and core/template/manifest/HTML/CLI paths use `SerializableValue { Null, Boolean(bool), Number(String), String(String) }`.
* Tests added or changed: existing parser, core, CLI fixture, and browser tests now assert the stabilized type names without fixture output churn.
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
* cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: not run locally because `just` is not installed in this shell
* Known failures: `cargo clippy --workspace --all-targets -- -D warnings` still fails on pre-existing warnings outside this slice, so the new CI workflow intentionally does not add a clippy gate yet.

Architecture decisions made

* Decision: Keep the parser type named `ParsedSerializableValue` and the compiler-owned type named `SerializableValue`.
* Reason: The parser still reports parsed source facts, while core owns the compiler/runtime value contract that flows into templates, manifests, HTML, and browser initialization.
* Tradeoff: Number literals still serialize with the existing raw-string representation until a later value-model decision changes numeric manifest semantics.
* Follow-up: Use `SerializableValue` for typed operands introduced by later mutation slices instead of adding operation-specific scalar types.

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
* `pnpm test:e2e`
* `cargo test --workspace`
* `cargo run -p ezc_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/ezc-manual/double-binding-counter`

Changed but uncommitted files

* None
