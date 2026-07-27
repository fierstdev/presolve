# Environment ownership contract

`presolve_compiler::environment_ownership` is schema v1 of the V2 immutable
ownership projection. Its input is explicit canonical facts: every supplied
semantic value has a stable ID, environment class, lifetime class, and source
provenance. The five environment classes are compile-time, server, browser,
shared serializable, and opaque external. Lifetime classes are application,
route, request, component instance, action invocation, effect execution, and
resource load.

The product does not derive classifications from a module path, source spelling,
or package name. A future lowering owns completeness of these facts and may
adapt the existing Context, Form, and component-instance ownership products;
this graph does not replace or reinterpret them.

## Edges and diagnostics

Facts distinguish ownership edges from references. All endpoints and configured
browser-artifact/shared-publication roots must name supplied canonical nodes;
duplicate IDs and unknown endpoints are rejected.

The graph reports an ownership cycle, a path from a browser artifact root to a
server value, and a path from a shared-publication root to a request-lifetime
value. Every diagnostic includes the complete deterministic semantic-ID path
that caused it. This enforces that server-only values cannot enter browser
artifacts and request-private values cannot enter shared publication.

The product is diagnostic evidence only. It neither changes existing ownership
graphs nor grants publication or serialization permission beyond the facts it
has validated.
