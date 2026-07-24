# Deploy to Cloudflare

The first Presolve provider adapter targets Cloudflare Workers Static Assets.
It deploys compiler-published static assets and verifies their immutable digest
inventory before it invokes Wrangler.

## Prerequisites

- A Presolve application that passes `pnpm check` and `pnpm build`.
- A Cloudflare account with permission to deploy the target Worker.
- Wrangler installed as the project's development dependency.

Authenticate without placing credentials in source:

```sh
pnpm exec wrangler whoami
pnpm exec wrangler login
```

Prepare and inspect the deployment locally:

```sh
pnpm deploy:prepare
```

This writes the deployment projection under `.presolve/cloudflare/`. Review it
and commit only application source/configuration, not generated `.presolve/`
or `dist/` output. Deploy when the prepared inventory is correct:

```sh
pnpm deploy
```

The adapter is static-only in this alpha. It rejects executable server loaders
and server actions rather than silently running arbitrary application
JavaScript on a Worker. Secrets are represented by declared binding names; do
not commit secret values to `.dev.vars`, `.env*`, or Wrangler configuration.

Deploy the complete compiler artifact inventory, including resumability
artifacts when an application is interactive. Do not mix generated files from
separate builds; the runtime validates matching identities before it resumes.
See [resumability](../guide/resumability.md).
