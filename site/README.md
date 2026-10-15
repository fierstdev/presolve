# Presolve website

Static source for `presolve.dev`.

The site intentionally has no application framework or build step. Cloudflare Workers Static Assets serves the `site/` directory directly through the repository-root `wrangler.jsonc`.

## Documentation policy

`presolve.dev/docs/` is the primary human-readable documentation surface.

Repository Markdown under `docs/` remains the canonical implementation/reference and audit trail, but normal documentation navigation should stay on `presolve.dev`. Website pages should teach the developer workflow first, expand into concepts only when needed, and use GitHub links as secondary source/audit links rather than the main reading path.

The documentation information architecture is intentionally task-oriented:

```text
Start
  /docs/                         documentation home
  /docs/getting-started/         complete first-run tutorial

Build & run
  /docs/cli/                     CLI, workflows, JSON, diagnostics
  /docs/application/             define what the application needs
  /docs/environment/             define what an environment provides

Understand
  /docs/releases/                built .presolved release identity
  /docs/capabilities/            service semantics and provider guarantees
  /docs/model/                   formal architecture and invariants

Verify
  /docs/proofs/                  reproducible portability evidence
  /status/                       implemented vs not-yet-guaranteed
```

New developers should be able to complete the getting-started tutorial without first learning Presolve's formal architecture terminology. Reference pages may use precise terms, but should introduce them from the developer-facing concept they represent.

When capability semantics or public proof evidence changes, update both the canonical repository documentation/code and the corresponding website page in the same development phase.

Proof documentation should describe the same operational path exercised by CI. Readiness checks for external providers must validate the provider-visible connection path used by conformance/execution rather than relying only on container-local health checks.

## Navigation and responsive behavior

`site/site.js` is a small progressive-enhancement layer shared by every public page. It provides:

- the mobile site menu
- grouped documentation navigation
- current-page state
- generated per-page documentation outlines
- compact mobile documentation and “On this page” menus

`site/navigation.css` contains the shared responsive navigation behavior. The original HTML navigation remains usable as a no-JavaScript fallback.

Desktop documentation uses a persistent grouped sidebar plus a generated “On this page” outline where viewport width permits. Tablet layouts retain the grouped documentation tree and collapse the page outline. Mobile layouts replace the desktop navigation with explicit touch-friendly disclosure menus rather than merely hiding links.

`scripts/check-docs.py` validates repository Markdown links, the local static-site link graph, local hash anchors, required responsive navigation assets, documentation shell structure, and sitemap coverage. The existing Architecture CI job therefore rejects broken documentation routes, missing navigation assets, stale anchors, or unsitemapped public pages.

## Local preview

From the repository root:

```bash
python3 -m http.server 8788 --directory site
```

Then open `http://localhost:8788`.

Recommended manual viewport checks before merging significant site changes:

```text
390 × 844    phone
430 × 932    large phone
768 × 1024   tablet portrait
1024 × 768   tablet landscape
1440 × 900   desktop
```

At each viewport verify that the header remains usable, no page-level horizontal overflow appears, code/table overflow is locally scrollable, documentation navigation is reachable, and the current page remains obvious.

## Cloudflare Workers

Recommended Git deployment settings:

```text
Repository:        fierstdev/presolve
Production branch: main
Path:              /
Build command:     (none)
Deploy command:    npx wrangler@latest deploy
Preview command:   npx wrangler@latest versions upload
```

`wrangler.jsonc` declares:

```json
{
  "name": "presolve",
  "assets": {
    "directory": "./site"
  }
}
```

The custom domain is `presolve.dev`.
