# Build and package workflow

Presolve separates application release construction from environment compatibility and deployment realization.

```text
presolve.toml
+ workload artifacts
        ↓
presolve build
        ↓
canonical PresolveBundle
        ↓
encode + verify
        ↓
dist/<application>.presolved
```

The build command does **not** load `presolve.env.toml`.

That separation is intentional:

```text
presolve build
    "what immutable application release is this?"

presolve check
    "can this application run in this environment?"
```

An environment must not participate in application release identity.

## Chunk 19.1

The first developer-facing build path supports the currently proven single Component workload flow.

For the existing Rust authoring convention:

```bash
presolve build
```

Presolve discovers the nearest `presolve.toml`, finds a `Cargo.toml` beside it, builds the single `cdylib` target for `wasm32-wasip2` in Cargo release mode, and packages the resulting Component.

A prebuilt language-neutral Component can be supplied explicitly:

```bash
presolve build --component build/app.wasm
```

The default release location is:

```text
dist/<application-name>.presolved
```

An explicit output path can be supplied:

```bash
presolve build --output release/orders.presolved
```

The `.presolved` extension is required.

## Release verification

The CLI constructs `PresolveBundle`, computes the canonical release digest, encodes the bundle, decodes and verifies the encoded bytes, recomputes release identity, and rejects the build if identity changes during that round trip.

The filesystem path is not application identity. The canonical bundle manifest and workload artifact content determine release identity.

## Generated output

`dist/` is generated build output and is already ignored by the repository's standard `.gitignore`.

No new configuration file is introduced by the build workflow.

## Deferred

Chunk 19.1 deliberately does not yet define the full artifact-input UX for multiple Component workloads, OCI container descriptors, mixed Component/container releases, or automatic TypeScript-to-Component compilation.

The canonical bundle already supports those release representations. Later Chunk 19 work makes them first-class CLI build inputs without changing the bundle identity model.

## Chunk 19.2: explicit multi-workload Component mapping

Multi-workload Component applications use explicit workload-to-artifact mappings:

```bash
presolve build   --component-for api=./build/api.wasm   --component-for worker=./build/worker.wasm
```

`--component-for` is repeatable. The supplied workload names must exactly cover the Component workloads declared by the Application Contract.

Presolve rejects:

- unknown workload names
- duplicate mappings
- missing mappings
- mixing `--component PATH` with `--component-for`
- using the single-workload `--component PATH` shorthand for a multi-workload application

The ordering of `--component-for` arguments is not release identity. Presolve restores Application Contract workload order before constructing the canonical bundle, and the underlying bundle representation is itself order-independent.

Automatic source building remains intentionally narrow. When exactly one Component workload exists and no explicit artifact input is supplied, the existing Rust `cdylib` release build convenience is used. Multi-workload source orchestration is not guessed; callers provide explicit artifacts.

Container workloads remain deferred to Chunk 19.3 rather than being coerced into the Component artifact path.

## Chunk 19.3: OCI/container release inputs

`presolve build` can now package immutable OCI image descriptors for container workloads.

The CLI form is:

```bash
presolve build   --oci-image-for api=manifest,sha256:<digest>,<size>
```

or, for a multi-platform OCI image index:

```bash
presolve build   --oci-image-for api=index,sha256:<digest>,<size>
```

The `manifest` and `index` tokens are CLI aliases. The canonical `.presolved` manifest stores the full OCI media type:

```text
manifest → application/vnd.oci.image.manifest.v1+json
index    → application/vnd.oci.image.index.v1+json
```

Component and container mappings compose for mixed applications:

```bash
presolve build   --component-for api=./build/api.wasm   --oci-image-for worker=manifest,sha256:<digest>,<size>
```

The union of explicit artifact mappings must exactly cover the workloads declared by `presolve.toml`, and each mapping must match the declared workload kind.

Container image bytes are not embedded in `.presolved`. The immutable OCI content descriptor is release identity. Registry repositories, tags, registry credentials, pull URLs, and source mirrors remain deployment/transport concerns and are deliberately excluded.

The build path still does not load `presolve.env.toml`.

## Chunk 19.4: TypeScript source-to-Component builds

`presolve build` now has an automated TypeScript Component authoring path.

The default single-workload convention is:

```text
package.json
tsconfig.json
src/main.ts
```

When that convention is present and `Cargo.toml` is absent, `presolve build` type-checks the project, derives the supported Component WIT world from `presolve.toml`, invokes the project-local Jco/ComponentizeJS toolchain, and packages the generated Component.

Use an explicit entry when needed:

```bash
presolve build --typescript-entry src/application.ts
```

Generated bootstrap and WIT files live under `.presolve/build/typescript/`; they are build state, not application configuration.

The generated bootstrap connects Jco guest imports to `@presolve/sdk/component-jco`. Application source remains transport-neutral and continues to use `@presolve/sdk`.

If both the Rust `Cargo.toml` convention and TypeScript `package.json` + `src/main.ts` convention are present, Presolve refuses to guess. Explicit source/artifact selection is required.

The first automatic TypeScript WIT projection covers required baseline KV and object-storage capabilities. Custom capabilities, optional capability imports, and required optional features still require a prebuilt Component.

No TypeScript-specific deployment metadata enters `.presolved`, and `presolve.env.toml` is still absent from the build path.

## Chunk 19.5: reproducibility checkpoint

Chunk 19 closes with a CLI-level reproducibility checkpoint:

```bash
./scripts/check-build-package.sh
```

The checkpoint verifies byte-identical `.presolved` output when equivalent release inputs are supplied through:

- single prebuilt Component packaging
- multi-Component mappings in different CLI orders
- immutable OCI descriptors
- mixed Component/OCI mappings in different CLI orders

For TypeScript source builds, the checkpoint runs two clean builds and verifies that Presolve regenerates the same bootstrap source, the same WIT package, and the same exported/imported Component interface. It also repackages one generated Component artifact twice and requires byte-identical `.presolved` output.

The external Jco/ComponentizeJS compiler output itself is observed rather than treated as a Presolve invariant. Public CI demonstrated that the locked toolchain can emit different Component bytes across two clean builds on the same Ubuntu runner while preserving the same Presolve-generated inputs and Component interface.

The distinction is deliberate: deterministic Presolve packaging is a Presolve invariant, while source compiler byte reproducibility belongs to the selected external toolchain. Presolve therefore guarantees deterministic packaging for equal artifact identity rather than claiming same-platform or cross-platform compiler reproducibility.
