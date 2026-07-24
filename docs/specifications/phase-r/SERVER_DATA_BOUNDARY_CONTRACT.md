# Phase R server and data boundary contract

## Authority

Server work is a compiler-issued handoff, not a second Presolve runtime. A
host receives only validated route selection, immutable package capability
coordinates, typed serialized inputs, and a closed response/cache policy. It
does not parse application TypeScript, derive routes, inspect package source,
or infer data dependencies.

## R6-A: route request context

`resolve_file_route_request_match_v1` is the canonical request-selection
product. It retains the compiler-selected route path, page component identity,
and named dynamic path segments alongside the exact artifact/redirect target.
Static segments take precedence over dynamic segments exactly as file-route
publication already specifies.

Parameters are non-empty, raw request path segments. They remain
percent-encoded in this first product; query parsing, header normalization,
body decoding, cookies, and URL decoding are not silently performed by a host
or framework layer. A future normalization product must define them before a
loader or action can use them.

## Loader model

The public authoring form will be a field declaration, rather than a route
module callback:

```tsx
@loader("loadPost")
post!: Resource<Post, NotFound>;
```

The designator must resolve to an imported semantic-package `resource` export
whose endpoint is `server` or `shared` and whose published contract explicitly
admits the canonical route-request input record. The compiler owns the loader
identity from route, component, field, endpoint coordinate, and response
codec. It issues a route data plan containing only that identity, exact package
version/integrity/module/export, parameter schema, JSON result/error codec,
cache policy, and failure policy.

There is no arbitrary `async` route callback, `fetch` in render, module-level
loader, source inspection of package implementation, ambient request object,
or framework data cache. Client Resources retain their existing browser
activation contract; a route loader is a distinct server handoff.

## Server actions

The public form will be an explicitly marked Action method:

```tsx
@action()
@serverAction("savePost")
save(): void {}
```

It is legal only for an otherwise empty, compiler-valid Action. The designator
must select a published server-action capability from an imported semantic
package. Form submission binds its existing compiler-owned Form data record;
non-form actions use an explicit later input schema. The generated action
handoff contains an anti-confusion action identity, package coordinate,
validated request/input codec, response/error policy, invalidation targets,
and cold-resume policy. It may not receive arbitrary closures or mutate
compiler-owned state directly.

## Responses, cache, and errors

Each route data/action product declares exactly one response class:

* document — the existing static compiler artifact;
* JSON data — an integrity-bound serializable loader result;
* redirect — a compiler-validated internal route target or explicit external
  location; or
* typed failure — a status class and serializable error record.

Caching is declared in the semantic-package capability contract as `no-store`,
`private`, or `public(maxAgeSeconds)`. It is emitted as immutable policy facts;
an adapter may honor but never broaden it. Error boundaries are compiler-owned
route/layout selections, not catch-all JavaScript callbacks. An unhandled
loader/action failure returns the declared failure product and must not render
partially evaluated application output.

## Admission order and exclusions

R6 proceeds in this order: route request context; package capability schema;
loader identity/plan; server-action identity/plan; response/cache/error plans;
then adapter execution evidence. Every stage fails closed on a missing exact
capability, schema, codec, or route selection.

R6 excludes arbitrary server TypeScript, middleware, database abstractions,
sessions, cookies, environment-variable reads, streaming, generic SSR,
implicit revalidation, and a server-side component renderer. Those require
separate contracts rather than becoming escape hatches.
