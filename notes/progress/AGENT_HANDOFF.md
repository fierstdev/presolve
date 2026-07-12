EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest completed slice: D6-F - invalidation graph
* Working tree: clean after the D2-G6 commit
* Date: 2026-07-11

Last completed slice

* Slice: D6-F - invalidation graph
* Summary: Added ordered compiler-owned optimization pass execution.
* Key files: crates/ezc_core/src/intermediate_representation.rs
* New behavior: `analyze_reachability` partitions canonical blocks into entry-reachable and unreachable sets.
* Fixtures added or changed: none.

Current in-progress slice

* Slice: D3-D - reachability
* Status: Complete
* Completed: Phase C1 through C35; Phase D1-A through D3-D
* Remaining: D6-G - dependency queries.

Verification

* cargo fmt --all --check: pass
* cargo clippy --workspace --all-targets -- -D warnings: pass
* cargo test -p ezc_core: pass

Architecture decisions made

* Decision: State initializer expressions have stable graph roots and recursively keyed canonical nodes in the ASM.
* Reason: Folding and inspection now consume the same compiler-owned topology instead of independently traversing field-local lowering structures.
* Tradeoff: B10 retains legacy field-local trees only as lowering compatibility data; every semantic consumer added in this phase reads the canonical graph. General expression owners remain later work.

* Decision: Every lowered IR function begins with one stable, empty entry basic block whose identity is derived from the canonical method ID.
* Reason: Subsequent CFG slices can name branch and loop edges against compiler-owned nodes without backend-specific or source-offset-generated block identities.
* Tradeoff: D2-A creates only entry regions; it does not lower statements, define terminators, or create normal/branch/loop edges.

* Decision: Conditional branch arms are represented as source-provenanced directed edges owned by the enclosing IR function.
* Reason: Later condition lowering and CFG analyses can use structural true/false connectivity without coupling to a backend or recovering topology from source control-flow syntax.
* Tradeoff: D2-B models edge shape only. No source branches are lowered, conditions are not represented as operands, and unconditional/loop edges remain later work.

* Decision: Natural loops are explicit function-owned regions with stable loop IDs and canonical block topology.
* Reason: Dominator, post-dominator, liveness, and scheduling consumers can reason about loop boundaries without recognizing source syntax or inferring loops from backend artifacts.
* Tradeoff: D2-C stores loop structure only. It does not derive regions from CFG edges, validate natural-loop invariants, lower source loops, or assign loop-specific instructions.

* Decision: Dominators are derived immutably from IR block and branch-edge topology rather than stored as mutable IR state.
* Reason: CFG analysis remains repeatable and backend-independent, and later optimization passes can consume deterministic analysis output without changing canonical lowering artifacts.
* Tradeoff: D2-D considers only declared conditional branch edges and entry reachability. It does not validate malformed CFGs, include loop-back/unconditional edges, or expose higher-level dominance query helpers yet.

* Decision: Post-dominators are derived immutably in reverse from blocks without declared branch successors.
* Reason: Later code motion, control-dependence, and cleanup passes gain a deterministic reverse-flow relation without requiring backend code generation or mutating canonical IR.
* Tradeoff: D2-E treats blocks without declared branch successors as exits. It does not model explicit terminators, non-terminating control flow, loop-back/unconditional edges, or post-dominator query helpers yet.

* Decision: CFG connectivity and dominance are exposed through read-only function and analysis-tree queries.
* Reason: Future data-flow and optimization consumers can navigate compiler-owned control flow without indexing public vectors directly or rebuilding predecessor/successor relationships.
* Tradeoff: D2-F exposes only current branch-edge topology. Loop-region membership, edge filtering, explicit terminators, reachability, and data-flow queries remain later work.

* Decision: Authored semantic entities, lowered instructions, transient values, and storage slots use separate typed identity domains.
* Reason: A single semantic entity may lower to several operations and values, while optimization may rewrite IR artifacts without changing authored meaning or provenance.
* Tradeoff: D2-G1 defines stable identity only. Existing instructions and state initialization retain their prior representations until later D2-G slices migrate them.

* Decision: Executable operands form a closed enum of value, inline primitive constant, and storage references.
* Reason: Data-flow can distinguish value uses from storage access while retaining constants inline, without an ambiguous catch-all semantic operand.
* Tradeoff: D2-G2 excludes function, template, aggregate, and runtime-allocated operands until concrete lowering needs them.

