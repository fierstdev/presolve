EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: core: add asm ownership traversal
* Working tree: clean after committing this slice
* Date: 2026-07-11 03:40:29 PDT

Last completed slice

* Slice: C7-A - ASM ownership traversal queries
* Summary: Added deterministic ownership traversal to the canonical application model.
* Key files: crates/ezc_core/src/application_semantic_model.rs
* New behavior: ASM consumers can query application-root semantic IDs with `application_roots()` and direct children with `children_of()`, returned in semantic-ID order.
* Tests added or changed: Core coverage verifies root, direct component-child, and method-action traversal ordering.
* Fixtures added or changed: none.

Current in-progress slice

* Slice: C7-A - ASM ownership traversal queries
* Status: Complete
* Completed: ASM-1 through ASM-8; Era III-A through III-E; Era IV-A through IV-G; Era V-A through V-C; C1-A through C1-B; C2-A through C2-D; C3-A through C3-D; C4-A through C4-B; C5-A through C5-M; C6-A through C6-G; C7-A
* Remaining: C7-B add deterministic transitive ownership traversal for semantic tooling.

Verification

* cargo fmt --all --check: pass
* cargo test -p ezc_parser: pass
* cargo test -p ezc_core: pass
* cargo test -p ezc_cli --test explain: pass
* RUST_TEST_THREADS=1 cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* CI=true RUST_TEST_THREADS=1 cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* cargo clippy --workspace --all-targets -- -D warnings: pass
* RUST_TEST_THREADS=1 cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: pass

Architecture decisions made

* Decision: Template descendants are lowered into canonical semantic entities separate from backend-local `n*` template IDs.
* Reason: Developer tools and compiler analyses need typed, owned, provenance-backed template semantics without taking a dependency on DOM emission details.
* Tradeoff: Template entity paths are deterministic traversal paths, while generated HTML/template-manifest contracts retain their existing local anchor IDs.

* Decision: Only direct `this.<stateField>` template reads resolve to `TemplateState` references in C3-B.
* Reason: The ASM gains reliable dependency edges for supported template behavior without introducing a general expression evaluator or speculative partial references.
* Tradeoff: Member access, computed expressions, and unresolved field names remain absent from the relation graph; keyed-list iterables are resolved by the dedicated C3-C extension.

* Decision: Keyed-list iterable dependencies reuse `TemplateState` with the list semantic entity as their source.
* Reason: A list's iterable is a direct template state read, so it has the same component ownership, provenance, and invalidation semantics as other direct template reads.
* Tradeoff: Item/index scope, keys, item members, and nested list-item expressions remain outside the component-state dependency model.

* Decision: Template event attributes reuse `EventMethod` with the canonical event-attribute entity as their source.
* Reason: The existing render-handler edge remains for backend compatibility, while ASM consumers can now trace an event directly from its authored template entity to the resolved method.
* Tradeoff: Both legacy render-handler and canonical template-event sources point at the same method until the backend-facing graph is migrated.

* Decision: `ezc asm --format json` owns an explicit schema-versioned inspection document rather than serializing compiler structs directly.
* Reason: CLI consumers need a stable, deterministic interface that can evolve independently of Rust data-layout changes.
* Tradeoff: The document exposes generic entity kinds, owners, provenance, relations, and diagnostics, not every compiler-internal field or backend artifact.

* Decision: `ezc asm` accepts explicit source paths and constructs a `CompilationUnit` in compiler path order.
* Reason: Multi-file semantic inspection must share the compiler's application input boundary rather than independently aggregating file-local outputs.
* Tradeoff: The command does not discover project files. Multi-file JSON retains the C4-A primary `file` field and adds an ordered `files` field only when more than one input is supplied.

* Decision: Parser summaries retain type annotations only when a property is recognized as a `state(...)` declaration.
* Reason: Explicit state types are the first type-semantic input needed by the canonical model, without expanding this slice to general TypeScript declaration analysis.
* Tradeoff: Annotation text is captured verbatim apart from outer whitespace and the leading colon; no inference, imports, compatibility checks, or non-state property types are modeled yet.

