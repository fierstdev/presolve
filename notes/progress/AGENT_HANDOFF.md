EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: support direct assignment actions
* Working tree: clean after committing this slice
* Date: 2026-07-09 21:46:48 PDT

Last completed slice

* Slice: 6F-C - Direct literal assignment
* Summary: Added support for `this.<field> = <literal>` through parser extraction, typed compiler operations, manifest operands, and browser runtime execution.
* Key files: crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: Direct assignment actions serialize as `"assign"` with typed `operand`; runtime writes the operand through the existing delegated event path.
* Tests added or changed: parser assignment extraction, core action/manifest assertions, CLI fixture tests, and a real-browser reset probe.
* Fixtures added or changed: fixtures/0011-direct-assignment

Current in-progress slice

* Slice: 6F-D - Boolean toggle pattern
* Status: Not started
* Completed: None
* Remaining: Recognize `this.<field> = !this.<field>`, serialize a closed boolean-toggle operation, and execute it in the browser runtime.

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

* Decision: Reuse the typed manifest `operand` path for direct assignment and write the operand as-is at runtime.
* Reason: 6F-C should preserve literal values without introducing arbitrary expression evaluation.
* Tradeoff: Numeric assignments currently write the manifest's raw number string because `SerializableValue::Number(String)` serializes as a JSON string.
* Follow-up: Treat any numeric manifest representation change as a later value-model decision, not part of 6F-D.

Known limitations

* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, and `this.<field> = <literal>` are recognized as actions. Boolean toggle and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment/decrement/add-assign/subtract-assign/direct-assignment actions, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 6F-D by adding boolean toggle support for `this.<field> = !this.<field>`, including parser/core/manifest models and browser behavior.

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
* `cargo run -p ezc_cli -- build fixtures/0010-add-subtract-assign/input/StepCounter.tsx --out target/ezc-manual/step-counter`
* `cargo run -p ezc_cli -- build fixtures/0011-direct-assignment/input/ResetCounter.tsx --out target/ezc-manual/reset-counter`

Changed but uncommitted files

* None
