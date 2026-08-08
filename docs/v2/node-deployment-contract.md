# Node deployment and static-export contract

`presolve deploy node --prepare` publishes a schema-v3 Node release inventory
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

The schema-v3 host serves static routes, canonical Form-bound server actions,
and canonical route loaders. It verifies the digests of both Vite-bundled
named-export registries before import. Action coordinates retain the closed
FormData/abort/JSON/redirect/typed-failure lifecycle. Loader routes use the
compiler's exact parameter mapping, data/error codecs, cache policy, and
Resource activation bootstrap; package modules remain server-only. A route
whose only dynamic records are executable server actions continues to serve
its compiled HTML on `GET`; the action path admits only `POST`.

The [Node capability executor contract](node-capability-executor-contract.md)
defines both closed execution lifecycles. Any missing compiler plan, Resource
bootstrap target, runtime module, registry digest, codec, or cache fact fails
preparation or execution rather than falling back to source evaluation.

This is intentionally distinct from the Cloudflare static adapter. Both
adapters consume the same compiler artifacts, but neither re-matches source
files, derives route topology, or modifies the published identities.