* Decision: Declared state types are attached to `StateField` as metadata with independent source provenance.
* Reason: The canonical component and ASM models can expose the actual authored type and its source location without treating a type annotation as a separate executable semantic entity.
* Tradeoff: Declared type metadata is descriptive only; it creates no type references, diagnostics, runtime behavior, or compatibility requirements.

* Decision: ASM JSON exposes declared type data only on state entities that actually have a declaration.
* Reason: The optional field extends the stable inspection document without changing the representation of existing untyped programs.
* Tradeoff: The document reports raw declared text and provenance; it does not classify, resolve, or validate the type expression.

* Decision: Primitive declared type classification recognizes only exact `string`, `number`, `boolean`, and `null` text.
* Reason: The compiler gains a reliable first typed vocabulary without silently interpreting unions, aliases, generics, literals, or imported names.
* Tradeoff: Any other valid TypeScript type remains available as raw declared text but has no classification or checking semantics yet.

* Decision: ASM JSON serializes an optional primitive `declared_type.kind` directly from canonical declared-state metadata.
* Reason: Inspection consumers receive a stable, reliable classification field without duplicating TypeScript interpretation in the CLI or inventing an `unknown` category.
* Tradeoff: Unclassified declarations omit `kind`; this remains descriptive metadata and has no validation, runtime, manifest, or backend effect.

* Decision: Primitive initializer validation compares only exact recognized declared types with statically known primitive initializer values.
* Reason: The canonical compiler can provide immediately reliable diagnostics from authored source without implying general TypeScript assignment compatibility or runtime flow analysis.
* Tradeoff: Unclassified declarations, arrays, objects, missing values, action updates, inferred types, aliases, imports, and unions do not produce type diagnostics in this slice.

* Decision: Compiler diagnostics carry optional provenance, and `EZC1016` locates the declared type annotation that establishes the incompatible contract.
* Reason: Developer tools can navigate from a semantic diagnostic to authoritative authored source while legacy diagnostics remain compatible when no reliable location exists.
* Tradeoff: Only primitive initializer mismatches populate diagnostic provenance in this slice; other compiler and ASM validation diagnostics intentionally omit the optional JSON field.

* Decision: Direct literal action assignments use a distinct `EZC1017` diagnostic and the parser's action-expression span.
* Reason: Initializer and action failures have different authored causes, so tooling can explain the operation precisely without collapsing them into one generic type mismatch.
* Tradeoff: Only `this.<field> = <primitive literal>` participates; increments, compound assignments, toggles, composite literals, variable flow, and unclassified types remain unvalidated.

* Decision: Boolean toggle validation uses distinct `EZC1018` diagnostics for exact primitive declared fields that are not `boolean`.
* Reason: The recognized toggle action has a fixed boolean result, so the compiler can validate it without interpreting arbitrary expressions.
* Tradeoff: Only the exact self-toggle form participates; numeric operators, compound assignments, variable flow, and unclassified types remain unvalidated.

* Decision: Increment and decrement validation uses distinct `EZC1019` diagnostics for exact primitive declared fields that are not `number`.
* Reason: The recognized numeric update operations have fixed numeric requirements, so the compiler can validate their targets without evaluating application state.
* Tradeoff: Compound arithmetic operands, arbitrary expressions, variable flow, and unclassified types remain unvalidated.

* Decision: Compound arithmetic uses `EZC1020` for non-number exact primitive targets and `EZC1021` for non-number literal operands.
* Reason: Target and operand failures are independent compiler facts, so separate diagnostics give tools actionable, source-provenanced evidence without expression evaluation.
* Tradeoff: Only serializable literal operands are classified; arbitrary expressions, variable flow, and unclassified declarations remain outside the type system.

* Decision: Text ASM inspection appends deterministic compiler diagnostic details only when compiler diagnostics exist.
* Reason: Command-line users can see the same canonical source evidence as JSON consumers without changing successful zero-diagnostic output.
* Tradeoff: ASM validation diagnostics are still count-only in text output; JSON remains the complete machine-readable inspection interface.

