EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: feat: support boolean state literals
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:13:34 PDT

Last completed slice

* Slice: 6E-B - Boolean literals
* Summary: State initializers now accept boolean literals and preserve boolean type through parser, component graph, template graph, static HTML rendering, manifest JSON, and browser runtime initialization.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/html_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: `state(true)` and `state(false)` render as `true` / `false`, manifest binding `initial_value` fields serialize as JSON booleans, and `window.__EDGEZERO__` preserves strict boolean state values.
* Tests added or changed: parser boolean assertions, core HTML/manifest assertions, CLI html/template/manifest fixture assertions, and browser e2e boolean-state boot probe.
* Fixtures added or changed: fixtures/0007-boolean-state/input/BooleanFlags.tsx plus expected HTML, template, and manifest outputs

Current in-progress slice

* Slice: 6E-C - Null literals
* Status: Not started
* Completed: None
* Remaining: Support `state(null)`, preserve null in parser/compiler/manifest values, define static and runtime text rendering as empty string, and add static plus browser coverage.

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

* Decision: Introduce narrow scalar initial-value enums now instead of waiting until 6E-D.
* Reason: Boolean support must distinguish `true` from `"true"` so manifests can serialize JSON booleans and the browser runtime can preserve boolean state.
* Tradeoff: The enum is still scoped to number/string/boolean and is not yet the final roadmap `SerializableValue` shape.
* Follow-up: Slice 6E-C should add null, and Slice 6E-D should rename/refine this scalar path into the final serializable value model before arrays and objects.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment actions, numeric/string/boolean initial state, and binding callback text updates.
* Item: Null state literals are still unsupported until 6E-C.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.

Exact next step

Start Slice 6E-C by adding a null variant to the parser/core initial-value enums, rendering null bindings as empty text, serializing manifest `initial_value` as JSON null, and adding a null fixture plus browser coverage.

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
