# Phase R scaffold, DX, and dogfood contract

## New application path

`npm create presolve <directory>` is the normal new-app entrypoint. It creates
an `app/routes` project with a landing route and public documentation route,
a small `package.json` command surface, and a conservative `.gitignore`.
It does not create a custom build configuration, route registry, runtime
wrapper, generated component name, or deployment credentials.

The generated commands are intentionally ordinary:

* `npm run dev` → `presolve dev`;
* `npm run check` → `presolve check`;
* `npm run build` → `presolve build`;
* `npm run deploy:prepare` → local Cloudflare handoff validation; and
* `npm run deploy` → validated Cloudflare deployment through Wrangler.

The public `presolve` import is declaration-only compiler vocabulary, not an
npm semantic-package capability. It therefore needs no `presolve.contract.json`
and cannot introduce package runtime behavior.

## Explanation surfaces

`presolve explain route` reads the compiler project/discovery and validated
file-route products, then reports canonical paths, entry components, and layout
chains. It does not scan paths independently or invent a router model.

`presolve explain deployment` reads the immutable prepared Cloudflare plan and
reports the Worker name, compiler release digest, route count, and artifact
inventory count. Preparation is required first, so this command never guesses
deployment state or credentials.

## Dogfood public site

`examples/presolve-site` is the first production-shaped Presolve public site.
It has a landing page, public documentation, getting-started guide, component
and deployment pages, examples, and a comparison page. It is deliberately
authored as file-routed Presolve components with the conventional application
layout and normal `import { component } from "presolve"` source form.

The comparison page may describe architectural differences but must not make
numeric performance claims without separately checked-in workloads, environment
details, and measured results. The dogfood acceptance proof is that the
compiler builds all routes, composes the layout, emits the public content, and
prepares it for the same Cloudflare adapter available to users.

## Exclusions

This slice does not claim a package publication registry, browser-based docs
CMS, search service, analytics integration, image system, or live production
deployment credentials. The site is deployable through the same static adapter;
publishing it to an account remains a caller-authorized release operation.
