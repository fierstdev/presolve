# Chunk 19 checkpoint: build/package workflow

Chunk 19 turns `.presolved` construction from an internal bundle primitive into a developer-facing, environment-independent build workflow.

## Implemented developer workflow

The public build command is:

```bash
presolve build
```

Its responsibility is application release construction:

```text
Application Contract
+ workload source/artifact identity
        ↓
presolve build
        ↓
verified canonical PresolveBundle
        ↓
.presolved
```

It deliberately does not load `presolve.env.toml`.

Environment compatibility remains a separate `presolve check` operation.

## Supported release inputs

Chunk 19 proves these paths:

```text
Rust Component source
    Cargo.toml
    → wasm32-wasip2 release Component

TypeScript Component source
    package.json + tsconfig.json + src/main.ts
    → Jco/ComponentizeJS Component

prebuilt Component
    --component PATH

multiple Components
    --component-for WORKLOAD=PATH

OCI container
    --oci-image-for WORKLOAD=TYPE,DIGEST,SIZE

mixed Component/container release
    component mapping(s) + OCI mapping(s)
```

All paths converge on the same canonical bundle model.

No language-specific deployment model is introduced.

## Reproducibility boundary

Presolve distinguishes two separate reproducibility properties.

### Packaging determinism

Given the same normalized Application Contract and the same workload artifact identity:

```text
same inputs
    → same canonical manifest
    → same release digest
    → same .presolved archive bytes
```

This is a Presolve-controlled invariant.

The bundle encoder normalizes archive metadata and emits archive entries in stable order. Component artifact mappings are canonicalized by workload identity rather than CLI argument order. OCI images are represented by immutable content descriptors.

The Chunk 19 checkpoint exercises the CLI repeatedly for:

- one prebuilt Component
- multiple Components with reversed mapping order
- one OCI image descriptor
- a mixed Component/OCI application with reversed mapping order

Each pair must be byte-identical.

### Source-toolchain reproducibility

Source compilation occurs before canonical packaging:

```text
source
    ↓
language/compiler toolchain
    ↓
workload artifact
    ↓
Presolve packaging
```

Presolve cannot make every external compiler reproducible merely by packaging its output.

For the TypeScript checkpoint, Presolve executes two clean source builds and requires the Presolve-generated bootstrap, generated WIT package, and resulting Component WIT interface to remain identical. The emitted Component bytes are observed but are not a Presolve-controlled invariant.

Public CI demonstrated why this boundary matters: the same locked Jco/ComponentizeJS toolchain on the same Ubuntu runner emitted different Component bytes across two clean builds seconds apart, while preserving the same generated Presolve inputs and Component interface.

The checkpoint therefore does **not** claim that:

- repeated Jco/ComponentizeJS builds are byte-identical even on the same platform
- different Jco/ComponentizeJS versions are reproducible
- different Node.js versions produce identical Component bytes
- different operating systems produce identical compiler output
- every Rust toolchain produces byte-identical Wasm from identical source
- arbitrary external build systems are reproducible

Those are source-toolchain properties.

Presolve's durable guarantee is narrower and stronger: equal workload artifact identity plus equal Application Contract semantics produces equal release identity and deterministic `.presolved` encoding. The TypeScript checkpoint separately proves that Presolve generates stable compiler inputs and a stable semantic Component interface around the external toolchain.

## TypeScript build adapter boundary

Chunk 19.4 selects Jco/ComponentizeJS as the first TypeScript Component build adapter.

Application code remains:

```ts
import { kv, objects } from "@presolve/sdk";
```

Jco-specific normalization lives in:

```text
@presolve/sdk/component-jco
```

Generated bootstrap/WIT material lives under:

```text
.presolve/build/typescript/
```

Neither becomes a second Application Contract.

The initial automatic WIT projection covers required baseline KV/object capability imports. More general custom-capability binding generation remains future work.

## Configuration surface

Chunk 19 adds no new required project-root configuration file.

The normal durable authoring surface remains:

```text
presolve.toml
presolve.env.toml
```

Build-generated state belongs under `.presolve/`.

Language-native files such as `Cargo.toml`, `package.json`, and `tsconfig.json` remain language toolchain inputs, not Presolve configuration substitutes.

## Checkpoint

Run:

```bash
./scripts/check-build-package.sh
```

The checkpoint covers:

- bundle and CLI tests
- strict build/package Clippy
- TypeScript SDK tests
- explicit Component packaging reproducibility
- multi-workload mapping-order reproducibility
- OCI descriptor packaging reproducibility
- mixed workload packaging reproducibility
- clean repeated TypeScript Component source builds
- deterministic generated TypeScript bootstrap/WIT inputs
- stable TypeScript-generated Component WIT interface
- deterministic `.presolved` packaging when the same generated Component artifact is supplied twice

It does not require an Environment Specification, provider credentials, cloud access, Docker, or deployment infrastructure.

## Explicit non-claims

Chunk 19 does not establish:

- a stable public compiler-plugin ABI
- arbitrary language source auto-detection
- automatic multi-workload source orchestration
- general custom-capability WIT/binding generation
- same-platform or cross-platform source compiler byte reproducibility
- container image building or registry publishing
- deployment realization from the build command
- a stable Presolve CLI or bundle format

Those remain pre-stable or later work.

## Result

Chunk 19 establishes the developer-facing release boundary:

> Build produces an immutable application release. Environment resolution decides where that release can run.

Source languages and workload forms may differ, but they converge on the same environment-independent `.presolved` application identity.
