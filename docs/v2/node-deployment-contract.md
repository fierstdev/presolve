# Node deployment and static-export contract

`presolve deploy node --prepare` publishes a schema-v1 Node release inventory
at `.presolve/node/deployment.plan.json`, a syntax-checkable static host at
`.presolve/node/server.mjs`, and a minimal local `package.json`. The adapter
first runs the ordinary compiler-owned production publication, then consumes
only `file-routes.manifest.json`, `route-loaders.plan.json`, and
`route-server-actions.plan.json` from that output.

The inventory contains the immutable route path, compiler artifact root,
artifact digest, and one execution classification per route. A route is
`static` only if the compiler-issued loader and server-action plans both record
zero entries for that exact route. Otherwise it is `node`. Route sets must
match exactly across all three compiler products; malformed, duplicate, or
foreign route records fail rather than being reconciled by the adapter.

The generated host can serve only `static` routes and validates the release
inventory before it is written. It returns a stable 501 response for `node`
routes: the inventory proves that Node deployment is required, but it does not
pretend to execute a loader or server action. A capability-specific Node
executor remains a subsequent contract because no existing compiler product
defines server invocation, request decoding, or response serialization.

This is intentionally distinct from the Cloudflare static adapter. Both
adapters consume the same compiler artifacts, but neither re-matches source
files, derives route topology, or modifies the published identities.
