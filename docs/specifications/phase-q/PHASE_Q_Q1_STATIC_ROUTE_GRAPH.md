# Phase Q Q1 static route graph

**Status:** route publication product complete; CLI adoption next.

The compiler now exposes a validated schema-v1 static route graph/manifest over
existing `@route("/path")` component records. Paths are sorted deterministically
and reject dynamic segments and duplicate ownership. This is a compiler
identity product only; it does not yet publish route artifacts or expose a CLI
route command.

`build_static_route_publication_v1` accepts explicit complete application
requests, validates that each selected entry owns one route, derives each
route's canonical application product, namespaces its exact artifacts, and
emits `routes.manifest.json`. The next Q1 slice exposes this compiler product
through an explicit CLI request; it may not use source discovery or construct
route pages in JavaScript.
