# V2 implementation tracker

This tracker translates the 45 pull requests in the V2 specification into
repository-owned work. A `new` path is an intended ownership location, not an
assertion that it exists today. Each implementation pull request must refine
this mapping with exact APIs and fixture names before code is merged.

## Required evidence for every tracked pull request

Every row must record, in its implementation notes and tests:

1. scope and repository trace;
2. changed semantic products and their schema versions;
3. focused tests and precise unsupported-semantics diagnostics;
4. tooling and migration effects;
5. cold behavior, resumed behavior, and generated-artifact consequences; and
6. a benchmark when the product is performance-sensitive.

This is a delivery checklist, not permission to combine rows. A row that
changes public artifacts must update the matching fixture and golden assertions
in the same change.

## Foundation and platform

| PR | Scope | Repository trace | Products and proof |
| ---: | --- | --- | --- |
| 1 | Architecture map and characterization snapshots | `docs/v2`, workspace manifests, `docs/architecture.md`, all crate/package entry points | This snapshot and tracker; `pnpm run docs:check`. |
| 2 | TypeScript compatibility corpus | `tests/typescript-compatibility`; `scripts/test-typescript-compatibility.mjs`; `tests/framework-public-api` | Implemented against pinned TypeScript 7.0.2: aliases, exports, package imports, TSX, class/access, async flow, project references/source maps, and native diagnostic codes. |
| 3 | Semantic authority adapter | `packages/typescript-authority`; TypeScript native async API; compatibility corpus | Implemented schema v1 semantic query boundary for symbols, aliases, types, contextual types, signatures, assignability, diagnostics, and resolved modules. |
| 4 | Resolved module and symbol identity | `packages/typescript-authority`; later consumers include `binding_table`, `module_graph`, `semantic_reference`, `semantic_id` | Implemented resolved declaration-module identities, alias targets, and module-specifier resolutions with alias/re-export proof. |
| 5 | General source AST | `presolve_parser::{oxc_adapter,model}`; OXC ESTree serialization | Implemented complete TypeScript/TSX ESTree JSON with source text, spans, and existing recovery diagnostics; derived parser facts remain intact. |
| 6 | Canonical intrinsic registry | `packages/typescript-authority`; `framework/packages/presolve` declarations | Resolved-target registry and use-site classification; spelling-independent framework export proof. |
| 7 | Canonical authored semantic model | `application_semantic_model`, `component_graph`, `semantic_graph` | One normalized authored-model schema; deterministic model snapshots. |
| 8 | Legacy decorator lowering | legacy parser facts, `component_graph`, new lowering boundary | Legacy-to-canonical lowering records; compatibility fixtures and migration output. |
| 9 | Vite backend skeleton | new `packages/vite`; package workspace; compiler publication API | Empty compiler-product-to-Vite adapter boundary; package smoke test. |
| 10 | Virtual module registry | new `packages/vite`; `application_publication`, `tooling_products` | Versioned virtual-module names/content derived from compiler products; golden modules. |
| 11 | Dev server integration | `packages/vite`, CLI command adapters | Dev transport adapter with compiler diagnostics; dev-server integration probes. |
| 12 | Production build integration | `packages/vite`, `application_publication`, production products | Physical Vite output mapped back to Presolve manifest; production build fixtures. |

## Analysis

| PR | Scope | Repository trace | Products and proof |
| ---: | --- | --- | --- |
| 13 | Control-flow graph | new `control_flow`; `expression_graph`, IR lowering | Function CFG schema and control-flow corpus. |
| 14 | Function summaries | new analysis module; `semantic_graph`, `service` | Stable summaries for calls and dependencies; cross-module fixtures. |
| 15 | Purity/effect classification | `effect`, `effect_capability`, `computed_value` | Conservative purity/effect classifications and unknown-call diagnostics. |
| 16 | Capture and escape analysis | `resume_capture`, `resume_liveness`, `resume_boundary` | Capture/escape products; serialization and resume rejection fixtures. |
| 17 | Environment ownership | `context_ownership`, `form_ownership`, component instance scope | Ownership graph; isolation/cycle diagnostics and fixtures. |
| 18 | Serialization and codec protocol | `semantic_type`, `form_serialization`, `resume_schema`, `platform` | Versioned codec protocol; valid/invalid serialization corpus. |

## V2 language normalization

