EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: refactor: add explicit browser state store
* Working tree: clean after committing this slice
* Date: 2026-07-09 18:42:07 PDT

Last completed slice

* Slice: 6D-C - Introduce an explicit runtime state store
* Summary: Runtime mutation now flows through a store with component, binding, action, and element maps plus `readField`, `writeField`, and `notifyField` helpers.
* Key files: crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/explain.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: Two bindings to the same state field update together after clicks, and `window.__EDGEZERO__.store` exposes live runtime maps for debugging.
* Tests added or changed: crates/ezc_core/src/runtime_codegen.rs runtime string assertions, crates/ezc_cli/tests/explain.rs build artifact assertions, crates/ezc_cli/tests/runtime_browser.rs double-binding real-browser probe
* Fixtures added or changed: fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx

Current in-progress slice

* Slice: 6D-D - Switch to delegated events
* Status: Not started
* Completed: None
* Remaining: Extend manifest events with explicit event type, preserve JSX event names as event types, update template graph/manifest schema, install one document-level listener per event type, dispatch through template node IDs, and add nested-target/two-button event tests.

Verification

* cargo fmt --check: pass
* cargo test --workspace: pass
* Other tests: `cargo test -p ezc_core`, `cargo test -p ezc_cli`, and `cargo test -p ezc_cli --test runtime_browser -- --ignored --nocapture` passed before the full workspace gate.
* Known failures: None

Architecture decisions made

* Decision: Expose the internal runtime store under `window.__EDGEZERO__.store` while also preserving the existing `window.__EDGEZERO__.components` debug view.
* Reason: Slice 6D-C requires inspectable store maps, and the existing components array is already used by the browser probe as a stable debug convenience.
* Tradeoff: The debug surface is richer than the minimum runtime needs, but it remains generated runtime state rather than source-derived behavior.
* Follow-up: Runtime contract versioning and diagnostics in Slice 6G should decide which debug fields are durable.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only per-node click listeners, increment actions, numeric state, and binding callback text updates.
* Item: Browser e2e coverage is present but ignored by default until the dedicated e2e gate slice.

Exact next step

Start Slice 6D-D by adding event type semantics end to end. Begin in parser/template semantics by preserving `onClick` as `click`, then emit manifest events with `{ node, event, handler }` and update the runtime to install one delegated document-level click listener that resolves the nearest `data-ez-node`.

Useful commands

* `cargo fmt --check`
* `cargo test -p ezc_parser`
* `cargo test -p ezc_core`
* `cargo test -p ezc_cli`
* `cargo test -p ezc_cli --test runtime_browser -- --ignored --nocapture`
* `cargo test --workspace`
* `cargo run -p ezc_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/ezc-manual/double-binding-counter`

Changed but uncommitted files

* None
