EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: support jsx fragments
* Working tree: clean after committing this slice
* Date: 2026-07-10 11:38:36 PDT

Last completed slice

* Slice: 7D - JSX fragments
* Summary: Added parser/render/template fragment nodes, fragment-root support, wrapperless static HTML generation, and transparent manifest traversal.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/html_codegen.rs, crates/ezc_core/src/template_manifest.rs
* New behavior: Fragment roots and nested fragments preserve compiler IDs/spans in development output while emitting only their sibling children in HTML and omitting fragment nodes from runtime manifests.
* Tests added or changed: parser fragment assertions, core wrapperless HTML/transparent manifest assertions, CLI html/template/manifest fixture checks, and a real-browser fragment sibling identity probe.
* Fixtures added or changed: fixtures/0016-fragments

Current in-progress slice

* Slice: 7E - Conditional rendering
* Status: Not started
* Completed: None
* Remaining: Support one narrow conditional rendering form first, preserving branch template identity and defining runtime update behavior.

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

* Decision: 7D models fragments explicitly in parser/render/template graphs but keeps them transparent in static HTML and runtime manifest output.
* Reason: Explicit fragment nodes preserve source spans and compiler identity for tooling while avoiding artificial wrapper DOM elements.
* Tradeoff: Fragment IDs currently exist only in compiler/template output; runtime manifests contain the fragment children and their element/binding IDs.
* Follow-up: 7E should preserve this fragment handling when conditional branches contain fragment or sibling children.

Known limitations

* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: Static and `this.<stateField>` dynamic JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic attributes are limited to primitive state-field bindings; arbitrary expressions, method calls, spread attributes, arrays, and objects are not emitted yet.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Source spans are available on parser/render/template structures and CLI development output, but runtime manifests intentionally omit source metadata for now.
* Item: Fragment nodes are visible in compiler/template output but intentionally omitted from runtime manifests until a runtime range-anchor use case appears.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 7E by parsing a single supported conditional expression shape and deciding how branch nodes appear in template graph, static HTML, and runtime update plans.

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
* `cargo run -p ezc_cli -- build fixtures/0016-fragments/input/FragmentPanel.tsx --out target/ezc-manual/fragments`
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
