# Presolve website

Static source for `presolve.dev`.

The site intentionally has no application framework or build step. Cloudflare Pages can publish the `site/` directory directly.

## Local preview

From the repository root:

```bash
python3 -m http.server 8788 --directory site
```

Then open `http://localhost:8788`.

## Cloudflare Pages

Recommended Git deployment settings:

```text
Repository:        fierstdev/presolve
Production branch: main
Build command:     (none)
Build output:      site
```

The custom domain is `presolve.dev`.
