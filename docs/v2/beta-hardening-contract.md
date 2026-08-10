# Beta hardening contract

The final beta gate is a reproducible compatibility and release proof over
compiler-issued products. It does not permit a test-only decoder, a legacy
authoring fallback, or a packaging-only substitute for browser behavior.

## Required evidence

`pnpm release:check` must pass from a frozen workspace installation. It
composes:

1. workspace formatting and strict Rust lint;
2. deterministic production baselines, budgets, reports, and representative
   application publication;
3. the real-browser compatibility matrix for canonical Actions, Effects,
   conditional/keyed/nested structural components, Slot projection, Context,
   Forms, Resources, diagnostics, resume fallback, and production CSP;
4. compiler, package, TypeScript compatibility, documentation, and public API
   checks;
5. a newly scaffolded application installed only from locally packed release
   artifacts, then checked, built, and deployment-prepared; and
6. parser crate packaging, native release preparation, VSIX packaging, and
   SHA-256 reporting for every published package artifact.

The scaffold verifier must override every unpublished direct workspace
dependency from its packed tarball. A registry fetch is not release evidence
for an unpublished package.

## Public-contract integrity

The capability registry, CLI diagnostics, scaffold, and public guides must
describe the same release. Canonical V2 examples use the current admitted
source forms. Any retained decorator example must be labelled as alpha
compatibility; the documentation verifier rejects an unlabelled guide example.
The CLI obtains its unsupported-platform diagnostic version from its
own package manifest, with a package-level regression test, so an old release
label cannot be shipped with a later binary package.

## Completion

The dry run emits the versioned `presolve.release-dry-run` artifact inventory
only after every gate passes. The beta source surface is the canonical V2
surface, including the closed Action product recorded in
`action-beta-surface-contract.md`. Legacy decorators remain compatibility-only
and cannot satisfy this gate.
