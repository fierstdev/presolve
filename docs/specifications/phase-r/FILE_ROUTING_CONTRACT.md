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
Route publication consumes it through compiler-issued default-Slot child edges:
the outer layout is the materialization root, each following layout/page is
placed into the preceding layout's declared default Slot, and no generated
source, wrapper component, or router runtime is introduced.

## Publication

`FileRoutePublicationRequestV1` is the compiler product for a complete
discovered route project. It lowers every validated page through the existing
entry-scoped application-publication product and namespaces the resulting exact
artifact families below a compiler-issued route artifact root. Its schema-v1
manifest records route paths, entry components, layout chains, profile, and
the digest inventory. The compiler also resolves a request path to an opaque
published artifact path (or canonical trailing-slash redirect), including
static-over-parameter specificity. A host consumes that result; it does not
implement matching itself.

Each route is composed independently before its standard compiler artifact
family is lowered. The manifest's `entry_component_id` remains the page
identity, while the compiler selects the outer layout as the materialization
root when a layout chain exists. Runtime, Context, action, Slot, and resume
products derive from that same composed semantic model.

## Diagnostics

* `PSROUTE1010_LAYOUT_CANNOT_DECLARE_ROUTE` — a conventional layout attempts
  to be a page.
* `PSROUTE1011_LAYOUT_COMPONENT_AMBIGUOUS` — a layout scope has multiple
  component declarations.
* `PSROUTE1012_INVALID_FILE_ROUTE_PATH` — an override is not a supported
  static/parameterized file-route path.
* `PSROUTE1013_FILE_ROUTE_CONFLICT` — two pages match the same request shape.
