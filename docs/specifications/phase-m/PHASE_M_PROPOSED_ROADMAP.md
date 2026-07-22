# Proposed Phase M roadmap: Presolve Framework Foundation

**Status:** Owner-directed proposal for acceptance; not implementation authority.

## Product boundary

Phase M establishes the Presolve Framework as a distinct user-facing product
layer built on the frozen Presolve Compiler. The framework is not a thin set of
compiler commands, generated artifacts, or re-exported compiler types. It owns
an ergonomic application programming model, framework runtime integration,
developer-facing diagnostics, conventions, examples, and its own compatibility
contract. The compiler remains the sole authority for language semantics,
canonical identities, diagnostics, and compiler products.

The future Presolve Metaframework is expressly out of scope. Routing, data
loading, server rendering, build/dev orchestration, deployment adapters,
hosting, package installation, and `presolve create` belong to that later
product layer. `create`, `dev`, `benchmark`, and `doctor` therefore remain
reserved exit-6 commands throughout Phase M.

## Architectural decisions

| Decision | Phase M direction |
| --- | --- |
| Repository topology | Establish an isolated top-level `framework/` area for framework source, package manifests, fixtures, examples, and tests. It cannot change compiler crates or take ownership of existing compiler packages. |
| Compiler integration | Use only documented canonical compiler products and public package boundaries through a narrow framework adapter. No framework-owned parser, semantic analyzer, product decoder, source discovery, or alternate compiler path. |
| Programming model | Define framework-owned application/composition APIs that hide raw product plumbing while preserving compiler-established semantics and diagnostics. Syntax or semantics changes require a separate amendment. |
| Runtime | Provide one framework runtime integration boundary over compiler-produced runtime/resume products. It may add framework lifecycle and composition conventions, never a competing renderer or resume protocol. |
| Package boundary | Publish no package in Phase M. The framework package is local/private alpha evidence until a later authorized distribution decision. |
| Metaframework separation | No router, loader, SSR host, dev server, bundler, deployment target, project generator, or package manager behavior. |
| Compatibility | Freeze a framework compatibility matrix separately from compiler bytes; framework versions cannot silently alter compiler product meanings. |

## Slice sequence

### M0 — framework constitution and acceptance amendment

Accept this roadmap, establish `framework/` ownership, define the compiler /
framework / metaframework boundary, public terminology, compatibility policy,
and rollback. The amendment names every frozen Phase L representation it may
consume and confirms no compiler semantics or reserved-command disposition
changes. No framework implementation is added.

### M1 — isolated topology and package contract

Create the `framework/` workspace boundary and a private framework package
contract. Freeze allowed dependencies, exports, generated-artifact policy,
test roots, examples, and prohibition on imports from compiler internals. The
slice does not expose a user API or build a runtime.

### M2 — application programming-model contract

Specify the framework's concise author-facing application, composition, state,
event, context, component, and form API shapes. Map every capability to an
existing compiler authority and explicitly list unsupported interactions. This
is a framework API contract, not a new language-semantics contract.

### M3 — compiler-to-framework adapter

Implement a narrow adapter that accepts only canonical compiler products and
maps them into framework-owned application descriptors. It is deterministic,
strictly validates the supported product boundary, retains no source text, and
cannot invoke an alternate compiler or decoder.

### M4 — framework runtime integration

Implement the single framework runtime integration boundary that composes the
existing runtime/resume products into the M2 application model. Define mount,
update, error, cleanup, and resume handoff rules without changing renderer,
resume, or compiler protocol bytes.

### M5 — ergonomic primitive implementation

Implement the contracted framework primitives for application composition,
state/action/computed use, context, components/slots, effects, and forms.
Each primitive has exact compiler-product provenance, diagnostic propagation,
deterministic fixture output, and an explicit unsupported boundary.

### M6 — framework DX and diagnostics

Add framework-level error presentation, inspection, and local test utilities
that translate only already-established compiler facts into author-facing
guidance. No source scanning, editor protocol, dev server, watcher, timing
gate, or source-map system is activated.

### M7 — framework examples and conformance matrix

Create framework-owned examples for composition, state/actions/computed,
context/slots, effects/forms, and resumability. Each proves the framework API,
compiler-product provenance, runtime behavior where relevant, and absence of
metaframework authority.

### M8 — framework compatibility, docs, and handoff

Freeze the framework public surface, support matrix, versioning policy,
diagnostic/error contract, examples, and migration/rollback rules. Document
the metaframework handoff requirements: canonical application descriptor,
routing/data/SSR decisions, dev/build authority, deployment boundary, and
project-template ownership. `presolve create` remains deferred to that later
roadmap.

### M9 — Phase M framework freeze

Run the full M0–M8 matrix plus inherited Phase L evidence. Freeze framework
fixtures, adapter/runtime contracts, package exports, compatibility table, and
metaframework exclusions. Completion requires a clean committed tree and no
change to compiler semantics, compiler bytes, or reserved-command status.

## Evidence matrix

| Concern | Required proof |
| --- | --- |
| Layer separation | import/dependency audit proving no compiler internals or metaframework authority |
| Framework API | contract-to-compiler-authority map and exact unsupported-capability matrix |
| Adapter integrity | canonical-product fixtures, strict validation, deterministic descriptors, no source retention |
| Runtime integrity | browser/runtime and resume conformance against frozen compiler products |
| Ergonomic DX | author-facing diagnostics, examples, and framework-only conformance fixtures |
| Compatibility | version/support matrix, migration/rollback fixtures, public docs checks |
| Metaframework deferral | audit for no router, server, bundler, deployment, generator, package manager, or reserved command |

## Acceptance checklist

Before M0 starts, the owner must explicitly accept this framework direction,
approve the `framework/` topology and package identity, select the initial
application-model API scope, and approve the compiler/framework compatibility
boundary. No slice may begin from this proposal alone, and `presolve create`
remains a metaframework concern until a future accepted roadmap defines it.
