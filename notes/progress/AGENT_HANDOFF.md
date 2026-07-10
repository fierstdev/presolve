EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: represent keyed list semantics
* Working tree: clean after committing this slice
* Date: 2026-07-10 12:39:21 PDT

Last completed slice

* Slice: 7F-A - List semantic model
* Summary: Added compiler-owned keyed-list nodes that preserve the iterable dependency, item variable, optional index variable, key expression, source span, and item template.
* Key files: crates/ezc_parser/src/model.rs, crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/component_graph.rs, crates/ezc_core/src/template_graph.rs, crates/ezc_core/src/html_codegen.rs, crates/ezc_core/src/template_manifest.rs
* New behavior: Expression-bodied JSX maps such as `{this.items.map((item, index) => <li key={item.id}>{index}: {item.label}</li>)}` compile into first-class list nodes. The list iterable is recorded as a parent binding dependency, and `key` is treated as structural list metadata rather than an unsupported ordinary JSX attribute.
* Tests added or changed: parser list assertions, core parser-to-template assertions including intentionally empty static list output, and CLI parse/graph/template golden checks.
* Fixtures added or changed: fixtures/0019-keyed-list-semantics

Current in-progress slice

* Slice: 7F - Lists and keys
* Status: In progress
* Completed: 7F-A - List semantic model
* Remaining: Start 7F-B static initial list rendering after extending `SerializableValue` with arrays.

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

* Decision: Conditional nodes are first-class parser/render/template children with a conditional node ID plus separate start/end boundary IDs.
* Reason: The compiler needs stable branch identity for tooling and runtime updates, while the DOM needs comment anchors that can bound branch replacement without a wrapper element.
* Tradeoff: Runtime manifests serialize branch HTML snippets for this first slice instead of recursively hydrating dynamic bindings/events inside branch snippets.
* Decision: Logical-and shorthand reuses the same conditional model with an empty false branch.
* Reason: It keeps ternary and shorthand update behavior identical once the branch anchor path is stable.
* Decision: Keyed list nodes are represented before arrays, static list HTML, manifest serialization, or runtime reconciliation.
* Reason: The compiler needs a stable semantic contract for list identity before committing to array values or DOM update behavior.
* Tradeoff: List item templates are visible in parser/component/template tooling, but they intentionally emit no initial DOM content or runtime manifest records yet.

Known limitations

* Item: Conditional rendering only supports simple `this.<stateField>` conditions with JSX element or fragment branches.
* Item: Conditional branch snippets are replaced as static HTML. Bindings, events, and nested dynamic behavior inside swapped-in branch snippets are not re-registered yet.
* Item: Keyed lists currently accept only `iterable.map((item, index?) => <element key={expression}>...</element>)` with identifier parameters and an expression-bodied callback. Diagnostics for missing or unstable keys are deferred to 7F-D.
* Item: List nodes have no static array rendering, runtime manifest representation, or reconciliation behavior until `SerializableValue` supports arrays and later 7F slices add keyed updates.
* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, binding callback text/attribute updates, and conditional branch replacement.
* Item: Static and `this.<stateField>` dynamic JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic attributes are limited to primitive state-field bindings; arbitrary expressions, method calls, spread attributes, arrays, and objects are not emitted yet.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Source spans are available on parser/render/template structures and CLI development output, but runtime manifests intentionally omit source metadata for now.
* Item: Fragment nodes are visible in compiler/template output but intentionally omitted from runtime manifests until a runtime range-anchor use case appears.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 7F-B by extending `ParsedSerializableValue` and `SerializableValue` with constrained serializable arrays, then render the initial keyed-list contents from a list node without adding runtime reconciliation yet.

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
* `cargo run -p ezc_cli -- build fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx --out target/ezc-manual/conditional-rendering`
* `cargo run -p ezc_cli -- build fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx --out target/ezc-manual/logical-and-conditional`
* `cargo run -p ezc_cli -- template fixtures/0019-keyed-list-semantics/input/KeyedList.tsx`
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