| PR | Scope | Repository trace | Products and proof |
| ---: | --- | --- | --- |
| 19 | Structural components and props | `component_*`, `composition_typing`, framework declarations | Canonical component/prop records; structural compatibility and artifact fixtures. |
| 20 | Slots | `slot`, `slot_binding`, `slot_content` | Slot ownership and projection products; nested/cold/resume cases. |
| 21 | State | `state_instance_storage`, `component_graph`, runtime codegen | State semantic records and update protocol; deterministic runtime fixtures. |
| 22 | Actions | `component_graph`, `lazy_action_chunks`, `runtime_*` | Action call/capture products; lazy activation and diagnostics. |
| 23 | Computed getters | `computed_value`, `computed_instance_slots`, `runtime_computed` | Cached computed products; cycles and invalidation fixtures. |
| 24 | Effects | `effect`, `effect_*`, `runtime_effect_*` | Effect scheduling/ownership products; cold/resume behavior and diagnostics. |
| 25 | Context tokens/providers | `context`, `provider`, `context_*` | Token/provider records and diagnostics; structural resolution cases. |
| 26 | Context consumption/resume | `consumer`, `context_resolution`, `context_resume` | Context resume plan; nearest-provider cold/resume fixtures. |
| 27 | Forms and fields | `form`, `form_field`, `form_binding`, `form_ir` | Canonical form/field products; generated form artifacts and diagnostics. |
| 28 | Validation/coercion | `form_validation`, `form_validation_plan`, `form_diagnostics` | Validation/coercion protocol; invalid and normalized-data fixtures. |
| 29 | TSX binding | parser syntax, `template_semantics`, `binding_table`, template graph | Typed TSX binding model; syntax/type diagnostic corpus. |
| 30 | Form submission/resume | `form_submission`, `form_submission_host`, `resume_*` | Submission/resume records and artifacts; browser cold/resume probes. |
| 31 | Resources | `resource`, `runtime_resource_artifact`, route products | Resource lifecycle product; cancellation, error, and resume fixtures. |
| 32 | Loaders | `route_loader`, `route_graph`, publication | Loader declaration/publication products; route integration tests. |
| 33 | Server actions | `route_server_action`, `semantic_package`, CLI deployment adapter | Server-action contract and capability diagnostics; route/runtime tests. |
| 34 | Capabilities | `semantic_capability`, `semantic_package`, effect capability registry | Resolved capability admission model; migration/explain tests. |

## Publication, tooling, and product proof

| PR | Scope | Repository trace | Products and proof |
| ---: | --- | --- | --- |
| 35 | Stable IDs and incremental invalidation | `semantic_id`, `watch`, `persistent_cache`, `platform` | Stable identity and invalidation products; reorder/edit regression corpus. |
| 36 | Versioned publication/resume contracts | `application_publication`, `resume_manifest`, `template_manifest`, runtime artifacts | Schema migration policy and cold/resume equivalence fixtures. |
| 37 | Presolve-aware HMR | `watch`, new Vite adapter, compiler publication products | Semantic invalidation to Vite transport mapping; HMR browser probes. |
| 38 | Production audit | `production_audit`, `production_*`, Vite manifest adapter | Audit report product and failure diagnostics; production fixture/budget checks. |
| 39 | Source maps | Vite adapter, publication manifest, CLI explain | Source-map translation product; source-location regression fixtures. |
| 40 | LSP and explain | `tooling_*`, `explain`, WASM, language-service, LSP, VS Code | Canonical query projections; hover/definition/reference/explain integration tests. |
| 41 | Migration command | CLI command modules, `semantic_capability`, new codemod package | Versioned migration report/codemods; before/after project fixtures. |
| 42 | V2 scaffold/examples | `create-presolve`, framework package, Vite package | V2 starter contract; scaffold snapshot and install/build test. |
| 43 | Representative applications | new application fixtures; CLI/Vite test harness | Product-level cold/resume/HMR/build evidence for supported app shapes. |
| 44 | Performance budgets | `production_reports`, `production_benchmarks`, compile-cost tooling | Versioned budgets and benchmark baselines; regression gate. |
| 45 | Beta hardening | all public packages, schemas, fixtures, docs, release scripts | Compatibility matrix, release dry run, determinism, diagnostics, artifact, and product gates. |

## Sequencing guardrails

Rows 2 through 8 establish the TypeScript and authored-semantics foundation.
Rows 9 through 12 establish a Vite adapter that consumes compiler products.
Rows 13 through 18 establish analysis contracts before language features rely on
them. Later rows may extend the canonical model but must not bypass these
boundaries with package-specific recognition or a parallel compiler path.
