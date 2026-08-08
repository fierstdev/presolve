# Route-loader execution contract

## Scope

A Presolve route loader is an integrity-bound server capability selected by a
canonical route-owned `Resource<Data, Error>` field. The compiler proves the
authored callback and package coordinate, publishes exact request and bootstrap
products, and the Node adapter executes only that closed plan. Application
component source and the loader module are never shipped to the browser.

## Canonical authoring

```tsx
import { Component, loader, type Resource, type RouteParameters } from "presolve";
import { loadPost } from "post-service";

type PostRecord = { slug: string; title: string };
type NotFound = { code: "not_found" };

export class Post extends Component {
  post: Resource<PostRecord, NotFound> = loader<PostRecord, NotFound>(
    async (params: RouteParameters, signal: AbortSignal) =>
      loadPost(params, signal),
  );

  render() {
    return <article>{this.post.data?.title ?? "Loading"}</article>;
  }
}
```

The TypeScript authority proves that `Component`, `loader`, `Resource`, and
`RouteParameters` are the canonical Presolve exports; that the endpoint is one
direct named import; and that its parameters are exactly canonical
`RouteParameters`, DOM `AbortSignal`, and a Promise result. Aliases are allowed
only when their symbol identities remain exact. Lookalikes, `any`, namespace or
default imports, captures, reordered arguments, extra statements, missing
authority results, and widened signatures fail closed.

The semantic-package export must be an integrity-qualified `resource` with a
`route_loader` capability, a `server` or `shared` execution boundary, abort
cancellation, reload resume, typed failure, and the exact signature
`(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>`.

## Compiler products

Route-loader plan schema v2 is emitted as deterministic
`dist/route-loaders.plan.json`. Every record joins the canonical file route,
component instance, Resource declaration and activation, semantic
`Resource<Data, Error>` codecs, and package binding. It publishes:

- the route, component, field, capability, package, version, integrity, export,
  and runtime-module identities;
- the exact data and error semantic types and closed resume codecs;
- Resource declaration, activation, component-instance, state, data, and error
  slot IDs;
- ordered route-parameter names and zero-based decoded URL segment indexes;
- strict UTF-8 percent decoding and duplicate-parameter rejection; and
- SHA-256 cache-key ingredients and the private-partition requirement.

Resource artifact schema v4 uses the same declaration and activation IDs. For
server route loaders it omits a browser runtime location and publishes one
server-bootstrap descriptor naming the exact loader capability and activation.
Sibling route Resources are excluded from each selected route publication.

## Node execution

`presolve deploy node --prepare` consumes only compiler-published plans and the
semantic-package runtime module table. Vite bundles each admitted named export
into `.presolve/node/presolve.route-loaders.mjs`; Node deployment schema v3
records the bundle digest. The generated host verifies that digest before
importing the frozen registry.

For an exact dynamic route request, the host strictly decodes path segments,
constructs the ordered frozen parameter record, and invokes the registry
function with `(params, signal)`. A malformed, empty, traversal, slash,
backslash, NUL, duplicate, or unmapped parameter cannot reach package code.
Disconnect and host shutdown abort active work.

Success must decode through the exact data codec. A thrown plain value may
settle the Resource as failed only when it decodes through the exact error
codec. Unknown exceptions, extra or missing object fields, non-finite numbers,
unsupported values, and codec mismatches return the stable Node executor error
without reflecting package details.

Cache behavior is explicit:

- `no_store` performs no lookup or insertion and omits `max_age_seconds`;
- `public` requires a positive maximum age and keys only on capability and
  canonical parameters; and
- `private` requires a positive maximum age and additionally keys on a SHA-256
  digest of the request authorization and cookie partition. Its response also
  emits `Vary: Authorization, Cookie`.

Cache entries live only within the host process. Concurrent work is coalesced
only for one complete key. Each waiter owns independent cancellation; when the
last waiter disconnects, unsettled package work is aborted.

## Browser bootstrap and resume

The host injects one script-safe `presolve-resource-bootstrap` payload directly
before the compiler-issued stable or content-hashed runtime script. The payload
contains one generation-1 `ready` or `failed` value for every exact bootstrap
descriptor. The browser validates cardinality, key, lifecycle, and data/error
codec before allocating Resource slots. It never imports or invokes the server
loader.

Loaded data participates in ordinary compiler-planned dependencies. A binding
such as `this.post.data?.title` renders from the bootstrap before the runtime is
reported ready. A missing, duplicate, stale, or invalid bootstrap is a fatal
runtime diagnostic. The declared resume policy is `reload`; pending work is not
serialized or replayed.

## Completion evidence

The implemented gate has focused proof for:

1. exact TypeScript symbol/signature authority and fail-closed missing evidence;
2. deterministic schema-v2 planning and schema-v4 Resource bootstrap identity;
3. multiple-route publication isolation and mixed static/dynamic inventory;
4. deterministic registry preparation plus missing module/export failures;
5. strict request decoding, typed success/failure, and codec rejection;
6. public caching, private partitioning, no-store, disconnect, and shutdown; and
7. a real browser consuming the bootstrap and rendering Resource-backed data
   without diagnostics or a browser import of server code.
