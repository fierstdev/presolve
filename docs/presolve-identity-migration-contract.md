# Presolve identity migration contract

**Status:** Authoritative owner-directed amendment, 2026-07-21.

This amendment replaces the retained-identity exceptions in the prior public
identity transition. It is a representation migration only: authored language
semantics, identity construction, ordering, validation, and the authority
boundaries frozen by Phases A--K remain unchanged. Every active product-facing
representation is re-emitted under the Presolve namespace and every affected
fixture is updated in the same change.

## Canonical namespaces

| Surface | Canonical spelling |
| --- | --- |
| Public compiler diagnostics | `PSC` plus the established numeric suffix |
| Internal inspection diagnostics | `PSASM` plus the established numeric suffix |
| Browser runtime diagnostics | `PSR_` plus the established suffix |
| HTML attributes | `data-presolve-*` |
| HTML comments and element identifiers | `presolve-*` |
| Browser globals | `window.__PRESOLVE__` and `window.__PRESOLVE_*__` |
| Rust implementation crates and paths | `presolve_*` and `crates/presolve_*` |
| CLI inspection command | `presolve explain` |

Numeric suffixes, marker payloads, manifest schemas, canonical ordering, and
all non-namespace bytes retain their established meaning. The migration is not
a compatibility layer: the old spellings are not parsed, emitted, documented,
or accepted as aliases.

## `presolve explain`

`presolve explain` is the sole inspection command. Its source-summary view
remains the default. Its semantic-inspection view is selected by `--inspect`,
by an entity/source selector, or by `--format graph`; the latter continues to
emit the semantic graph. `--inspect` permits a complete semantic inspection
without a selector. The former short inspection command fails closed with exit
code 6 and directs callers to `presolve explain`.

## Scope and evidence

The active repository contains no old product identity in compiler source,
runtime source, CLI behavior, fixtures and goldens, browser probes, public
documentation, package/workflow metadata, or verifiers. The migration verifier
checks these classes and checks that only `presolve explain` is documented and
dispatched.

`docs/archive/` and `notes/progress/` are immutable historical provenance and
are intentionally excluded from this spelling rule. They are not current
product surfaces, cannot define behavior, and must remain visibly archival.

## Completion conditions

The amendment is complete only when the identity verifier passes, focused CLI
inspection and runtime fixture suites pass, generated fixture bytes are
regenerated from the compiler, documentation links remain valid, and the
repository-wide Phase L gate passes. A future compatibility or namespace
change requires a new explicit amendment.
