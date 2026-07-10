EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: support boolean toggle actions
* Working tree: clean after committing this slice
* Date: 2026-07-10 06:02:53 PDT

Last completed slice

* Slice: 6F-D - Boolean toggle pattern
* Summary: Added support for the explicit `this.<field> = !this.<field>` boolean toggle pattern through parser extraction, closed compiler operations, manifest serialization, and browser runtime execution.
* Key files: crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: Boolean toggle actions serialize as `"toggle"` with no operand; runtime requires a boolean field value and writes its negation through the existing delegated event path.
* Tests added or changed: parser toggle extraction, core action/manifest assertions, CLI fixture tests, and a real-browser toggle probe.
* Fixtures added or changed: fixtures/0012-boolean-toggle

Current in-progress slice

* Slice: 6F-E - Multi-step action plans
* Status: Not started
* Completed: None
* Remaining: Allow one method to contain multiple supported state updates in source order, serialize ordered action plans, and execute each step in the browser runtime.

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

* Decision: Model boolean toggle as a closed `StateOperation::Toggle` with no manifest operand.
* Reason: 6F-D only admits the explicit same-field `this.<field> = !this.<field>` pattern and should not generalize arbitrary unary expressions.
* Tradeoff: Runtime rejects non-boolean current values with `EZR_NON_BOOLEAN_FIELD`.
* Follow-up: Multi-step methods in 6F-E should preserve source order across all existing closed operations.

Known limitations

* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as actions. Multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, one operation per method, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 6F-E by allowing one method to contain multiple supported state updates in source order, including parser/core/manifest model changes and browser behavior.

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
* `cargo run -p ezc_cli -- build fixtures/0012-boolean-toggle/input/ToggleFlag.tsx --out target/ezc-manual/toggle-flag`

Changed but uncommitted files

* None
