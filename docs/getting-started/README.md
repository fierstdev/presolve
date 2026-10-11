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

- provider declaration and local realization
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

Add a provider with at least one exact capability:

```bash
presolve env provider add memory-services \
  --capability presolve:kv/store@0.1.0 \
  --capability presolve:objects/store@0.1.0
```

Presolve generates and persists a stable `ProviderId`.

Advertise supported optional semantic features by targeting the exact supplied capability:

```bash
presolve env provider add object-services \
  --capability presolve:objects/store@0.1.0 \
  --feature presolve:objects/store@0.1.0=range-read \
  --feature presolve:objects/store@0.1.0=user-metadata
```

The syntax is intentionally explicit:

```text
--capability INTERFACE@VERSION
--feature    INTERFACE@VERSION=FEATURE
```

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
