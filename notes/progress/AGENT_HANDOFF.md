EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: runtime: reconcile object keyed lists
* Working tree: clean after committing this slice
* Date: 2026-07-10 18:38:30 PDT

Last completed slice

* Slice: 7F-G - Object keyed-list reconciliation
* Summary: Extended browser list reconciliation to derive keys and inserted member bindings from object items using the compiler's dot-member contract.
* Key files: crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs, crates/ezc_cli/tests/explain.rs
* New behavior: The runtime resolves `item.id` keys for initial object roots and later list assignments. New object rows materialize member bindings such as `item.label` and `item.details.region` before insertion, while unchanged member keys retain their DOM roots across moves and deletions.
* Tests added or changed: runtime codegen contract assertions, CLI HTML/manifest golden checks, and a browser probe covering object-key reorder, insertion, deletion, retained identity, and inserted nested-member text.
* Fixtures added or changed: fixtures/0025-object-keyed-list-reconciliation

Current in-progress slice

* Slice: 7F - Lists and keys
* Status: In progress
* Completed: 7F-A - List semantic model; 7F-B - Static initial list rendering; 7F-C - Keyed reconciliation; 7F-D - Key diagnostics; 7F-E - Recursive object serializable values; 7F-F - Member-expression evaluation; 7F-G - Object keyed-list reconciliation
* Remaining: 7F-H - Dynamic list-item behavior.

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
* Decision: Serializable arrays may contain serializable primitives or nested arrays, but static list item binding resolution is limited to the exact item variable and optional index variable.
* Reason: It enables initial list rendering without introducing object-value semantics or arbitrary expression evaluation.
* Tradeoff: Object entries and member expressions such as `item.id` remain semantic metadata only; their initial values are not rendered in 7F-B.
* Decision: A list manifest owns a start/end anchor pair, the iterable state dependency, item/key variables, an item-root template ID, and placeholder HTML for a new root.
* Reason: The runtime can build a key-to-element index from static HTML and reconcile roots without an application-specific virtual DOM.
* Tradeoff: Runtime reconciliation is intentionally constrained to one root element per item and local item/index text substitution; nested dynamic behavior inside list items is not hydrated yet.
* Decision: Before member-expression evaluation, keyed lists were constrained to their direct item variable as the key.
* Reason: The initial runtime could prove direct primitive key identity while object/member access semantics were isolated into a dedicated follow-on slice.
* Tradeoff: That temporary constraint delayed `item.id` support until 7F-F established the compiler evaluator and 7F-G mirrored it in the browser runtime.
* Decision: Serializable object values use ordered maps with static identifier, string, or numeric keys.
* Reason: The compiler needs recursively traversable data that serializes to deterministic JSON objects in manifests without introducing a new dependency or arbitrary expression evaluation.
* Tradeoff: Spreads, computed keys, shorthand references, methods, accessors, and non-literal property values remain unsupported until later language slices define their semantics.
* Decision: Static list member evaluation accepts non-empty dot paths rooted at the list item variable.
* Reason: A small, compiler-owned evaluator can resolve object members for static HTML, member-derived IDs, and diagnostics without becoming a general JavaScript interpreter.
* Tradeoff: The compiler evaluator covers static rendering and diagnostics; 7F-G mirrors it only for runtime keys and insertion-time text, leaving retained item refresh to 7F-H.
* Decision: The browser runtime mirrors the compiler's dot-member semantics for list keys and insertion-time text bindings.
* Reason: Initial compiler IDs, manifest templates, and future object list assignments need one consistent object path contract to retain roots and materialize new rows correctly.
* Tradeoff: Runtime member evaluation is intentionally limited to list keys and text binding comments. It does not yet refresh retained item content, attributes, events, or nested dynamic behavior.

Known limitations

* Item: Conditional rendering only supports simple `this.<stateField>` conditions with JSX element or fragment branches.
* Item: Conditional branch snippets are replaced as static HTML. Bindings, events, and nested dynamic behavior inside swapped-in branch snippets are not re-registered yet.
* Item: Keyed lists currently accept only `iterable.map((item, index?) => <element>...</element>)` with identifier parameters and an expression-bodied callback. Static and runtime reconciliation support a direct primitive item key or a dot-member key such as `item.id` that resolves to a unique primitive.
* Item: Missing keys, index keys, unsupported expressions, duplicate statically-known primitive keys, and missing/non-primitive member keys emit `EZC1011` through `EZC1015`.
* Item: List item templates must have one root element. New runtime instances substitute direct item/index values and dot-member text bindings, but retained item text, attributes, events, and nested dynamic behavior do not refresh yet.
* Item: Duplicate runtime list keys that arise from dynamic state still produce `EZR_DUPLICATE_LIST_KEY` and the later duplicate is skipped. A missing/non-primitive dynamic member key falls back to index identity; compiler diagnostics cover only statically-known initial items.
* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, binding callback text/attribute updates, and conditional branch replacement.
* Item: Static and `this.<stateField>` dynamic JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic attributes are limited to primitive state-field bindings; arbitrary expressions, method calls, spread attributes, arrays, and objects are not emitted yet.
* Item: The serializable value model supports primitives, recursive arrays, and recursive object literals. Static and insertion-time runtime member paths resolve object data, while retained item updates remain deferred to 7F-H.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Source spans are available on parser/render/template structures and CLI development output, but runtime manifests intentionally omit source metadata for now.
* Item: Fragment nodes are visible in compiler/template output but intentionally omitted from runtime manifests until a runtime range-anchor use case appears.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 7F-H - Dynamic list-item behavior. Refresh retained list item bindings after array assignments, then add the narrow runtime behavior needed for dynamic item attributes and delegated events without disturbing keyed root identity.

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
* `cargo run -p ezc_cli -- html fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx`
* `cargo test -p ezc_cli --test runtime_browser keyed_lists_reconcile_in_a_real_browser -- --nocapture`
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
