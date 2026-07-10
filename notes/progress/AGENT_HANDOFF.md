EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: preserve static jsx attributes
* Working tree: clean after committing this slice
* Date: 2026-07-10 06:54:57 PDT

Last completed slice

* Slice: 7A - Preserve static JSX attributes
* Summary: Replaced JSX attribute string summaries with structured parser/render attributes and preserved ordinary static/boolean attributes in template graph and static HTML output.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/html_codegen.rs
* New behavior: String literal attributes and boolean presence attributes emit in source order before compiler-owned `data-ez-*` attributes; expression/spread/complex attribute values remain unsupported with diagnostics.
* Tests added or changed: parser structured attribute assertions, core static HTML/template assertions, duplicate/expression/spread diagnostics, and CLI fixture checks.
* Fixtures added or changed: fixtures/0014-static-attributes

Current in-progress slice

* Slice: 7B - Dynamic attribute bindings
* Status: Not started
* Completed: None
* Remaining: Support expression-backed attributes such as `disabled={this.disabled}` and `title={this.label}`, add manifest binding target metadata, and define runtime update behavior by attribute category.

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

* Decision: 7A only emits string literal and boolean presence attributes as static HTML attributes.
* Reason: Expression-backed attributes need binding target metadata and runtime update rules, which belong to 7B.
* Tradeoff: Expression, spread, and complex JSX attribute values are parsed structurally but currently diagnosed as unsupported and omitted from static output.
* Follow-up: 7B should reuse `RenderAttributeValue::Expression` as the entry point for dynamic attribute bindings.

Known limitations

* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: Static JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic expression attributes and spread attributes are not emitted yet.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 7B by turning supported `RenderAttributeValue::Expression(Some("this.<field>"))` attributes into attribute binding records and runtime update plans.

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
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
