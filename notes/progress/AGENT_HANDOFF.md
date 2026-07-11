EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: compiler: add resume diagnostics
* Working tree: clean after committing this slice
* Date: 2026-07-10 19:49:12 PDT

Last completed slice

* Slice: Era IV-D - Resume diagnostics
* Summary: Added validation for incomplete resume instance coverage.
* Key files: crates/ezc_core/src/resume_diagnostics.rs, crates/ezc_core/src/lib.rs
* New behavior: Resume validation reports missing component instances and missing planned state slots.
* Tests added or changed: core suite and clippy pass.
* Fixtures added or changed: None.

Current in-progress slice

* Slice: Era IV-D - Resume diagnostics
* Status: Complete
* Completed: ASM-1 through ASM-8; Era III-A through III-E; Era IV-A through IV-D
* Remaining: None

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
* Decision: Scoped list bindings emit a compiler-owned end anchor after each binding comment, plus element-local metadata for dynamic item attributes.
* Reason: A retained item can refresh text without swallowing adjacent static punctuation, and attribute updates can reuse the runtime's existing attribute semantics without adding a client-side expression evaluator.
* Tradeoff: The metadata accepts only the direct item/index variables and non-empty dot-member paths; arbitrary expressions remain unsupported.
* Decision: List-item click actions are discovered from retained or inserted DOM roots and registered in the delegated event map; removed roots explicitly unregister their nodes.
* Reason: Keyed root identity remains stable while events work for both compiler-rendered and runtime-inserted rows.
* Tradeoff: Only the existing delegated click/action contract is hydrated. Conditional branch replacement inside a list item still does not register new dynamic behavior.
* Decision: Semantic IDs are typed compiler values with readable component-scoped paths, such as `component:x-counter/state:count`.
* Reason: Existing template IDs are local backend anchors. The ASM needs stable identities that survive unrelated declaration ordering and can be shared by future semantic consumers.
* Tradeoff: Component identity uses `@component(...)` when available and falls back to the class name for invalid components. Duplicate identity validation and source provenance are deferred to later ASM slices.
* Decision: Actions use their source order within the owning method as their final semantic-ID segment.
* Reason: The parser does not yet retain per-action source spans, but ordered action steps are already a compiler contract.
* Tradeoff: Inserting an earlier action step changes later action IDs; source-provenance-backed refinement remains deferred to ASM-4.
* Decision: Semantic IDs do not change template manifests, static HTML, or runtime artifacts in ASM-1.
* Reason: This establishes a compiler-platform contract without forcing a backend schema change before ownership and cross-reference semantics exist.
* Tradeoff: Semantic IDs are inspectable through compiler APIs only until the planned `ez asm` CLI slice.
* Decision: Ownership is a typed relationship with either the application root or a direct owning `SemanticId`.
* Reason: The compiler can distinguish top-level component roots from semantic children without reserving a synthetic application ID before the Application Semantic Model exists.
* Tradeoff: Ownership is currently stored on existing graph entities rather than in a centralized ASM relation table; the query API and ASM shell will consolidate access later.
* Decision: Component state, methods, and rendered templates are component-owned; action steps are method-owned.
* Reason: These are the direct lexical/semantic containment relationships already established by the current frontend and action model.
* Tradeoff: Render-tree descendants do not yet have semantic IDs, so their ownership remains deferred until a later ASM slice extends semantic identity below the template root.
* Decision: Cross references are directed resolved edges stored on `ComponentGraph`, with a kind, source semantic ID, and target semantic ID.
* Reason: Existing graph consumers can access relationships without an ASM shell, while later ASM/query slices can migrate this stable relation shape intact.
* Tradeoff: There is no reverse index or centralized relation table yet; query-oriented traversal remains deferred to ASM-6.
* Decision: Event handlers use deterministic component-scoped IDs in render traversal order and are owned by the rendered template.
* Reason: Event-to-method references need a distinct semantic source even though general render descendants still lack semantic identity.
* Tradeoff: Inserting an earlier event changes later event IDs; source-provenance-backed refinement remains deferred to ASM-4.
* Decision: Only fully resolved action/state and event/method pairs create semantic references.
* Reason: Reference consumers can rely on every target ID existing in the graph, while existing diagnostics remain the source of unresolved-reference feedback.
* Tradeoff: Unresolved attempts are not retained as partial relation records until diagnostics and provenance gain their planned ASM models.
* Decision: Source provenance is stored once in a `SemanticId`-keyed registry on `ComponentGraph`.
* Reason: Every current semantic consumer shares one authoritative path/span record instead of duplicating source fields across graph structures.
* Tradeoff: The registry is not yet a cross-file application index; the Application Semantic Model shell will own aggregation later.
* Decision: State update spans originate in the parser from update/assignment expressions.
* Reason: Action semantic provenance must identify the actual operation rather than approximating it from the enclosing method.
* Tradeoff: The span excludes the expression statement semicolon, matching the AST operation span used by the compiler.
* Decision: Resolved references carry the provenance of their source entity.
* Reason: Tooling and diagnostics can trace an edge to the authored action or event handler without requiring a reverse lookup first.
* Tradeoff: Target-side provenance remains available through the semantic provenance registry; relations intentionally store only their own origin.
* Decision: `ApplicationSemanticModel` assembles copies of the existing graph outputs instead of replacing backend-facing graph builders immediately.
* Reason: Existing HTML, manifest, and runtime paths keep their stable contracts while all future compiler consumers gain one application-level semantic entry point.
* Tradeoff: Graph data is temporarily duplicated at assembly time; later backend migration and query slices can eliminate redundant construction when the API is mature.
* Decision: ASM ownership is centralized as a `SemanticId` to `SemanticOwner` map while entity-local ownership fields remain available.
* Reason: The model provides the first global ownership view without forcing all existing consumers through a new lookup API in this slice.
* Tradeoff: Both representations coexist until the query API and backend migration establish a single consumption path.