* Decision: An instruction identity, optional produced value, and optional authored semantic origin are independent fields on canonical IR instructions.
* Reason: Value-producing operations can now participate in data flow without conflating an operation instance, its result, and the semantic entity from which lowering originated.
* Tradeoff: D2-G3 adds operation shapes but does not lower load/store/arithmetic instructions from source, and result values are not yet registered or validated.

* Decision: Transient values are owned by the IR function in a deterministic registry and identify their defining instruction, parameter, or future block parameter explicitly.
* Reason: Definition/use, liveness, and optimization can inspect one canonical value model without inferring definitions from operation shape or conflating values with semantic entities.
* Tradeoff: D2-G4 creates empty registries during current lowering. Parameter/block-parameter lowering and consistency validation are deferred.

* Decision: State fields lower to `IrStorage` slots separate from both semantic identity and transient values.
* Reason: Storage reads, writes, promotion, resumability, and reactive partitioning can evolve without treating an authored field as a runtime slot or computed value.
* Tradeoff: D2-G5 only lowers storage declarations and initialization references; load/store source lowering and storage validation remain deferred.

* Decision: IR integrity is validated as an immutable compiler-owned query over canonical IR, before data-flow consumers run.
* Reason: Definition/use and optimization passes can reject malformed IDs, dangling operands, missing result records, and invalid storage references instead of silently producing misleading analysis.
* Tradeoff: D2-G6 validates current structural contracts only; it does not yet validate source lowering coverage, type operation compatibility, terminators, or loop invariants.

* Decision: Expression graph nodes own `SourceProvenance` rather than an unqualified source span.
* Reason: Tooling and diagnostics need the canonical expression node itself to provide a path-aware authored location without reconstructing it from a state field.
* Tradeoff: B11 derives provenance only for the existing state-initializer graph. Template, action, and general JavaScript expressions are not lowered yet.

* Decision: Expression graph queries return compiler-owned references in stable semantic-ID order and expose direct graph edges only.
* Reason: Language tooling and future optimizations can navigate one canonical graph without reparsing expression trees or inferring ownership from identifier strings.
* Tradeoff: B12 provides no graph mutation, transitive dependency closure, query language, or non-state expression support.

* Decision: `SemanticType` is a compiler-owned algebra with an initially empty ASM assignment model, separate from legacy raw declared-type metadata.
* Reason: Later inference, alias resolution, assignability, and tooling can share one canonical semantic representation without prematurely treating TypeScript text as type semantics.
* Tradeoff: C1 lowers no annotations, infers no values or expressions, and adds no type diagnostics, identities, provenance, normalization, or query APIs.

* Decision: Canonical type assignments identify the typed subject and separately retain the semantic origin, declared-or-inferred status, and source provenance.
* Reason: Aliases, imported types, and inferred expressions will need stable attribution without collapsing their type meaning into raw TypeScript text.
* Tradeoff: C2 defines metadata only; no assignment is populated from source, and identity does not yet model named aliases or cross-module declarations.

* Decision: C3 lowers only exact primitive and literal state annotation text into canonical semantic types.
* Reason: The compiler gains durable primitive and literal semantics while later slices add structured parsing for arrays, tuples, objects, unions, aliases, and imports.
* Tradeoff: Unsupported annotation text produces no canonical assignment or diagnostic yet; C3 does not infer from initial values or alter existing compatibility diagnostics.

* Decision: C4 represents tuples directly and lowers unresolved array element names as `unknown` rather than raw TypeScript names.
* Reason: Collection topology is canonical now, while alias and import slices can later replace unknown element meaning without revising the array contract.
* Tradeoff: Tuple parsing is limited to comma-separated current annotation forms; object elements, nested delimiter-aware parsing, and named-type resolution remain later work.

* Decision: C5 lowers semicolon-separated structural object properties into deterministic maps.
* Reason: Object shape is canonical before template member resolution and list-item typing need it.
* Tradeoff: Object-property declarations have no individual identity or provenance yet; nested delimiter-aware syntax and member-access validation remain later work.

