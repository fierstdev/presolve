# Deploy to Cloudflare

Presolve's first provider adapter targets Cloudflare Workers Static Assets.

```sh
pnpm check
pnpm build
pnpm deploy:prepare
pnpm deploy
```

`presolve deploy cloudflare --prepare` writes a deployment plan, generated
static-request Worker, and Wrangler configuration under `.presolve/cloudflare/`.
It verifies every published artifact digest before it asks Wrangler to upload.

Authenticate before deployment:

```sh
pnpm exec wrangler whoami
```

If necessary, sign in with `pnpm exec wrangler login`. Never place secrets in
source, `wrangler.jsonc`, or command arguments. The 0.1 static adapter records
only declared secret binding names and does not allow application source to
read them.

The adapter rejects non-empty server loader/action handoffs. That failure is
intentional: a future Cloudflare server executor must consume a closed compiler
capability contract rather than quietly run arbitrary application JavaScript.
