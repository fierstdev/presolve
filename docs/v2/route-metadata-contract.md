# Route metadata contract

Route metadata is a compiler-owned publication product, not a Vite convention
or a component decorator. A route may declare one sidecar document at
`app/routes/<route>.metadata.json`; its identity is the canonical file route
selected by `route_graph`, never the sidecar filename alone.

Schema v1 admits only a non-empty string `title` and optional non-empty string
`description`. The compiler must reject a sidecar with no matching canonical
route, duplicate route metadata, unrecognized fields, non-string values, or a
sidecar for a layout. On success, publication emits one deterministic
`route-metadata.json` artifact keyed by compiler route path and entry component
identity. Vite, Node, and deployment adapters may transport that artifact but
must not infer metadata from HTML, source filenames, or module exports.

This contract deliberately does not introduce component syntax or runtime head
mutation. The next implementation slice must add sidecar discovery to the
existing ergonomic project input and join it against the compiler-issued route
manifest before publication.
