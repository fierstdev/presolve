# Node deployment and static-export contract

`presolve deploy node --prepare` publishes a schema-v2 Node release inventory
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

The schema-v2 host serves static routes and canonical Form-bound server actions.
It verifies the digest of the Vite-bundled named-export registry before import,
accepts only compiler-issued action coordinates, and owns the closed
FormData/abort/JSON/redirect/typed-failure lifecycle. A route whose only
dynamic records are executable server actions continues to serve its compiled
HTML on `GET`; the action path admits only `POST`. Loader-bearing routes return
stable `501` because loader result codecs and a Resource bootstrap target are
not yet compiler products.

The [Node capability executor contract](node-capability-executor-contract.md)
is the additive successor. It requires a canonical authored Form binding,
compiler-issued request coordinate, digest-bound server registry, and closed
request/response lifecycle before a server action may execute. Route loaders
remain fail-closed until their result codecs and Resource bootstrap target are
published as compiler facts.

This is intentionally distinct from the Cloudflare static adapter. Both
adapters consume the same compiler artifacts, but neither re-matches source
files, derives route topology, or modifies the published identities.
