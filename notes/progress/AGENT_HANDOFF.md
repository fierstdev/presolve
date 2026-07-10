EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: feat: delegate runtime events by template node id
* Working tree: clean after committing this slice
* Date: 2026-07-09 18:50:33 PDT

Last completed slice

* Slice: 6D-D - Switch to delegated events
* Summary: Event semantics now carry explicit event types, manifest events include `event`, and runtime dispatch uses one delegated document listener per event type.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/runtime_codegen.rs
* New behavior: JSX `onClick` is represented as event type `click`; browser clicks on nested targets walk up `data-ez-node` ancestors and dispatch through manifest event/action indexes without per-node listeners.
* Tests added or changed: parser event assertions, component graph event/diagnostic assertions, runtime string assertions, CLI fixture expectations, ignored Chrome browser probe for nested target plus two button handlers
* Fixtures added or changed: event schema updates in fixtures/0001-source-summary, fixtures/0003-semantic-errors, fixtures/0004-nested-jsx, and expanded fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx

Current in-progress slice

* Slice: 6D-E - Add the first browser e2e suite as a permanent gate
* Status: Not started
* Completed: None
* Remaining: Promote browser behavior testing from the ignored Rust integration probe into a reproducible local and CI e2e gate with deterministic build setup, static serving, scripts/just target, and unexpected runtime-console failure handling.

Verification

* cargo fmt --check: pass
* cargo test --workspace: pass
* Other tests: `cargo test -p ezc_core`, `cargo test -p ezc_cli`, and `cargo test -p ezc_cli --test runtime_browser -- --ignored --nocapture` passed before the full workspace gate.
* Known failures: None

Architecture decisions made

* Decision: Keep event type as a compiler-owned string at this stage and reject unsupported events with diagnostics.
* Reason: The roadmap only requires `click` now, but the manifest/runtime boundary needs the explicit event field before delegation and future event growth.
* Tradeoff: The model is still narrow and string-based rather than a broader event enum or schema-versioned contract.
* Follow-up: Runtime contract versioning in Slice 6G should stabilize event schema expectations.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment actions, numeric state, and binding callback text updates.
* Item: Browser e2e coverage is present but ignored by default until the dedicated e2e gate slice.

Exact next step

Start Slice 6D-E by turning the existing ignored `crates/ezc_cli/tests/runtime_browser.rs` behavior into a permanent browser e2e gate. Decide whether to keep the Rust harness as the official gate or introduce the roadmap-preferred Playwright package, then add the local command/just target and CI path without weakening the current Chrome probe assertions.

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
