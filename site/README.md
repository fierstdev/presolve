# Presolve website

Static source for `presolve.dev`.

The site intentionally has no application framework or build step. Cloudflare Workers Static Assets serves the `site/` directory directly through the repository-root `wrangler.jsonc`.

## Documentation policy

`presolve.dev/docs/` is the primary human-readable documentation surface.

Repository Markdown under `docs/` remains the canonical implementation/reference and audit trail, but normal documentation navigation should stay on `presolve.dev`. Website pages should summarize, expand, and visualize the current canonical model and then use GitHub links as secondary source/audit links rather than the main reading path.

Current first-class documentation routes include:

```text
/docs/
/docs/model/
/docs/application/
/docs/environment/
/docs/releases/
/docs/capabilities/
/docs/cli/
/docs/proofs/
```

When capability semantics or public proof evidence changes, update both the canonical repository documentation/code and the corresponding website page in the same development phase.

`scripts/check-docs.py` validates both repository Markdown links and the local `href`/`src` graph across the static site. The existing Architecture CI job therefore rejects broken documentation routes or missing site assets.

## Local preview

From the repository root:

```bash
python3 -m http.server 8788 --directory site
```

Then open `http://localhost:8788`.

## Cloudflare Workers

Recommended Git deployment settings:

```text
Repository:      fierstdev/presolve
Production branch: main
Path:            /
Build command:   (none)
Deploy command:  npx wrangler@latest deploy
Preview command: npx wrangler@latest versions upload
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
