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

HMR classification and production-output hooks still require their own compiler
products and will be added as later tracked slices.

## Development server

`startPresolveDevServer` is the implementation seam for `presolve dev`: it
starts and closes the Vite lifecycle, adds the compiler-product plugin, and
publishes a versioned combined diagnostic product over Vite's custom WebSocket
event channel. TypeScript and Presolve diagnostics retain their original
records with an explicit authority label; the adapter does not reinterpret
either diagnostic vocabulary.

The required `requestHost` is a compiler-owned transport callback. It receives
each request first and either returns a complete response for a document,
route, loader, or server action, or returns `undefined` to delegate the request
to Vite middleware for JavaScript, CSS, and assets. Endpoint classification is
therefore not invented by the Vite package. Vite's native watcher remains live
across ordinary asset edits; semantic HMR classification is a later slice.

## Production build

`buildPresolveProduction` runs Vite for one explicit compiler artifact and
requires an explicit output directory. It leaves existing output files intact,
writes Vite's physical manifest, then returns a versioned Presolve product that
maps the manifest entry back to the publication manifest's stable
`entry_component_id`. Physical Vite filenames remain output metadata, never
semantic identity.

The workspace keeps `esbuild` build scripts disabled while this skeleton does
not execute Vite. A later slice that starts a Vite process must make and test
an explicit build-tool approval decision.
