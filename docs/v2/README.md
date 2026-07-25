# Presolve V2 engineering handoff

This directory starts the implementation path from `0.1.0-alpha.1` to
`0.2.0-beta.1`. It is intentionally an engineering record rather than a second
user guide.

## Authority

Current behavior remains defined by this repository. V2 behavior is defined by
the user-supplied `presolve-v2-beta-specification.zip`, whose SHA-256 is
`483a5b3a2ea6e43970c64e93c63834f0a94f0942158f69662e10669d7ce3bf1b`.
The archive's `CODEX-HANDOFF.md` establishes the order of work: characterize
the current repository, create the implementation tracker, then establish
canonical authored semantics and the TypeScript authority boundary before any
V2 feature rewrite.

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

## Current boundary

The characterization, compatibility-corpus, semantic-authority, source-AST,
intrinsic-registry, canonical-authored-semantics, legacy-lowering, the Vite
backend skeleton, virtual-module registry, and dev-server integration are
complete, as are production build integration, the control-flow graph
foundation, function summaries, purity/effect classification, and explicit
capture/escape analysis and environment ownership. The next slice is the
serialization and codec protocol. The next slice is structural components and
props and slots. The next slice is state.
