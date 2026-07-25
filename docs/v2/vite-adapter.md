# Vite adapter boundary

`@presolve/vite` is the required external adapter for Vite integration. Its
first schema is intentionally empty: it accepts only a compiler-produced
application-publication manifest with schema version `1` and compiler contract
`presolve-application-publication:1`.

The package exposes the `presolve:compiler-products` Vite plugin identity and
an immutable adapter API carrying the compiler contract and workspace snapshot
identity. It does not parse source, call TypeScript, infer framework semantics,
or generate artifact contents. It also has no virtual-module, dev-server, HMR,
or production-output hooks yet; those require their own compiler products and
will be added as later tracked slices.

The workspace keeps `esbuild` build scripts disabled while this skeleton does
not execute Vite. A later slice that starts a Vite process must make and test
an explicit build-tool approval decision.
