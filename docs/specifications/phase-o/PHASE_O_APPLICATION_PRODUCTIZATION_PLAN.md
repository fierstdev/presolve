# Phase O: Application Productization

**Status:** active. O0 and O1 are implementation authorities.

## Objective

Turn the frozen compiler and framework into an operable application product
without creating a second compiler, renderer, reactive runtime, package
resolver, or artifact protocol. Phase O owns application-oriented invocation,
configuration presentation, and development orchestration only where the
existing compiler already has a canonical product.

## Governing rule

```
caller-owned application request
        ↓
Phase O validation and exact CLI invocation projection
        ↓
canonical Presolve CLI/service product
        ↓
unchanged compiler artifacts and generated runtime
```

Phase O never obtains authority over source semantics, dependency analysis,
package implementation, runtime scheduling, DOM updates, Context, Forms,
Resources, resume, or artifact decoding.

## Slice sequence

### O0 — application product constitution

Freeze ownership, package names, invocation boundaries, compatibility policy,
and explicit exclusions. Verify all proposed commands against the public CLI
before implementation.

### O1 — explicit application build handoff

Create one private metaframework package that validates a caller-owned
single-entry build request and projects it to the exact existing `presolve
build` syntax, including explicit semantic-package contracts/runtime mappings
and production mode. It reads neither source nor package files and preserves
executor results unchanged.

### O2 — explicit workspace development handoff

Expose caller-owned configuration/source lists through the existing `presolve
workspace` and `presolve watch --once` products. It must not add file watching,
source discovery, HTTP serving, HMR, or diagnostics reinterpretation.

### O3 — application request and failure presentation

Define a versioned application request envelope, command-result passthrough,
and documentation for compiler diagnostics and artifact locations. It may not
decode artifact bytes or replace compiler errors.

### O4 — multi-source artifact-publication decision

The current public artifact publisher accepts one entry source while the L9
workspace product accepts explicit multi-source configuration but does not
publish application artifacts. Phase O must not paper over that mismatch. This
slice is blocked unless a future separately versioned compiler publication
contract provides a multi-source artifact product.

### O5 — routing and server product intake

Routing, layouts, request-time rendering, loaders, server actions, sessions,
and middleware require their own compiler/runtime contracts. They are not Phase
O conveniences and cannot be inferred from static application artifacts.

### O6 — deployment and public distribution intake

Package publication, environment-variable policy, deployment targets, asset
hosting, secrets, observability, and adapter contracts require separate product
authority. No provider integration is implied by O1–O3.

## Completion boundary

Phase O may be frozen after O1–O3 only as an explicit single-entry application
orchestration product. It must not claim general multi-page, SSR, or deployed
application readiness until the blocked/missing compiler and platform contracts
above are resolved.
