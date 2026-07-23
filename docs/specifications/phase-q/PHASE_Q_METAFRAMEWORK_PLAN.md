# Phase Q: Presolve Metaframework

**Status:** frozen through Q5.

## Objective

Deliver the first production-ready Presolve metaframework as a compiler-owned
application topology, request boundary, and deployment handoff. It must extend
the frozen framework rather than become a JavaScript UI runtime around it.

```text
explicit metaframework project request
        ↓
compiler-owned route/request/deployment products
        ↓
canonical application artifacts and manifests
        ↓
thin @presolve/application invocation projection
```

## Authority

The compiler owns route identity, conflict diagnostics, route-to-entry
topology, request/server capability boundaries, serialization, generated
artifact inventories, and deployment release identity. The framework keeps its
existing authoring vocabulary. `@presolve/application` validates only
caller-owned request shape, projects canonical CLI commands, and preserves
executor output unchanged.

No layer may parse application sources independently, discover files implicitly,
merge generated artifacts, execute a generic renderer, use a runtime router,
or infer deployment configuration.

## Q0 — constitutional contract

Freeze public package/command names, compatibility policy, explicit project
inputs, route and request ownership, deployment handoff, and unsupported
features. Existing applications stay explicit: no file-system route discovery
or magical project configuration.

## Q1 — static route graph and publication

Introduce a compiler-owned route request with explicit route declarations and
one explicit component entry per route. Validate path grammar, duplicate and
ambiguous routes, parameter names, and route-entry identity. Publish a
route-manifest plus a static artifact inventory through the existing canonical
application product; do not create a client-side router runtime.

## Q2 — navigation and layout topology

Extend the compiler route graph with static nested layouts, route parameters,
and compiler-generated navigation metadata. Navigation is ordinary browser
document navigation in v1. No SPA interception, generic client router, or
untyped nested children API is admitted.

## Q3 — request/server boundary

Add an explicit server target product: typed request inputs, allowed terminal
capabilities, handler ownership, response/serialization rules, and static
versus request-time artifact topology. SSR, streaming, loaders, server actions,
sessions, and middleware are separately gated subproducts; no generic Node
server is embedded in the metaframework.

## Q4 — deployment handoff

Define a versioned deployable release manifest derived from compiler artifacts,
target capability declarations, public configuration only, immutable release
identity, asset integrity, rollback metadata, and audit diagnostics. Provider
adapters project this product; secrets remain caller/provider-owned.

## Q5 — DX, examples, compatibility freeze

Add compiler-backed `presolve explain route`/deployment inspection, examples
for static routes and request boundaries, positive/negative fixtures, browser
evidence where navigation executes, and the framework/compiler/metaframework
compatibility matrix. Freeze the public metaframework exports and unsupported
surface.

## Initial exclusions

File-based discovery, SPA router runtime, HMR, dev server, generic SSR,
streaming, arbitrary server code, implicit environment variables, provider
SDKs, authentication/database abstractions, and package-manager scaffolding
are excluded until their compiler/product authority is separately proven.
