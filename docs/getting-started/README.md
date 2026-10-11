# Getting started

Presolve's portability foundation is implemented, and the local developer workflow is now being built on top of it.

The first implemented CLI surface manages the local Environment Specification:

```text
presolve env init
presolve env show
```

`presolve environment` is accepted as a visible long-form alias for `presolve env`. Public documentation uses the shorter `env` form.

## The three primary artifacts

A Presolve workflow is built around three distinct artifacts:

```text
presolve.toml
    Application Contract
    what the application is and requires

presolve.env.toml
    Environment Specification
    what one environment offers

application.presolved
    immutable application release
    application semantics + workload artifacts
```

A fourth artifact is produced by resolution:

```text
DeploymentPlan
    how one application is resolved in one environment
```

A `DeploymentPlan` is not part of the immutable application release.

## Create a local environment

From the directory where you want the Environment Specification:

```bash
presolve env init
```

This creates:

```text
presolve.env.toml
```

with:

- a generated stable `EnvironmentId`
- the human-readable name `local`
- WebAssembly Component execution support
- a default local Presolve allocation budget of 1024 MiB and 1000 CPU millicores
- no providers yet

The default capacity is a Presolve allocation policy, not automatic host-resource detection.

Choose a different name or capacity when creating the environment:

```bash
presolve env init local-dev --memory-mib 4096 --cpu-millis 4000
```

Presolve refuses to overwrite an existing `presolve.env.toml`.

## Inspect the environment

Run:

```bash
presolve env show
```

Presolve searches the current directory and then each parent directory for the nearest `presolve.env.toml`.

The discovered file is parsed and validated before its identity, resources, execution support, and providers are displayed.

This upward discovery allows commands to work from nested application directories without duplicating the Environment Specification.

## Current implementation path

The repository currently proves this flow:

```text
presolve.toml
      │
      ▼
Application Contract
      │
      ├───────────────┐
      │               │
      ▼               │
.presolved            │
                      │
presolve.env.toml     │
      │               │
      ▼               │
Environment           │
Specification         │
      │               │
      ▼               │
Environment Inventory │
      │               │
      └───────┬───────┘
              ▼
           Resolver
              │
              ▼
       DeploymentPlan
              │
              ▼
        Presolve Node
              │
              ▼
           Runtime
```

## What is next

Chunk 15 is continuing with:

- local provider realization
- Application Contract discovery
- application/environment compatibility checking
- local materialization and execution
- actionable CLI diagnostics

Commands are documented here only after they are implemented.

Reference documentation:

- [Application Contract](../reference/application-contract.md)
- [Environment Specification](../reference/environment-specification.md)
- [`.presolved` bundle](../reference/presolved-bundle.md)

For the underlying model, start with [Application portability](../concepts/application-portability.md).

## Declare providers

An Environment Specification describes provider **semantic supply**. It still does not contain concrete runtime implementation configuration.

For built-in Presolve capabilities, use the short form:

```bash
presolve env provider add memory-services \
  --provides kv \
  --provides objects
```

`--provides` can also use a comma-separated list:

```bash
presolve env provider add memory-services --provides kv,objects
```

Current built-in aliases are:

| Alias | Canonical capability |
| --- | --- |
| `kv` | current `presolve:kv/store` contract |
| `objects` | current `presolve:objects/store` contract |

The CLI resolves these aliases through Presolve's canonical built-in capability contracts. The generated `presolve.env.toml` still records the exact interface and semantic version.

Advertise built-in optional features with:

```bash
presolve env provider add object-services \
  --provides objects \
  --features objects=range-read,user-metadata
```

For custom capabilities or when exact control is needed, the explicit form remains available:

```bash
presolve env provider add search-services \
  --capability example:search/index@1.2.3
```

The low-level feature form also remains available:

```bash
presolve env provider add object-services \
  --capability presolve:objects/store@0.1.0 \
  --feature presolve:objects/store@0.1.0=range-read
```

The two forms compose. For example, shorthand features may target an explicitly supplied built-in capability:

```bash
presolve env provider add object-services \
  --capability presolve:objects/store@0.1.0 \
  --features objects=range-read,user-metadata
```

Presolve generates and persists a stable `ProviderId`.

Known Presolve capability versions and feature claims are checked against canonical semantic contracts before the Environment Specification is rewritten.

Unknown/custom capability interfaces may be declared without semantic feature claims. Presolve does not accept feature claims for contracts whose semantics it does not know.

List providers:

```bash
presolve env provider list
```

Remove a provider by its unique human-facing name:

```bash
presolve env provider remove memory-services
```

Removing and later re-adding a provider creates a new `ProviderId`. Renaming provider declarations is not implemented yet.

These commands mutate only `presolve.env.toml`. Concrete local provider realization remains a separate node-side concern and will be added later in Chunk 15.