* Decision: C6 preserves top-level union member order without normalizing or deduplicating.
* Reason: The compiler can represent authored unions now while C28 later defines the one canonical normalization policy.
* Tradeoff: Literal parsing inside union members is supported, but aliases, imports, and diagnostics remain later work.

* Decision: C7 gives each local type alias a module-qualified semantic ID and uses that ID as the origin of resolved state assignments.
* Reason: Alias source identity remains available to tooling while consumers receive the alias's canonical semantic type meaning.
* Tradeoff: C7 resolves supported local aliases only; nested alias dependencies, exports, imports, generic aliases, and cycles remain later work.

* Decision: C8 indexes type aliases as module symbols and reuses named relative import/re-export bindings for unit-level type resolution.
* Reason: Imported types now share deterministic module identity and re-export behavior with the compiler's existing frontend infrastructure.
* Tradeoff: C8 excludes external packages, namespace imports, default type imports, generic aliases, and imported alias cycles.

* Decision: C9 widens direct serializable state literals to primitive semantic types and infers empty arrays as `Array<unknown>`.
* Reason: State APIs receive useful stable type contracts without prematurely encoding literal-value constraints or assuming an empty collection element type.
* Tradeoff: C9 infers only direct serializable values; constant expression nodes, actions, and non-serializable values remain later work.

* Decision: C10 uses a canonical state-initializer assignability relation and preserves `EZC1016` for incompatibilities.
* Reason: Arrays, tuples, objects, unions, and nullability now receive one compiler semantic compatibility check instead of primitive-only special cases.
* Tradeoff: C10 is limited to state initializers; C29 will establish the final general assignability engine for all compiler consumers.

* Decision: Expression graph nodes receive their inferred type as ordinary canonical ASM assignments keyed by the existing expression semantic ID.
* Reason: All consumers can query the same owned, provenanced expression topology and type information without re-evaluating authored syntax.
* Tradeoff: C11 propagates types only for the current constant state-initializer expression language. C12 still defines operand validity, while state reads, locals, templates, actions, and arbitrary JavaScript expressions remain later work.

* Decision: Operator typing is one compiler-owned relation over semantic operands and returns an explicit result or invalidity.
* Reason: Expression propagation, later diagnostics, and future optimizations can share defined EdgeZero semantics rather than inheriting JavaScript coercion behavior.
* Tradeoff: C12 covers only the current constant-expression operators. Invalid operations become `unknown` without new diagnostics until C32; strings, truthiness, calls, and non-state expression forms remain unsupported.

* Decision: Existing method-local declaration entities receive inferred type assignments directly from their lowered serializable values.
* Reason: Local bindings, template references, and tooling now meet at one canonical typed entity without inventing a second local-variable model.
* Tradeoff: C13 covers the currently lowered serializable `const` forms. Local expressions, state reads, annotations, flow, mutation, destructuring, and action/local references require later language lowering.

* Decision: Method parameters are first-class method-owned semantic entities with declaration annotations lowered through the canonical type model.
* Reason: Action, computed, IDE, and type-query consumers can address a stable parameter entity rather than interpret method metadata ad hoc.
* Tradeoff: C14 supports currently retained identifier parameters and supported annotations only. Defaults, destructuring, rest, optionality semantics, parameter value flow, and call-site validation remain later work.

* Decision: Method semantic type assignments represent their return contract, declared when annotated and inferred from currently supported serializable returns otherwise.
* Reason: Derived values, loaders, actions, and tooling can query one canonical method result contract before their dedicated phases add richer producers.
* Tradeoff: C15 considers only top-level serializable return expressions. JSX, state/local expressions, async promises, branches, throws, implicit returns, and return compatibility diagnostics remain later work.

* Decision: Direct action assignment compatibility is evaluated by the immutable ASM folding pass using canonical type assignments.
* Reason: State initialization and mutation now share the same compiler-owned compatibility relation rather than duplicating primitive frontend checks.
* Tradeoff: C16 covers only currently lowered direct literal assignments. Compound mutation, arbitrary expressions, parameter/local values, and final general assignability remain later work.

* Decision: Compound mutation typing uses canonical boolean/number compatibility in the same immutable ASM pass as direct assignment validation.
* Reason: Toggle and arithmetic mutation contracts now share compiler-owned types rather than frontend primitive classifiers.
* Tradeoff: C17 covers only currently lowered literal operands and mutation forms; arbitrary expressions, coercion, and generalized operation typing remain later work.

