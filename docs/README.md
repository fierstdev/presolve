# Presolve Documentation

Presolve is an application portability and distribution layer built around one separation:

> Applications describe what they are and what they need. Environments describe what they can provide. Presolve resolves the two without making deployment choices part of application identity.

Presolve is pre-stable. These documents distinguish implemented behavior from experimental local developer experience and longer-term direction.

## Start here

If you are new to Presolve:

1. Read [Application portability](concepts/application-portability.md).
2. Read [Applications](concepts/applications.md) and [Environments](concepts/environments.md).
3. Read [Capabilities and providers](concepts/capabilities-and-providers.md).
4. Read [Resolution and deployment](concepts/resolution-and-deployment.md).
5. Use the [Application Contract](reference/application-contract.md) and [Environment Specification](reference/environment-specification.md) references when authoring Presolve documents.

The initial local developer workflow is implemented. See [Getting started](getting-started/README.md) for the current CLI surface and its pre-stable boundaries.

## Documentation map

### Getting started

- [Getting started](getting-started/README.md) — current local CLI workflow, compatibility checking, and local Component execution.

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
- [CLI JSON output](reference/cli-json.md) — versioned machine-readable application, release, compatibility, and plan inspection projections.

### Architecture

- [Architecture overview](architecture/overview.md) — the complete application-to-runtime model.
- [Capability contracts](architecture/capability-contracts.md) — canonical semantic capability IR and bindings.
- [Runtime and materialization](architecture/runtime-and-materialization.md) — symbolic plans, provider IDs, node assembly, and runtime isolation.
- [TypeScript authoring](architecture/typescript-authoring.md) — transport-neutral SDK semantics and execution-target adapters.
- [Provider realization](architecture/provider-realization.md) — node-side adapter configuration, ProviderId joining, and secret-resolution boundary.
- [Build and package workflow](architecture/build-and-package.md) — environment-independent source/artifact packaging into verified `.presolved` releases.
- [Inspection and planning UX](architecture/inspection-and-planning.md) — semantic application inspection and the evolving check/plan operator surfaces.

### Development

- [Project status](development/project-status.md) — implemented, next, and planned surfaces.
- [Roadmap](development/roadmap.md) — current sequencing of major implementation chunks.
- [Chunk 17 checkpoint](development/chunk-17-checkpoint.md) — TypeScript authoring proof and explicit non-claims.
- [Chunk 18 checkpoint](development/chunk-18-checkpoint.md) — real cloud-provider portability proof, configuration boundary, and operational-characteristics conclusions.
- [Chunk 19 checkpoint](development/chunk-19-checkpoint.md) — build/package workflow, deterministic release packaging, and source-toolchain reproducibility boundary.
- [Chunk 20 checkpoint](development/chunk-20-checkpoint.md) — source/release inspection, compatibility, exact planning, structured output, and rejected-resolution behavior.
- [Chunk 21 release infrastructure](development/chunk-21-release-infrastructure.md) — public CI evidence, native CLI packaging, checksum verification, and tag-gated release publication.

## Status language

Documentation uses these terms deliberately:

- **Implemented** — present in the repository and covered by tests.
- **Experimental** — implemented but pre-stable and subject to incompatible change.
- **Planned** — intended direction but not yet an implemented product contract.
- **Possible direction** — architectural exploration, not a commitment.

No API, schema, capability contract, archive format, or command should be assumed stable until explicitly marked stable.
