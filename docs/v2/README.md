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

## Current boundary

The active slice is architecture mapping and characterization only. It makes no
runtime, compiler-semantic, package, or generated-artifact change. The next
slice is the TypeScript compatibility corpus; it may begin only after this
snapshot and tracker have been reviewed as the baseline for its assertions.