* Decision: Direct template text-binding entities inherit the canonical type of their resolved state/local target and are validated during immutable ASM folding.
* Reason: Rendering, diagnostics, and later tooling can query one typed template entity rather than infer renderability from expression strings.
* Tradeoff: C18 covers direct text bindings only. Attribute/property contracts, list scope, member access, arbitrary expressions, and non-direct template references remain later work.

* Decision: Supported dynamic DOM bindings use compiler-owned contracts that distinguish HTML attributes from DOM properties.
* Reason: Template validation and later IDE/schema work share deterministic type expectations without inheriting browser coercions.
* Tradeoff: C19 covers only `disabled`, `href`, and `value` direct bindings. Element-specific contracts, event payloads, spreads, styles, and arbitrary expressions remain later work.

* Decision: Template conditions are boolean-only compiler semantics and carry the resolved condition type on their canonical entity.
* Reason: Conditional output is predictable across backends and tooling does not need to reproduce JavaScript truthiness.
* Tradeoff: C20 covers direct resolved conditions only. Composite expressions, list scope, member access, and custom condition coercions remain later work.

* Decision: Template list entities carry canonical iterable types and a dedicated item/index scope type record.
* Reason: List-body member access, rendering, and tooling can consume stable scope semantics without re-inferring callback variables from source.
* Tradeoff: C21 supports direct state-backed array/tuple lists only. Member access validation, arbitrary iterables, callback expressions, and list control flow remain later work.

* Decision: Supported list-item member paths resolve through canonical object types and retain successful or failed access records in the type model.
* Reason: Templates and tooling can query member result types or deterministic failures without rescanning expression strings.
* Tradeoff: C22 currently resolves uniquely named list-item roots and dot-member object paths only. State/local member access, unions, optionality, indexes, methods, and arbitrary expressions remain later work.

* Decision: A computed value is a decorator-marked getter with a canonical result record that reuses the method's return type assignment.
* Reason: Computed consumers can query a durable typed contract before the full dependency/runtime computed phase exists.
* Tradeoff: C23 establishes metadata and result typing only. Dependency tracking, getter evaluation, template computed reads, caching, async computed values, and runtime behavior remain later work.

* Decision: An action signature is a decorator-marked method whose existing typed parameters and return contract are assembled into one canonical action record.
* Reason: Forms, server actions, and tooling can query input/output contracts before action transport/runtime semantics are introduced.
* Tradeoff: C24 establishes signature metadata only. Promise/generic resolution, input validation, invocation, transport, server boundaries, and runtime action behavior remain later work.

* Decision: Resources are represented directly in the semantic type algebra with explicit data, error, pending, serializability, and execution-boundary metadata.
* Reason: Later resources, resumability, and backend planning can share one durable contract rather than wrap untyped runtime values.
* Tradeoff: C25 provides only type representation. Resource declaration lowering, loading/execution, error propagation, serialization enforcement, and boundary validation remain later work.

* Decision: Serialization compatibility is one recursive canonical type query, with unknown/never treated as incompatible until proven otherwise.
* Reason: Resumability and backend planning can make deterministic decisions from semantic types instead of runtime value guesses.
* Tradeoff: C26 defines compatibility only; no source declarations are rejected and no boundary-specific diagnostics or generation behavior is introduced yet.

* Decision: Cross-boundary compatibility is a canonical query layered on serialization compatibility, with resource execution boundaries enforced explicitly.
* Reason: Backend planning and resumability can reject impossible crossings before code generation without inspecting runtime values.
* Tradeoff: C27 provides query semantics only. Source boundary annotations, diagnostics, backend enforcement, and resource declaration lowering remain later work.

* Decision: Semantic types normalize before ASM consumers observe assignments, aliases, scopes, accesses, computed values, and action signatures.
* Reason: Equality, caching, inspection, and later assignability operate on one deterministic representation rather than authored union order or nesting.
* Tradeoff: C28 defines canonical representation only. General assignability remains C29, and source aliases retain their separate identities/provenance.

* Decision: One normalized `is_assignable` engine owns semantic compatibility, while the C10 state-initializer API remains a forwarding compatibility surface.
* Reason: Diagnostics, templates, actions, and future consumers no longer scatter independent type relations.
* Tradeoff: C29 centralizes existing supported forms only. Generic variance, functions, conditional types, and language-level subtyping remain outside the current model.

