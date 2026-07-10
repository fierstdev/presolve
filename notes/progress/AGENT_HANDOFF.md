EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: support decrement actions
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:50:08 PDT

Last completed slice

* Slice: 6F-A - Decrement mutation operation
* Summary: Added support for `this.<field>--` through parser extraction, component graph actions, manifest serialization, and browser runtime execution.
* Key files: crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: Decrement actions serialize as `"decrement"` and execute by subtracting 1 from numeric state through the existing delegated event path.
* Tests added or changed: parser decrement extraction, core action/manifest assertions, CLI fixture tests, and a real-browser decrement probe.
* Fixtures added or changed: fixtures/0009-decrement-counter

Current in-progress slice

* Slice: 6F-B - Add-assign and subtract-assign literal operations
* Status: Not started
* Completed: None
* Remaining: Recognize `this.<field> += <literal>` and `this.<field> -= <literal>`, preserve the operand as a typed literal, serialize it through the manifest, and execute it in the browser runtime.

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

* Decision: Model decrement as a closed `StateOperation::Decrement` sibling of increment, not as a generalized arithmetic expression.
* Reason: Slice 6F-A only admits `this.<field>--`; broader arithmetic with operands belongs to 6F-B.
* Tradeoff: Runtime numeric coercion is still operation-specific and intentionally narrow.
* Follow-up: Use `SerializableValue` for 6F-B operands rather than adding ad hoc operand strings.

Known limitations

* Item: Only `this.<field>++` and `this.<field>--` are recognized as actions. `this.count += 1`, assignment, boolean toggle, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment/decrement actions, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 6F-B by adding add-assign/subtract-assign literal support for `this.<field> += <literal>` and `this.<field> -= <literal>`, including typed operands in parser/core/manifest models and browser behavior.

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
* `cargo run -p ezc_cli -- build fixtures/0009-decrement-counter/input/DecrementCounter.tsx --out target/ezc-manual/decrement-counter`

Changed but uncommitted files

* None
