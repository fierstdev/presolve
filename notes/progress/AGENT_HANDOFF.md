EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: support dynamic attribute bindings
* Working tree: clean after committing this slice
* Date: 2026-07-10 07:10:59 PDT

Last completed slice

* Slice: 7B - Dynamic attribute bindings
* Summary: Promoted supported `this.<stateField>` JSX expression attributes into template binding nodes with initial values, manifest target metadata, and runtime attribute/property update callbacks.
* Key files: crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/html_codegen.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: Attributes such as `disabled={this.disabled}` and `title={this.label}` emit attribute binding records, hydrate from component state, and update the existing DOM element when state changes.
* Tests added or changed: core template/manifest assertions, CLI fixture checks, runtime smoke string checks, and a real-browser probe that verifies boolean/property and string attribute updates.
* Fixtures added or changed: fixtures/0015-dynamic-attributes

Current in-progress slice

* Slice: 7C - Source spans on template nodes and edges
* Status: Not started
* Completed: None
* Remaining: Carry source spans through template nodes and binding/event edges so later diagnostics and tooling can point back to exact JSX source locations.

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

* Decision: 7B supports only dynamic attribute expressions of the form `this.<stateField>`.
* Reason: Those expressions map directly to the existing primitive state store, static initial rendering path, and binding callback runtime.
* Tradeoff: Other expression forms and spread attributes remain diagnostics instead of introducing partial JavaScript expression evaluation.
* Follow-up: 7C should add source spans before broadening diagnostics/tooling around attribute binding edges.

Known limitations

* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: Static and `this.<stateField>` dynamic JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic attributes are limited to primitive state-field bindings; arbitrary expressions, method calls, spread attributes, arrays, and objects are not emitted yet.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 7C by threading parser/source span information into template nodes and binding/event metadata without changing runtime behavior.

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
* `cargo run -p ezc_cli -- build fixtures/0013-multi-step-action/input/BatchActionCounter.tsx --out target/ezc-manual/batch-action-counter`
* `cargo run -p ezc_cli -- build fixtures/0014-static-attributes/input/StaticAttributePanel.tsx --out target/ezc-manual/static-attributes`
* `cargo run -p ezc_cli -- build fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx --out target/ezc-manual/dynamic-attributes`
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