* Decision: Type knowledge is exposed through read-only ASM queries rather than backend/parser-specific lookup paths.
* Reason: IDE, language services, inspection, and later optimization can consume the same canonical type facts.
* Tradeoff: C30 exposes direct queries only. CLI inspection output, richer type-declaration navigation, and composite predicates remain later work.

* Decision: Type inspection uses compiler-owned stable type text and attaches assignment provenance, status, and origin to ASM entities.
* Reason: Tooling can explain canonical type facts without decoding Rust debug output or re-deriving inference attribution from parser metadata.
* Tradeoff: C31 exposes assignment-backed entity types only. It does not add a standalone type-declaration browser, alias navigation UI, or source-summary type inference outside entity-scoped ASM inspection.

* Decision: Type diagnostics use an exported compiler-owned code/family vocabulary, while existing detailed codes remain stable.
* Reason: Diagnostics, IDE tooling, and future type consumers can categorize semantic failures without relying on message text or scattered string literals.
* Tradeoff: C32 reports unresolved declared state types now. Non-serializable-state has a reserved stable family but awaits source-level resource/boundary declarations before it can be emitted meaningfully.

* Decision: Type-system integrity is enforced by the existing ASM validator using deterministic `EZASM1101` through `EZASM1106` diagnostics.
* Reason: Type consumers fail early on corrupted canonical identities, ownership, provenance, or alias origins instead of relying on unchecked map contents.
* Tradeoff: Semantic types are value-owned, so recursive type cycles are unrepresentable; unresolved aliases are reported at lowering as `EZC1032`, while the validator checks the resulting model's origins and identities.

* Decision: Browser runtime integration tests acquire one process-wide lock before creating a Chrome probe, and the probe deadline allows 20 seconds for a cold Chrome start.
* Reason: Cargo's workspace runner schedules tests concurrently, and the previous five-second harness deadline could kill an otherwise healthy cold-start probe with SIGKILL.
* Tradeoff: The browser test binary runs serially under both workspace and dedicated e2e commands; independent non-browser tests remain parallel, while an actually hung probe still has a bounded deadline.

* Decision: Constant evaluation is an idempotent immutable transformation from raw ASM to a newly constructed folded ASM, rather than a side effect of parser or component-graph lowering.
* Reason: Compiler services and backend products now share one canonical evaluated result while retaining authored expression trees and preserving a read-only input model for optimization.
* Tradeoff: B9 folds only existing supported state initializer expressions and refreshes their template values. General expression graph nodes, local-expression folding, action expressions, runtime evaluation, and type propagation remain later work.

* Decision: Method locals receive stable method-child semantic IDs and resolve only from normal `render()` template scope through `template-local` references.
* Reason: Tooling and later optimizations need canonical declaration and use edges, while lexical resolution must not guess across list callback scopes, duplicate declarations, member access, calls, or closures.
* Tradeoff: Exact, uniquely declared identifiers are the only resolved form. Their known serializable values may seed static output, but runtime binding evaluation and all broader JavaScript scope behavior remain intentionally absent.

* Decision: Constrained method parameters are method-owned canonical metadata rather than runtime slots or standalone semantic entities.
* Reason: Parameter declarations need deterministic ownership and provenance for compiler services now, while B7 deliberately establishes no execution, closure, or binding-resolution behavior.
* Tradeoff: Only direct identifier declarations are retained in authored order; destructuring, defaults, rest parameters, captured values, render bindings, action values, and type semantics remain absent until an explicit later slice.

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

* Decision: Transitive ASM ownership traversal uses depth-first pre-order.
* Reason: Consumers encounter each semantic parent before its complete owned subtree while every sibling order remains the canonical semantic-ID order.
* Tradeoff: The query returns IDs only and does not encode depth, paths, filters, or source lookup results.

* Decision: ASM entity filtering uses a typed `SemanticEntityKind` enum and ownership-map ordering.
* Reason: Tooling can request stable semantic categories without stringly typed kinds or independent scans of graph-specific collections.
* Tradeoff: C7-C filters only the broad canonical entity categories; template subkinds, composite predicates, and source-location predicates remain follow-up work.

