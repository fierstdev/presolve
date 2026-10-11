# Getting started

Presolve's portability foundation is implemented, but its streamlined local developer workflow is not yet complete.

This page therefore documents the artifacts and concepts a user will encounter without presenting proposed CLI commands as if they already exist.

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

## Current implementation path

The repository currently proves this flow through library APIs and integration tests:

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

The next development chunk is focused on local developer experience. It is expected to add user-facing workflows for:

- creating and discovering a local Environment Specification
- generating stable environment and provider IDs
- configuring local provider implementations
- checking application/environment compatibility
- running applications locally with minimal manual wiring

The exact command names and flags are not yet a public contract.

Until that work lands, use the reference documentation to understand the formats:

- [Application Contract](../reference/application-contract.md)
- [Environment Specification](../reference/environment-specification.md)
- [`.presolved` bundle](../reference/presolved-bundle.md)

For the underlying model, start with [Application portability](../concepts/application-portability.md).
