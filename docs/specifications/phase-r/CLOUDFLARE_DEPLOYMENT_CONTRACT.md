# Phase R Cloudflare Workers deployment contract

## Authority and scope

R7's first provider adapter is Cloudflare Workers Static Assets. It projects a
successful compiler file-route publication; it does not compile source,
discover routes, merge artifacts, or add a client router. The generated Worker
is a narrow executor of the compiler-issued route table and asset paths.

`presolve deploy` defaults to the Cloudflare target. `presolve deploy
cloudflare --prepare` performs the complete local build, integrity validation,
and adapter projection without contacting Cloudflare. A normal deploy invokes
the project-local Wrangler command after those same checks.

## Generated deployment product

After a successful preparation, `.presolve/cloudflare/` contains:

* `deployment.plan.json` — schema-v1 provider, worker name, compatibility
  date, immutable release digest, declared secret names, compiler route table,
  and exact artifact digest inventory;
* `worker.mjs` — generated GET/HEAD static-asset executor; and
* `wrangler.jsonc` — a Workers Static Assets configuration using the project
  `dist/` directory, `ASSETS` binding, and `run_worker_first`.

The adapter validates every inventory digest directly from `dist/` before
preparation or upload. A changed, missing, or unsafe artifact aborts the
operation. The Worker replays static-over-dynamic route precedence, canonical
trailing-slash redirects, and route-local asset paths from the compiler product
only; it contains no component IDs, TypeScript source, application callbacks,
or framework state.

The default Worker name is a normalized project-directory name. `--name`
overrides it. The compiler release train pins a tested compatibility date;
`--compatibility-date` is an explicit override. The pinned date is part of the
plan and must advance through a verified Presolve release, not implicitly at
each deploy.

## Configuration, secrets, audit, and rollback

`--secret NAME` records only an uppercase binding name in Wrangler's required
secret declaration. It never accepts, writes, prints, or embeds a secret value.
Cloudflare validates declared secret presence at actual deploy time. No
Presolve source capability may read such a secret in R7: declaration reserves a
provider binding for a later compiler-owned server capability contract.

The immutable plan is the deployment audit record and includes the release
digest that maps one-for-one to compiler artifact bytes. The adapter does not
pretend a Presolve release digest is a Cloudflare Worker version ID. Once a
deployment exists, `presolve deploy cloudflare --rollback [version-id]`
delegates to Cloudflare's version operation using the prepared configuration.
The optional identifier is a provider-issued version ID; omitting it uses the
provider's previous-version behavior. Rollback never rewrites the compiler
inventory.

## Server-data boundary

Cloudflare's first R7 adapter serves static Presolve routes. It rejects a build
that contains a non-empty compiler loader or server-action handoff with
`PSCFL1017_SERVER_HANDOFF_EXECUTOR_UNAVAILABLE`. This is intentional: a
package-specific server executor must be a separately published compiler
capability, not an implicit generic JavaScript server hidden inside deployment.
The R6 handoffs remain exact and are ready for that follow-up executor.

## Exclusions

The initial adapter does not create Cloudflare storage resources, bind D1/KV/R2
or Durable Objects, run SSR, execute loader/action package modules, infer
domains/routes, upload secret values, or manage traffic splits. Those features
need their own closed compiler capability and provider projection contracts.