* Decision: ASM source lookup uses exact provenance paths and half-open byte spans.
* Reason: The compiler preserves source coordinates in this form, so tooling can map source selections to all overlapping semantic entities without inventing line/column conversions or boundary ambiguity.
* Tradeoff: Path normalization, line/column inputs, source remapping, nearest-entity ranking, and editor-range protocols remain future work.

* Decision: ASM reference-kind queries order results by source and target semantic IDs.
* Reason: Tooling receives a stable relation filter independent of construction order.
* Tradeoff: C7-E filters a single reference kind only; endpoint, provenance, and composite relation queries remain follow-up work.

* Decision: ASM reference provenance lookup reuses exact paths and half-open byte spans, with source/target ordering.
* Reason: Tooling can map a source selection to every canonical relation without reparsing source or relying on relation construction order.
* Tradeoff: C7-F does not provide line/column, path normalization, source remapping, composite predicates, or CLI query syntax.

* Decision: Entity inspection is an `asm --entity` extension, not a change to the legacy `explain` command.
* Reason: It exposes the canonical semantic model through an existing inspection surface without breaking the established source-summary explain contract.
* Tradeoff: Users must first obtain a semantic ID from full ASM output; source-position selection and migration of `explain` remain follow-up work.

* Decision: Source-selected inspection chooses the uniquely narrowest overlapping semantic span.
* Reason: Source selection is useful for nested semantic entities while remaining deterministic and refusing equal-specificity ambiguity rather than silently guessing.
* Tradeoff: The CLI accepts exact compiler paths and byte offsets only; line/column input, path normalization, source remapping, and user-selectable candidate lists remain future work.

* Decision: Entity inspection filters use typed broad entity and reference kinds, and only operate after a semantic entity has been selected.
* Reason: The CLI can reuse canonical ASM categories while making the result boundary unambiguous: direct child lists and incoming/outgoing relation lists are filtered without changing the selected entity or its ownership traversal.
* Tradeoff: C8-C accepts one child-kind and one reference-kind filter. Composite predicates, descendant filtering, diagnostics filtering, line/column selection, and path normalization remain future work.

* Decision: `ezc explain` delegates entity inspection to the same canonical ASM inspection runner as `ezc asm`.
* Reason: The developer-facing source-summary command can expose compiler semantics without duplicating selection, ordering, filtering, diagnostic, or schema behavior.
* Tradeoff: Plain `explain` remains a legacy source-summary surface; only explicit entity-selection or entity-filter options activate ASM inspection.

* Decision: Parent navigation is a canonical ASM query and reports the ancestor chain nearest-first through the application root.
* Reason: Semantic tooling can traverse ownership outward from any entity without reconstructing containment from entity-local fields or reversing the ownership map.
* Tradeoff: Parent navigation reports semantic IDs only. It does not add path metadata, transitive child filtering, reference endpoint predicates, or model mutation.

* Decision: Semantic graph export uses a roots collection plus parent-to-child ownership edges instead of a synthetic application node.
* Reason: The export preserves the ASM's distinction between real semantic entities and application ownership while still providing complete deterministic graph topology.
* Tradeoff: The graph is a JSON artifact for canonical roots, typed nodes, provenance, ownership, and resolved references only; diagnostics, parser facts, backend-local node IDs, manifests, runtime artifacts, and graph mutation are outside this slice.

* Decision: Canonical ASM ownership is structurally derived for component-level semantics and consumed through the centralized ownership map.
* Reason: Compiler analyses and semantic lowering should not silently depend on duplicate owner fields that can drift from the canonical application model.
* Tradeoff: Legacy ComponentGraph and template-entity lowering records still retain owner fields for compatibility and initial template containment ingestion; symbol-table and backend paths remain outside this ownership migration.

* Decision: Compiler analyses implement `ImmutableAsmPass`, while `AnalysisPass::analyze` remains a compatibility wrapper.
* Reason: New compiler work receives one explicit read-only ASM transformation boundary without forcing a breaking migration on existing analysis consumers.
* Tradeoff: Current passes produce immutable analysis products rather than rewritten ASM values. Semantic rewrite passes will use the same contract when a compiler-owned language transformation requires them.

