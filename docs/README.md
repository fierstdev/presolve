# Presolve Documentation

Presolve is an application portability and distribution layer built around one separation:

> Applications describe what they are and what they need. Environments describe what they can provide. Presolve resolves the two without making deployment choices part of application identity.

Presolve is pre-stable. These documents distinguish implemented behavior from planned developer experience and longer-term direction.

## Start here

If you are new to Presolve:

1. Read [Application portability](concepts/application-portability.md).
2. Read [Applications](concepts/applications.md) and [Environments](concepts/environments.md).
3. Read [Capabilities and providers](concepts/capabilities-and-providers.md).
4. Read [Resolution and deployment](concepts/resolution-and-deployment.md).
5. Use the [Application Contract](reference/application-contract.md) and [Environment Specification](reference/environment-specification.md) references when authoring Presolve documents.

The streamlined local developer workflow is the next implementation milestone. See [Getting started](getting-started/README.md) for the current boundary.

## Documentation map

### Getting started

- [Getting started](getting-started/README.md) — current entry point and what is intentionally deferred until the local-development workflow is implemented.

### Concepts

- [Application portability](concepts/application-portability.md) — what portability means in Presolve.
- [Applications](concepts/applications.md) — application identity, workloads, topology, and requirements.
- [Environments](concepts/environments.md) — declarative execution supply and stable environment identity.
- [Capabilities and providers](concepts/capabilities-and-providers.md) — semantic dependencies and concrete implementations.
- [Resolution and deployment](concepts/resolution-and-deployment.md) — compatibility, `DeploymentPlan`, and materialization.

### Reference

- [Application Contract](reference/application-contract.md) — `presolve.toml`.
- [Environment Specification](reference/environment-specification.md) — `presolve.env.toml`.
- [`.presolved` bundle](reference/presolved-bundle.md) — immutable application releases.
- [Diagnostics](reference/diagnostics.md) — currently assigned stable diagnostic ranges and codes.

### Architecture

- [Architecture overview](architecture/overview.md) — the complete application-to-runtime model.
- [Capability contracts](architecture/capability-contracts.md) — canonical semantic capability IR and bindings.
- [Runtime and materialization](architecture/runtime-and-materialization.md) — symbolic plans, provider IDs, node assembly, and runtime isolation.

### Development

- [Project status](development/project-status.md) — implemented, next, and planned surfaces.
- [Roadmap](development/roadmap.md) — current sequencing of major implementation chunks.

## Status language

Documentation uses these terms deliberately:

- **Implemented** — present in the repository and covered by tests.
- **Experimental** — implemented but pre-stable and subject to incompatible change.
- **Planned** — intended direction but not yet an implemented product contract.
- **Possible direction** — architectural exploration, not a commitment.

No API, schema, capability contract, archive format, or command should be assumed stable until explicitly marked stable.
