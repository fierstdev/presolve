# Metaframework workflow

Presolve discovers conventional application topology from the project root:

```text
app/
  layout.tsx
  routes/
    index.tsx
    docs/
      getting-started.tsx
```

`presolve dev`, `presolve check`, and `presolve build` consume that topology
through the canonical compiler path. The metaframework does not maintain a
second parser, router, renderer, or artifact merger.

Use an explicit configuration only when you need the hermetic compiler
interface. Ordinary applications should start with no configuration file.

The first deployment target is Cloudflare Workers Static Assets. This is a
static metaframework deployment surface: it serves compiler-issued route and
asset tables, validates artifact digests, and rejects server-capability plans
until a dedicated capability executor exists.
