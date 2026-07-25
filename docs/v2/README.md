# Presolve V2 engineering handoff

This directory starts the implementation path from `0.1.0-alpha.1` to
`0.2.0-beta.1`. It is intentionally an engineering record rather than a second
user guide.

## Authority

Current behavior remains defined by this repository. V2 behavior is defined by
the user-supplied base `presolve-v2-beta-specification.zip`, whose SHA-256 is
`483a5b3a2ea6e43970c64e93c63834f0a94f0942158f69662e10669d7ce3bf1b`, plus
the additive `presolve-v2-beta-specification-updated.zip`, whose SHA-256 is
`ea116678de4ea860daad03a1a50f5714470986d3831798d01784a9bf24dcf7d9`.
The base archive's `CODEX-HANDOFF.md` establishes the order of work:
characterize the current repository, create the implementation tracker, then
establish canonical authored semantics and the TypeScript authority boundary
before any V2 feature rewrite. The updated archive adds the Application
Platform workstream and gate before beta hardening.

The archive is an input artifact and is not copied here so that the tracked
documents remain a concise, reviewable account of its application to this
repository. Its `README.md`, `SUMMARY.md`, and `07-roadmap/` directory are the
normative source for this handoff.

## Start here

- [Alpha characterization snapshot](characterization.md) is the factual map of
  the repository at the V2 start point.
- [V2 implementation tracker](implementation-plan.md) maps all 45 specified
  pull requests to current modules, intended products, and proof surfaces.
- [TypeScript semantic-authority boundary](typescript-authority.md) records the
  V2 schema and ownership rule for TypeScript semantic queries.
- [General source AST](source-ast.md) records the source-faithful parser
  product and the compatibility role of the existing derived facts.
- [Canonical authored semantics](authored-semantics.md) records the normalized
  source-AST and TypeScript-identity boundary.
- [Vite adapter boundary](vite-adapter.md) records the compiler-product-only
  integration seam for the required external backend.
- [Presolve-aware HMR contract](hmr-contract.md) records the compiler-selected
  update vocabulary, state-preservation evidence, and Vite transport boundary.
- [Production audit contract](production-audit-contract.md) records the
  compiler-produced production-report audit and adapter verification boundary.
- [Source maps contract](source-maps-contract.md) records Vite physical-map
  ownership and the manifest-bound compiler artifact translation boundary.
- [Tooling query contract](tooling-query-contract.md) records the canonical
  WASM, language-service, LSP, hover, and explain query boundary.
- [Migration command contract](migration-contract.md) records the
  registry-derived report and explicit no-unowned-source-rewrite boundary.
- [V2 scaffold contract](scaffold-contract.md) records the conventional
  Application Platform layout and public-environment starter boundary.
- [Representative applications contract](representative-applications-contract.md)
  records the conventional-project build corpus and its product evidence.
- [Application Platform contract](application-platform-contract.md) records
  the additive project, environment, Vite, testing, and deployment boundary.
- [Control-flow contract](control-flow-contract.md) records the fail-closed
  IR projection and the coverage required before later analyses may rely on it.
- [Function-summary contract](function-summaries-contract.md) records the
  explicit call-fact boundary and conservative transitive-summary rules.
- [Purity and effect contract](purity-effect-contract.md) records the
  compiler-visible effect classes and conservative unknown classification.
- [Capture and escape contract](capture-escape-contract.md) records the
  explicit evidence boundary and fail-closed resume admission rule.
- [Environment ownership contract](environment-ownership-contract.md) records
  explicit environment/lifetime classifications and path-bearing leak rules.
- [Serialization and codec protocol contract](codec-protocol-contract.md)
  records independent serialization classes and versioned codec declarations.
- [Structural components and props contract](structural-components-contract.md)
  records the TypeScript-authority boundary for inheritance and generic props.
- [Slots contract](slots-contract.md) records immutable slot ownership and the
  explicit boundary before capture/resume analysis.
- [State contract](state-contract.md) records instance-qualified State storage
  and codec-backed resume admission.
- [Actions contract](actions-contract.md) records the authored action authority.
- [Computed contract](computed-contract.md) records computed runtime inspection.
- [Context tokens/providers contract](context-contract.md) records canonical
  token and provider inspection.

## Current boundary

The tracker records completed, repository-owned V2 authority products through
the compiler-selected HMR transport. It is the current source of implementation
status and intentionally separates those products from the remaining audit,
source-map, tooling, migration, representative-application, performance, and
hardening evidence required for beta readiness.