* Decision: Text ASM inspection appends deterministic ASM validation diagnostic details only when validation failures exist.
* Reason: Inspectors can now see every diagnostic class in the text surface while normal source compilation stays compact and compatibility-safe.
* Tradeoff: Standard source-driven `ezc asm` inputs generally have no ASM validation failures; direct formatter coverage exercises this defensive contract without inventing invalid CLI inputs.

* Decision: Canonical ASM/frontend consumers use module-qualified semantic IDs, while the existing backend-facing graph retains legacy component-scoped IDs until its runtime contracts are deliberately migrated.
* Reason: A canonical application model must distinguish semantically equivalent components from different modules, but the established HTML/template runtime protocol does not serialize these IDs and should not be changed implicitly.
* Tradeoff: Two identity entry points coexist temporarily: `build_component_graph_for_module` for canonical compiler products and `build_component_graph` for legacy backend compatibility.

* Decision: Relative re-exports are flattened with a bounded fixed-point pass over the compilation unit.
* Reason: It resolves named and export-all chains through the same `ModuleGraph` and `BindingTable` products without recursive parser work or order-dependent results.
* Tradeoff: External and namespace re-exports remain unresolved, and the current component-scoped semantic IDs still require the next module-qualified identity migration.

* Decision: C2-B resolves only local exports and imports whose module-graph target is a relative file in the current `CompilationUnit`.
* Reason: Compiler bindings must use the same source/module/symbol products as the rest of `ezco`, while package resolution and re-export chains require additional well-defined frontend semantics.
* Tradeoff: External packages remain unbound, and `export { ... } from` / `export * from` chains remain C2-C work.

* Decision: C2-A indexes only declarations local to each source module using class-qualified member names.
* Reason: Local declaration identity must be stable before imports, exports, aliases, and package resolution can form cross-module bindings.
* Tradeoff: Imports and export aliases are intentionally absent from `SymbolTable`; resolving them is the next C2-B slice.

* Decision: Module edges are derived from parsed import and re-export declarations in `CompilationUnit`, not from component provenance.
* Reason: File relationships are frontend semantics that must exist before symbol resolution and must be shared by all ASM consumers.
* Tradeoff: C1-B resolves only relative source files already present in the unit. Package resolution, tsconfig aliases, extension remapping, and symbol bindings remain C2 or later frontend work.

* Decision: The compiler frontend now accepts a deterministic `CompilationUnit` before application-level semantic construction.
* Reason: `ezco` needs a project-wide input boundary so every later graph, analysis, plan, and developer product can consume one semantic model rather than independently reparsing files.
* Tradeoff: C1-A aggregates existing file-local semantics only. Import/export declarations, resolved module edges, duplicate semantic identity diagnostics, and symbols remain later compiler-front-end work.

* Decision: Existing route, module, layout, and resume metadata remain experimental compiler consumers rather than evidence that application-platform semantics are complete.
* Reason: The revised roadmap prioritizes real multi-file frontend and symbol resolution before adding further platform graphs.
* Tradeoff: Era V-D and adjacent application-platform slices are deferred until the canonical ASM has the necessary compiler foundations.

* Decision: Every completed slice updates both this handoff and the active weekly progress log before its checkpoint is finalized.
* Reason: The handoff preserves immediate continuation context while the progress log preserves the durable implementation chronology.
* Tradeoff: Documentation-only recovery commits may be required when a prior checkpoint omitted the weekly entry.

* Decision: `ezc check` defaults parser failures to `error` and keeps that policy command-scoped until a project configuration format is deliberately designed.
* Reason: The compiler can establish a predictable default without implying that an undocumented configuration file is accepted or that compiler/ASM integrity findings are suppressible.
* Tradeoff: Teams must pass a policy threshold in their command invocation; project presets and policy-file discovery remain future work.

* Decision: Browser e2e recipe entry points run with one Rust test thread.
* Reason: Each test launches a real Chrome process, and serial execution prevents host-resource contention and stale profile locks during the documented verification commands.
* Tradeoff: The browser suite takes longer to run, but `pnpm test:e2e` and `just e2e` now produce a reproducible result on constrained development hosts.

