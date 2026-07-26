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
- [V2 authoring-syntax cutover contract](authoring-syntax-cutover-contract.md)
  records the decorator-free canonical source surface and the bounded alpha
  compatibility path.
- [V2 authoring build-adoption contract](authoring-build-adoption-contract.md)
  records the required authority bridge and downstream path for decorator-free
  projects.
- [Derived computed candidates](computed-derived-candidate-contract.md)
  defines the schema-v3 canonical-model amendment for analysis-proven,
  decorator-free computed getters.
- [V2 canonical ASM adapter contract](canonical-asm-adapter-contract.md)
  fixes the no-fallback boundary for canonical authoring records entering route
  and publication products.
- [V2 action-field runtime adoption contract](action-field-runtime-adoption-contract.md)
  defines the source/authority evidence required before V2 action fields reach
  the existing runtime product.
- [V2 action endpoint identity contract](action-endpoint-identity-contract.md)
  defines the no-synthetic-method migration from V2 action fields into the
  existing action-batch and runtime binding products.
- [V2 synchronous Action-parameter contract](action-parameter-contract.md)
  admits only typed static event arguments assigned directly to matching
  canonical State, using the existing ordinal runtime operation.
- [V2 synchronous Action-local-literal contract](action-local-literal-contract.md)
  projects compiler-retained local primitives to ordinary literal assignments
  without runtime source evaluation.
- [V2 effect-field source contract](effect-field-source-contract.md) defines
  the authority-backed, decorator-free effect declaration boundary before
  lifecycle runtime adoption.
- [V2 effect lifecycle adoption contract](effect-lifecycle-adoption-contract.md)
  defines cleanup programs, resume scheduling, ordering, and browser runtime
  ownership for V2 effect fields.
- [V2 effect instance-lifecycle contract](effect-instance-lifecycle-contract.md)
  defines the required instance-qualified ownership boundary before cleanup
  fields can be published.
- [Structural component materialization contract](structural-component-materialization-contract.md)
  defines the compiler-issued renderer and opaque occurrence-identity boundary
  required before dynamic structural component lifecycles can activate.
- [Structural occurrence identity contract](structural-occurrence-identity-contract.md)
  fixes the parent-scoped runtime codec required for nested conditional and
  keyed component materialization.
- [Structural instance state contract](structural-instance-state-contract.md)
  defines occurrence-qualified State and computed storage before dynamic
  templates may become live component instances.
- [Structural host renderer scope contract](structural-host-renderer-scope-contract.md)
  defines the exact static, nested, and keyed renderer inputs required before
  structural host fragments can be published or activated.
- [Structural static-conditional activation contract](structural-static-conditional-activation-contract.md)
  admits the first compiler-authoritative dynamic slice while keyed, nested,
  Slot, Effect, and resume paths remain fail-closed.
- [Structural keyed-host activation contract](structural-keyed-activation-contract.md)
  fixes per-item compiler membership before keyed reconciliation may activate.
- [Structural nested-activation contract](structural-nested-activation-contract.md)
  fixes compiler membership for recursive structural occurrence creation.
- [Environment-read lowering contract](environment-read-lowering-contract.md)
  defines the manifest-backed source boundary for browser-visible environment
  values.
- [Vite adapter boundary](vite-adapter.md) records the compiler-product-only
  integration seam for the required external backend.
- [Presolve-aware HMR contract](hmr-contract.md) records the compiler-selected
  update vocabulary, state-preservation evidence, and Vite transport boundary.
- [Production audit contract](production-audit-contract.md) records the
  compiler-produced production-report audit and adapter verification boundary.
- [Source maps contract](source-maps-contract.md) records Vite physical-map
  ownership and the manifest-bound compiler artifact translation boundary.
- [Vite styles and assets contract](vite-assets-contract.md) records explicit
  Vite-owned CSS, PostCSS/Tailwind, imported-asset, and public-directory input.
- [Tooling query contract](tooling-query-contract.md) records the canonical
  WASM, language-service, LSP, hover, and explain query boundary.
- [Migration command contract](migration-contract.md) records the
  registry-derived report and explicit no-unowned-source-rewrite boundary.
- [V2 scaffold contract](scaffold-contract.md) records the conventional
  Application Platform layout and public-environment starter boundary.
- [Representative applications contract](representative-applications-contract.md)
  records the conventional-project build corpus and its product evidence.
- [Route metadata contract](route-metadata-contract.md) records the
  compiler-owned sidecar schema and publication boundary.
- [Node deployment contract](node-deployment-contract.md) records the
  compiler-issued Node release inventory and static-export eligibility rule.
- [Testing integration contract](testing-integration-contract.md) records the
  Vitest and Playwright adapters over the existing Vite compiler boundary.
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
- [Explicit environment-input contract](environment-input-contract.md) records
  the named-file public/server value classification boundary.
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
- [Computed source-recognition contract](computed-source-contract.md) records
  the decorator-free getter proof boundary before runtime adoption.
- [Context tokens/providers contract](context-contract.md) records canonical
  token and provider inspection.

## Current boundary

The tracker records repository-owned V2 authority products through the
compiler-issued Node release inventory and static eligibility classification.
It remains the source of implementation status and intentionally separates
those products from the remaining server-executor, environment, performance,
and hardening evidence required for beta readiness.
