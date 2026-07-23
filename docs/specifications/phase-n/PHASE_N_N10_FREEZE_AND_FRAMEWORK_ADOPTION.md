# Phase N N10 freeze and framework adoption

**Status:** freeze authority.

Phase N freezes the compiler-owned semantic capability registry schema v1 and
its JSON, human matrix, and migration projections. Admitted entries are the
only framework-authorable forms; deferred entries remain compiler rejections
with no compatibility spellings or framework fallback.

## Opaque package terminal

The only admitted opaque form is
`@action() @opaque("package", "export") method(): void {}` on an empty,
synchronous, zero-parameter Action. Its named package/export must match an
imported `opaque` semantic-package contract with exact version, SHA-256
integrity, client boundary, `() -> void` signature, `cold_fallback` resume
policy, and explicit host runtime-module location.

The compiler emits `opaque.runtime.json` schema v1 and the generated runtime
imports only that location after the compiler-owned Action batch. Third-party
code has no compiler State, Form, Context, Resource, render, or resume-write
authority. Malformed records fail before execution; opaque presence forces
resume cold fallback. Package implementation is never parsed or inspected.

## Framework conformance

`@presolve/framework-types` may declare `opaque(packageSpecifier, exportName)`
only as a standard TypeScript decorator declaration. It may not import,
resolve, validate, schedule, or execute package code. Package typings remain
the application/package author's responsibility; semantic package contracts and
runtime mappings are canonical compiler build inputs.

The Phase M freeze remains in effect for all unchanged forms. This N10
amendment adds only the admitted N9 declaration, not a framework runtime,
adapter, parser, package installer, or compatibility shim.

## Evidence

`scripts/verify-n10-phase-n-freeze.sh` is the focused freeze gate. It replays
the N0 registry and N8 matrix/migration projections, opaque
declaration/contract/artifact tests, the real-browser terminal/malformed/resume
matrix, framework declaration typing under the pinned TypeScript 7.0 CLI, and
a clean diff. Earlier completed N family verifiers remain their canonical
per-capability evidence rather than being reinterpreted by the framework.
