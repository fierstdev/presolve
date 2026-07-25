# Alpha characterization snapshot

Snapshot date: 2026-07-25<br>
Repository commit: `a1ac8503f52ad27af03224c95714a19d73add1a9`<br>
Release baseline: `0.1.0-alpha.1`

This is a characterization of checked-in behavior, not a claim that every
existing alpha product satisfies the V2 contract.

## Repository map

| Surface | Current repository fact | Existing proof surface | V2 consequence |
| --- | --- | --- | --- |
| Syntax frontend | `crates/presolve_parser` uses OXC `0.87`; `oxc_adapter.rs` retains selected TypeScript and TSX facts in `model.rs`. | `crates/presolve_parser/tests/parse_file.rs` and parser unit tests. | Retain source fidelity while introducing a TypeScript-authority boundary; do not extend the selected-fact model into a second type checker. |
| Compiler semantics | `crates/presolve_compiler` owns `ApplicationSemanticModel`, component graphs, IR, diagnostics, routes, forms, resources, publication, and resume products. | Compiler unit tests plus CLI fixture suites. | Normalize legacy and V2 authoring into one authored semantic model; preserve compiler ownership of semantic meaning. |
| Legacy authoring | `framework/packages/presolve/src/index.d.ts` exports inert decorator-style intrinsics and `Component`; parser and component graph recognize their current forms. | `tests/framework-public-api`, `framework/packages/presolve` smoke test, and numbered fixture corpus. | Compatibility lowering must be explicit and terminate in the canonical V2 model. |
| TypeScript | The workspace depends on `@typescript/native` `7.0.2`, while no compiler-owned TypeScript semantic adapter package or protocol exists. | `pnpm run test:types`; the VS Code package delegates ordinary checking to the project TypeScript service. | This is the first architectural gap: V2 needs a single adapter for symbol, type, resolution, and diagnostic truth. |
| Symbol/module facts | `binding_table.rs`, `module_graph.rs`, `semantic_reference.rs`, `semantic_id.rs`, and parser import/export facts provide compiler-local identity products. | Component, context, and semantic fixture suites. | V2 must anchor intrinsic recognition to TypeScript-resolved symbol identity, including aliases and re-exports. |
| Publication and resume | Compiler products include template, resume, runtime, application-publication, production-artifact, and tooling schemas. | `component_fixtures.rs`, `application_publication.rs`, `runtime_browser.rs`, production suites. | Reuse these compiler-owned seams, version any new cross-package contracts, and prove cold/resume equivalence at their boundary. |
| CLI and project boundary | `presolve-cli` has command, configuration, compilation, workspace, cache, and Cloudflare deployment modules. | CLI integration tests, configuration fixtures, ergonomic-project coverage. | Keep V2 orchestration as an adapter over compiler products; Vite must not enter compiler semantic modules. |
| Development/build platform | The monorepo has no `vite` dependency or `@presolve/vite` package. Current CLI owns development and build orchestration. | CLI build checks, production fixture suites, browser tests. | Add Vite only at a new package boundary; it will handle transport and physical packaging, not semantic analysis. |
| Tooling | Compiler tooling products are exposed through WASM, language-service, LSP, CLI `explain`, and a VS Code extension. | Package smoke tests, `crates/presolve_cli/tests/explain.rs`, VS Code fixture. | Extend these projections from canonical products instead of reconstructing semantic facts in JavaScript. |
| Scaffold and applications | `create-presolve` creates a TypeScript 7 project; `packages/testing` and numbered fixtures supply the current product corpus. | Create-package tests, scaffold verifier, CLI fixtures, browser tests. | V2 migration, scaffold, and representative applications need new focused fixtures rather than a silent rewrite of the alpha corpus. |

## Current semantic pipeline

```text
TypeScript or TSX source
  -> OXC parse plus selected source facts
  -> compiler-owned component and application semantic models
  -> graphs, diagnostics, IR, and runtime/publication plans
  -> CLI-generated HTML, browser artifacts, manifests, and deployment inventory
```

The pipeline already puts semantic products in the compiler, which aligns with
V2's ownership direction. It does not yet provide the V2-required TypeScript
semantic authority, general source AST product, canonical intrinsic registry,
or Vite boundary.

## Existing product seams

| Product family | Principal modules | Primary current tests |
| --- | --- | --- |
| Parsing and source facts | `presolve_parser::{oxc_adapter,model}` | `parse_file.rs` |
| Authored/component semantics | `application_semantic_model`, `component_graph`, `component_*`, `binding_table`, `semantic_*` | `component_fixtures.rs` and numbered component fixtures |
| Reactivity and analysis | `expression_graph`, `computed_value`, `effect`, `semantic_type`, `resume_capture`, `resume_liveness` | compiler unit tests and effect/computed fixtures |
| Forms and server interaction | `form_*`, `resource`, `route_loader`, `route_server_action` | CLI component fixtures and browser probes |
| Publication/runtime/resume | `intermediate_representation`, `runtime_*`, `resume_*`, `template_manifest`, `application_publication` | `runtime_browser.rs`, `application_publication.rs` |
| Production and incremental work | `production_*`, `watch`, `persistent_cache`, `platform` | production baseline/budget/runtime suites |
| Tooling and packages | `tooling_*`, `explain`, compiler WASM, language service, LSP, VS Code | `explain.rs` and package smoke tests |

## Compatibility constraints discovered

1. The alpha parser accepts TypeScript/TSX syntax through OXC, but it stores a
   purpose-built subset of facts. V2 must introduce source-faithful general
   syntax and TypeScript semantic truth without giving semantic authority to
   either a new parser subset or a Vite plugin.
2. Existing decorator spelling is an alpha authoring form. V2 recognition must
   resolve canonical symbols, so aliases and re-exports cannot be accepted by
   adding special spellings.
3. Existing compiler modules already publish durable and runtime-facing
   products. Compatibility work must adapt or version those products instead of
   rebuilding a parallel V2 compiler.
4. No Vite integration exists. The first Vite work therefore has to establish
   an external adapter and virtual-module contract before it can supply dev or
   production transport.

## Baseline evidence

The snapshot was taken from 508 tracked paths at the commit above. It inspected
the workspace manifests, all crate/package boundaries, compiler module exports,
the parser frontend, framework declarations, CLI integration tests, browser
tests, and public architecture documentation. Documentation validation is the
appropriate executable proof for this mapping; semantic and release suites are
intentionally deferred until they exercise a changed product.
