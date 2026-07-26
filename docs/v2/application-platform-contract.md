# Application Platform contract

This contract incorporates the additive `08-application-platform` workstream
from the updated V2 beta specification. It turns the existing compiler and
Vite boundaries into the complete meta-framework surface that must be proven
before beta hardening.

## Ownership

Presolve owns the conventional project interpretation, file routes, layouts,
loaders, metadata, server actions, environment classification, static-export
eligibility, route publication, resumability, and provider-neutral deployment
inventory. Existing `route_graph`, `layout_composition`, `route_loader`,
`route_server_action`, `environment_ownership`, `file_route_publication`, and
`metaframework_handoff` products remain the authorities for those facts.

Vite owns development transport, CSS and asset processing, PostCSS/Tailwind
integration, physical bundles and hashes, source maps, and output manifests.
`@presolve/vite` may only transport compiler products and associate compiler
identities with Vite output; it must not derive route, environment, or static
eligibility semantics from filenames or Vite modules.

## Conventional layout

The beta scaffold will establish `app/routes`, `app/components`, `server`,
`styles`, `assets`, `public`, and `tests` as the complete conventional project
layout. Only `app/routes` is semantic routing input by default. `app/components`
is ordinary TypeScript source, `server` is server-owned source subject to the
existing environment analysis, and `styles`, `assets`, and `public` are Vite
inputs. No new parallel route table, environment resolver, or asset manifest
may be created.

## Environment and deployment

The compiler recognizes only `PRESOLVE_PUBLIC_*` values as browser-eligible;
all other environment values remain server-owned and must be rejected on a
browser path by the environment-ownership authority. The project-level
environment loader and its provenance are a later explicit product; this rule
does not authorize ambient environment reads in compiler analysis.

Node deployment is required for beta. `presolve deploy node --prepare` now
emits a compiler-derived release inventory and static host. Static export is
permitted only for a route whose exact compiler loader and server-action
handoffs are empty; routes requiring either handoff are marked `node`. The
host rejects those server-bound routes until a capability-specific executor is
published. Provider adapters consume the compiler's immutable release inventory
and must not reconstruct route or artifact identity.

## Vite styles and assets

`buildPresolveProduction` now accepts explicit Vite entries in addition to its
compiler virtual entry. CSS, CSS Modules, PostCSS/Tailwind configuration,
imported fonts/media, and `public` assets remain Vite inputs with physical
output identities only. The adapter records the Vite manifest output but gives
no compiler identity to those entries; it preserves a compiler identity only
for the compiler-selected virtual entry.

## Testing and proof

Vitest and Playwright are Vite-hosted integrations, not semantic authorities.
`@presolve/testing` now projects immutable Vitest configuration and Playwright
project metadata from the existing Vite compiler-product plugin. Their callers
execute published applications and report declared fixtures, while compiler
diagnostics and artifact comparisons remain the source of truth. The
Application Platform gate requires route/layout, CSS/assets, public and
server-environment isolation, HMR, Node deployment, static eligibility, a
scaffold snapshot, and representative cold/resume applications.

`just application-platform-check` is the executable beta-gate entrypoint. It
combines the Vite transport/assets/HMR smoke proof, the scaffold proof, the
representative production-build corpus, the environment and Node/static-host
ergonomic fixtures, and the decorator-free cold/resume browser fixture. It
does not replace the underlying fixtures: each remains the authority for its
own product assertion.
