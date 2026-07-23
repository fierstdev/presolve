# Phase R File Routing Contract

## Authority

The compiler derives the ergonomic route topology from logical source paths.
The framework and CLI consume that product; neither interprets route files or
implements a router runtime.

The frozen explicit `@route()` static-route product remains available for
hermetic applications. This contract adds a separate file-route graph and does
not alter its manifest or publication semantics.

## Routes

`app/routes/index.tsx` derives `/`. A nested `index.tsx` derives its directory
path, and any other `.ts` or `.tsx` basename derives one path segment.
`[name]` derives the typed dynamic segment `:name`. Parameter names contain
only ASCII letters, digits, or underscores.

An explicit `@route("/path")` on a route component overrides that component's
file-derived path. A route path is unique by match shape: parameter names do
not make distinct routes, so `/posts/:id` and `/posts/:slug` conflict.

## Layouts

`app/layout.tsx` is the application layout. A `layout.tsx` below
`app/routes/` is the layout for that directory and all of its descendants.
A conventional layout that declares components must declare exactly one and may
not declare `@route()`. A route receives its ordered layout chain from the
application layout through its nearest directory layout. Empty layout files
currently have no semantic component and therefore contribute no layout; their
source-level diagnostic is deferred until the compiler model retains empty
route modules.

The file-route graph records this ownership chain as compiler semantic data.
It does not yet prescribe a framework router runtime or silently synthesize
rendering code; a later route-publication slice must consume the exact chain
when it defines the page composition artifact.

## Diagnostics

* `PSROUTE1010_LAYOUT_CANNOT_DECLARE_ROUTE` — a conventional layout attempts
  to be a page.
* `PSROUTE1011_LAYOUT_COMPONENT_AMBIGUOUS` — a layout scope has multiple
  component declarations.
* `PSROUTE1012_INVALID_FILE_ROUTE_PATH` — an override is not a supported
  static/parameterized file-route path.
* `PSROUTE1013_FILE_ROUTE_CONFLICT` — two pages match the same request shape.
