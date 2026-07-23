# Phase R: Ergonomic Metaframework

**Status:** active.

## Goal

Make the normal Presolve application workflow lower-friction than equivalent
React/Next workflows without adding a second parser, router runtime, renderer,
artifact merger, or deployment authority.

The normal path is `presolve dev`, `presolve build`, and `presolve deploy` from
one project root. The compiler derives and records all defaulted source roots,
logical paths, component identities, routes, package contracts, and release
inventory as canonical products. Explicit protocol inputs remain an advanced
hermetic interface.

## R0 — ergonomic constitution

Freeze default roots, optional `presolve.config.ts`, default export route
entries, inferred `@component()` identity, compatibility policy, and the
compiler-owned discovery boundary. TypeScript 7.0 is the baseline and 7.1
decorator/type behavior is a compatibility target.

## R1 — public `presolve` authoring package

Ship normal importable authoring declarations and JSX types, moving ambient
globals to compatibility-only status. Support `@component()` identity inference
while retaining explicit identity as an advanced override.

## R2 — canonical project discovery

Add deterministic compiler-owned discovery from a named root/configuration
request. Default `app/` and `app/routes/`; preserve advanced explicit source
mode. Record discovery fingerprint and reject ambiguity.

## R3 — file routes

Lower `app/routes/index.tsx`, nested segments, and `[parameter]` segments into
the compiler route graph/manifest. Add deterministic layout convention and
route conflict diagnostics; preserve explicit `@route()` as an override.

## R4 — simple commands

Implement canonical `presolve dev`, `build`, `check`, and `deploy` commands
over discovery/compiler products. No command reimplements compiler semantics.

## R5 — package ergonomics

Discover published Presolve semantic contracts from declared package metadata;
give clear actionable diagnostics and retain an explicit opaque escape hatch.

## R6 — server/data boundary

Add compiler-owned route parameter, request, loader, server-action, response,
cache, and error-boundary products. No generic arbitrary server runtime.

## R7 — deployment adapters

Add one provider adapter over the frozen release handoff, including public
configuration, secret bindings, rollback, integrity, and audit products.

## R8 — scaffold and DX

Add `npm create presolve`, templates, explain views, examples, and migrations
from advanced explicit mode.

## R9 — usability freeze

Require a fresh-app proof with no configuration/manual source list/manual
component identity/artifact handling, plus browser/build/route/server/release
and compatibility matrices.