* Decision: Constant numeric state initializer arithmetic is a compiler-owned expression model that evaluates during canonical lowering.
* Reason: Authored numeric semantics remain inspectable in the compiler while established HTML, manifest, and runtime paths receive an already-computed serializable initial value.
* Tradeoff: B1 accepts only numeric literals, parentheses, and `+`, `-`, `*`, `/`, or `%` inside `state(...)`. State reads, local variables, calls, coercions, action expressions, comparisons, and expression typing remain later language slices.

* Decision: Arithmetic and comparison initializers share one canonical `ConstantExpression` slot on a state field.
* Reason: The compiler can extend expression semantics without parallel per-operator metadata, while preserving one inspectable authored expression and one evaluated initial value for every backend.
* Tradeoff: B2 comparisons are numeric-only and static: operands are numeric literals or B1 arithmetic, and supported operators are `===`, `!==`, `<`, `<=`, `>`, and `>=`. String/boolean comparisons, coercion, state reads, calls, local variables, logical operators, and action expressions remain unsupported.

* Decision: Constant logical expressions use explicit boolean operands and compiler-time short-circuit evaluation.
* Reason: The compiler can statically preserve `&&`/`||` reachability and avoid emitting runtime expression intelligence or diagnostics from unreachable branches.
* Tradeoff: B3 accepts only boolean literals, B2 comparisons, and nested logical expressions. Unary negation, truthiness, coercion, state reads, local variables, calls, nullish coalescing, and action expressions remain unsupported.

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
* Item: ASM query APIs expose nearest-first parent traversal through the application root, direct and transitive ownership traversal, broad entity kinds, entity/reference provenance lookup, and reference-kind filtering. `asm` and explicit `explain` inspection mode support semantic-ID or source-byte selection plus parent, direct-child, and incoming/outgoing reference navigation, with one typed child and relation filter; composite predicates, descendant/diagnostic filtering, line/column input, path normalization, and source remapping remain future work.
* Item: `ezc asm --format graph` exports a schema-versioned canonical semantic graph with roots, typed nodes, provenance, ownership edges, and resolved reference edges. It intentionally does not discover project files, include diagnostics, expose parser/backend/runtime artifacts, or provide graph filtering, mutation, or alternate serialization formats.
* Item: Canonical ASM ownership now drives template entity lookup, template dependency lowering, and dead-action analysis. Legacy ComponentGraph, TemplateSemanticEntity construction, and SymbolTable records still carry owner fields as compatibility/lowering data; their removal or migration requires a later dedicated frontend/backend compatibility slice.
* Item: Constant `state(...)` initializers use one compiler-owned expression model. Numeric arithmetic, comparisons, boolean logic, nullish coalescing, and unary `!`, `+`, and `-` evaluate statically. State reads, local variables, calls, coercions, truthiness, control flow, and semantic expression typing remain later Phase B work.
* Item: Method parameters are compiler-owned identifier declarations with canonical source provenance only. They do not execute, close over values, resolve local/template/action references, or support destructuring, defaults, rest declarations, or semantic type checking.
* Item: Method-local resolution accepts only exact, uniquely declared supported locals from `render()` template scope. List-item scopes, duplicate local names, member access, arbitrary expressions, calls, closures, action references, runtime updates, and semantic typing remain unresolved.
* Item: Constant folding handles only the existing supported state initializer expression language. It does not fold local-variable values, evaluate actions or templates generically, perform flow/type analysis, or introduce runtime evaluation.
* Item: The canonical expression graph currently covers supported state initializer expressions only. It supports deterministic direct topology, ownership, and provenance queries, while graph mutation, transitive query operators, general expression owners, and non-state expressions remain later work.
* Item: C30 exposes direct ASM type queries only. CLI inspection output, source diagnostics, backend enforcement, resource declaration lowering, and final type diagnostic families remain later Phase C work.
* Item: Canonical IR functions currently contain only empty entry basic blocks plus structural branch-edge and natural-loop records. Dominator and post-dominator results include only declared conditional edges; there are no source-lowered branches or loops, condition operands, explicit terminators, or statement instructions yet.
* Item: Source lowering still creates empty function value registries and no method load/store instructions. D3-A can analyze manually constructed canonical IR now; subsequent lowering slices must populate values before source data-flow results become non-empty.

Exact next step

Continue automatically with D3-E - constant propagation, committing the completed D3-D slice first.

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

* None after the D2-G6 commit.
