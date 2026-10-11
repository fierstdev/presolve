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
