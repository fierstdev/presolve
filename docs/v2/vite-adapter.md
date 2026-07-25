# Vite adapter boundary

`@presolve/vite` is the required external adapter for Vite integration. Its
first schema is intentionally empty: it accepts only a compiler-produced
application-publication manifest with schema version `1` and compiler contract
`presolve-application-publication:1`.

The package exposes the `presolve:compiler-products` Vite plugin identity and
an immutable adapter API carrying the compiler contract and workspace snapshot
identity. It does not parse source, call TypeScript, infer framework semantics,
or generate artifact contents.

## Virtual modules

Schema version `1` maps every manifest artifact to the versioned public ID
`virtual:presolve/v1/<artifact-path>` and Vite's corresponding internal ID.
The host supplies exact artifact bytes through `readArtifact`; the adapter
recomputes SHA-256 and rejects bytes that do not match the compiler manifest.
The resulting module exports the artifact path and exact UTF-8 content. The
registry therefore has no independent content authority.

Dev-server, HMR, and production-output hooks still require their own compiler
products and will be added as later tracked slices.

The workspace keeps `esbuild` build scripts disabled while this skeleton does
not execute Vite. A later slice that starts a Vite process must make and test
an explicit build-tool approval decision.
