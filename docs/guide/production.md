# Build, inspect, and deploy

During development run:

```sh
pnpm dev
```

Before committing or deploying, run:

```sh
pnpm check
pnpm build
```

The compiler publishes static HTML and only the browser artifacts required by
the admitted application. It does not hydrate an application through a generic
client renderer.

Interactive builds also publish a compiler-owned resumability manifest and its
matching browser artifacts. Read [resumability](resumability.md) before
inspecting, hosting, or diagnosing those files.

Use `presolve explain` to inspect compiler-derived application facts. Treat its
output as the explanation surface for state, actions, bindings, and artifacts;
do not infer equivalent facts from emitted JavaScript.

For Cloudflare Workers Static Assets, prepare first and upload only after the
prepared artifact inventory validates:

```sh
pnpm deploy:prepare
pnpm deploy
```

The JSON copies embedded in route documents are compact browser transports.
The matching pretty-printed files in `dist/` remain the canonical,
digest-bound compiler artifacts for inspection and deployment verification.
Do not compare their presentation whitespace; Presolve validates their schema
and value equivalence while avoiding repeated formatting bytes in every page.

See the complete [Cloudflare reference](../reference/cloudflare.md). The first
adapter is static-only and rejects executable server capabilities.
