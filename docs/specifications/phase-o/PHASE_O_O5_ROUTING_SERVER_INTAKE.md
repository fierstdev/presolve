# Phase O O5 routing and server product intake

**Status:** complete as an intake decision; no routing/server product is admitted.

The compiler retains a minimal `RouteGraph` record for components that already
carry a route path, but no public route declaration, route publication,
navigation runtime, layout ownership, request model, or server execution
contract exists. Phase O must not reinterpret that record as a router.

## Decision

O5 does not implement routing, layouts, request-time rendering, streaming,
loaders, server actions, sessions, middleware, or an HTTP server. The existing
static application publisher continues to publish exactly one explicit entry.

Those features require a successor compiler/runtime contract with, at minimum:

* versioned route declaration and parameter grammar;
* deterministic route conflict and layout ownership diagnostics;
* route-to-entry artifact topology and route manifest;
* request/input, data, serialization, and cache authority;
* server/client capability boundaries and failure semantics; and
* browser navigation/resume behavior.

Until that contract exists, `@presolve/application` exposes no router API and
the CLI exposes no route build or server command. This is a completed product
intake decision, not a deferred implementation hidden behind a convenience
API.
