# `.presolved` bundle reference

A `.presolved` file is Presolve's immutable application release artifact.

It captures application semantics and workload artifacts without embedding environment-specific deployment decisions.

## Current archive format

Bundle format `0.1` is encoded as a deterministic tar archive.

Its canonical manifest path is:

```text
manifest.json
```

WebAssembly Component artifacts are stored under:

```text
artifacts/<workload>.wasm
```

For example:

```text
commerce.presolved
├── manifest.json
└── artifacts/
    ├── api.wasm
    └── worker.wasm
```

Only regular files and format-defined paths are accepted by the decoder.

## Manifest

The canonical manifest contains:

- bundle format version
- application metadata
- application requirements
- workloads and artifact metadata
- application-internal interfaces
- application-internal relationships

Artifact metadata includes:

- canonical path
- content digest
- byte size
- media type

WebAssembly artifacts currently use:

```text
application/wasm
```

## Release identity

Application release identity is derived from the canonical serialization of `manifest.json`.

Artifact bytes participate transitively because the manifest records their content digests.

Therefore:

```text
change application semantics
    → release identity changes

change workload artifact bytes
    → artifact digest changes
    → manifest changes
    → release identity changes

re-encode the same logical release
    → release identity remains the same
```

Archive-container metadata is not application identity.

## Determinism

Collection ordering that is not application semantics is normalized before canonical manifest serialization.

Bundle archive metadata and entry ordering are also normalized during encoding.

Deterministic archive bytes are useful, but the release digest is intentionally derived from the canonical manifest rather than the tar container bytes.

## Verification

Bundle decoding verifies structural and semantic integrity, including:

- supported bundle version
- safe UTF-8 archive paths
- no duplicate archive entries
- no unexpected paths
- manifest topology consistency
- no duplicate workload/interface/relationship declarations
- required workload artifacts are present
- no unreferenced artifacts exist
- artifact size matches the manifest
- artifact content digest matches the manifest
- capability requirements are structurally valid

Malformed or adversarial archives are rejected.

## What a bundle does not contain

A `.presolved` release does not contain:

- selected environment ID
- selected provider IDs
- a `DeploymentPlan`
- provider credentials
- infrastructure configuration
- environment-specific endpoints

This is the central portability invariant:

> The same immutable application release can be resolved independently against different compatible environments.
