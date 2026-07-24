# Architecture

Presolve has one semantic authority: the compiler.

```text
TypeScript / TSX source
        ↓
compiler semantic products
        ├── static HTML and browser runtime artifacts
        ├── route and deployment handoffs
        ├── resumability products
        └── tooling query snapshots
```

The framework provides TypeScript declarations and compiler intrinsics. The
metaframework supplies conventional project discovery and provider projections.
The runtime, editor integrations, and deployment adapters consume compiler
products; none is allowed to rediscover source semantics independently.

This keeps dependency topology, component/instance identity, state storage,
DOM operations, capability scheduling, artifact integrity, and deployment
inventory in one verifiable place.
