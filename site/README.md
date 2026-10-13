# Presolve website

Static source for `presolve.dev`.

The site intentionally has no application framework or build step. Cloudflare Workers Static Assets serves the `site/` directory directly through the repository-root `wrangler.jsonc`.

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