Known limitations

* Item: Conditional rendering only supports simple `this.<stateField>` conditions with JSX element or fragment branches.
* Item: Conditional branch snippets are replaced as static HTML. Bindings, events, and nested dynamic behavior inside swapped-in branch snippets are not re-registered yet.
* Item: Keyed lists currently accept only `iterable.map((item, index?) => <element>...</element>)` with identifier parameters and an expression-bodied callback. Static and runtime reconciliation support a direct primitive item key or a dot-member key such as `item.id` that resolves to a unique primitive.
* Item: Missing keys, index keys, unsupported expressions, duplicate statically-known primitive keys, and missing/non-primitive member keys emit `EZC1011` through `EZC1015`.
* Item: List item templates must have one root element. Retained and inserted items refresh direct item/index/member text and direct item/index/member attributes, and hydrate delegated click actions. Nested dynamic behavior still does not refresh or re-register.
* Item: Duplicate runtime list keys that arise from dynamic state still produce `EZR_DUPLICATE_LIST_KEY` and the later duplicate is skipped. A missing/non-primitive dynamic member key falls back to index identity; compiler diagnostics cover only statically-known initial items.
* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, binding callback text/attribute updates, and conditional branch replacement.
* Item: Static and `this.<stateField>` dynamic JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic attributes are limited to primitive state-field bindings; arbitrary expressions, method calls, spread attributes, arrays, and objects are not emitted yet.
* Item: The serializable value model supports primitives, recursive arrays, and recursive object literals. Static and runtime list paths resolve direct item/index values and non-empty item-member paths; arbitrary JavaScript expressions remain unsupported.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Source spans are available on parser/render/template structures and CLI development output, but runtime manifests intentionally omit source metadata for now.
* Item: Fragment nodes are visible in compiler/template output but intentionally omitted from runtime manifests until a runtime range-anchor use case appears.
* Item: Semantic IDs and direct ownership currently cover components, state fields, methods, action steps, rendered templates, and event handlers. General template descendants still use backend-local `n*` IDs and have no semantic ownership or provenance entries yet.
* Item: Resolved references currently cover action-to-state and event-to-method pairs only. Bindings, attributes, conditionals, lists, routes, and unresolved reference attempts have no semantic relation records or provenance-backed relations yet.
* Item: `ApplicationSemanticModel` currently assembles one parsed file's existing compiler outputs. Multi-file aggregation, source remapping, generated-source provenance, query APIs, validation, and CLI inspection remain deferred to ASM-6 through ASM-8.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Era IV-E - Zero-replay browser boot planning. Connect resume artifacts to the runtime boot contract.

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