* Decision: `ezc check` projects parser label provenance as a deterministic array of source coordinates.
* Reason: CLI and automation consumers can navigate from a parser diagnostic to every parser-provided span without reparsing the source or depending on backend-specific diagnostics.
* Tradeoff: Labels currently provide only positional spans. Label messages, source excerpts, code frames, and compiler/ASM provenance in check JSON remain separate follow-up work.

* Decision: `ezc check --format json` reuses the ASM source-provenance shape for compiler diagnostics and omits it when unavailable.
* Reason: Check consumers receive the same canonical coordinates as ASM inspection without representing missing provenance as an invented location or a misleading null contract.
* Tradeoff: Only diagnostics with reliable compiler provenance include the field. Source remapping, code frames, and provenance for ASM validation diagnostics remain future work.

* Decision: ASM ownership traversal exposes application roots and direct children as semantic IDs ordered by the canonical ownership map.
* Reason: Tooling can navigate the compiler-owned hierarchy without rebuilding ownership from public fields or depending on source declaration order.
* Tradeoff: C7-A is intentionally direct-only; transitive traversal, entity-kind filters, and source-to-semantic lookup remain follow-up query capabilities.

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
* Item: Semantic IDs, direct ownership, and provenance cover components, state fields, methods, action steps, rendered templates, event handlers, and authored template descendants. Backend HTML/template-manifest nodes still use local `n*` IDs as a compatibility contract.
* Item: Resolved references cover action-to-state, event-to-method, and direct text-binding/dynamic-attribute/conditional/keyed-list-iterable-to-state pairs. Routes, member expressions, computed expressions, and unresolved reference attempts have no semantic relation records yet.
* Item: Canonical compiler products now include module-qualified template entities, direct template state dependencies, and direct template event-method dependencies, while `BindingTable` resolves local/relative re-export chains plus named/default/namespace imports. External and namespace re-exports, external package bindings, tsconfig aliases, source remapping, and type semantics are still absent. Legacy backend graph identity remains a compatibility path.
* Item: `ezc asm` accepts explicit source files and exposes generic JSON and text inspection. Text includes compiler and ASM validation diagnostic detail when present. Project discovery, tsconfig resolution, source remapping, typed action payloads, and machine-readable backend plans remain future slices.
* Item: Declared state types include canonical primitive classification, optional ASM JSON `declared_type.kind`, and source-provenanced `EZC1016` through `EZC1021` diagnostics for supported initializer and action forms. Other compiler/ASM diagnostics may omit provenance. Arbitrary action expressions, variable flow, manifests, runtime, imported types, non-state annotations, inference, unions, aliases, generics, and general assignment compatibility remain outside current type validation.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.
* Item: Check policy is selected per CLI invocation. Project policy files, presets, and policy discovery are not interpreted yet.
* Item: Parser diagnostic labels expose only `line`, `column`, `start`, and `end`; parser label messages and rendered source excerpts are not available yet. Compiler provenance in check JSON is optional, and ASM validation diagnostics still have no provenance field.
* Item: ASM ownership queries currently expose only application roots and one direct child level. Transitive walks, filters, reverse source lookup, and CLI query arguments remain future work.

Exact next step

Start C7-B - Add deterministic transitive ownership traversal over the canonical ASM while preserving C7-A direct-query ordering and ownership semantics. Do not add CLI syntax or mutate the semantic model.

Useful commands

* `cargo fmt --all --check`
* `cargo test -p ezc_parser`
* `cargo test -p ezc_core`
* `cargo test -p ezc_cli`
* `RUST_TEST_THREADS=1 cargo test -p ezc_cli --test runtime_browser -- --nocapture`
* `cargo clippy --workspace --all-targets -- -D warnings`
* `pnpm test:e2e`
* `just e2e`
* `RUST_TEST_THREADS=1 cargo test --workspace`
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
* `RUST_TEST_THREADS=1 cargo test -p ezc_cli --test runtime_browser keyed_lists_reconcile_in_a_real_browser -- --nocapture`
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
