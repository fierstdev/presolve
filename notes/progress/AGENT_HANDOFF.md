EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: feat: support null state literals
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:22:20 PDT

Last completed slice

* Slice: 6E-C - Null literals
* Summary: State initializers now accept null literals and preserve null through parser, component graph, template graph, static HTML rendering, manifest JSON, and browser runtime initialization.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_core/src/lib.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: `state(null)` renders as empty text, manifest binding `initial_value` fields serialize as JSON null, runtime binding formatting treats null as empty text, and `window.__EDGEZERO__` preserves strict null state values.
* Tests added or changed: parser null assertion, core HTML/manifest assertions, CLI html/template/manifest fixture assertions, runtime-codegen assertion, and browser e2e null-state boot probe.
* Fixtures added or changed: fixtures/0008-null-state/input/NullSelection.tsx plus expected HTML, template, and manifest outputs

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
* cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: not run locally because `just` is not installed in this shell
* Known failures: `cargo clippy --workspace --all-targets -- -D warnings` still fails on pre-existing warnings outside this slice, so the new CI workflow intentionally does not add a clippy gate yet.

Architecture decisions made

* Decision: Treat null rendering as empty text in both static HTML and runtime binding updates.
* Reason: This matches the 6E-C roadmap policy and avoids rendering the JavaScript string `"null"` into user-visible text.
* Tradeoff: The scalar type is still named around initial state rather than the final general serializable value contract.
* Follow-up: Slice 6E-D should stabilize this path as `SerializableValue` before arrays and objects.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment actions, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still scalar-only until later array/object slices.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.

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
