# Presolve Documentation

Presolve is an application portability and distribution layer built around one separation:

> Applications describe what they are and what they need. Environments describe what they can provide. Presolve resolves the two without making deployment choices part of application identity.

Presolve is pre-stable. These documents distinguish implemented behavior from experimental developer experience and longer-term direction.

## Website documentation

[`presolve.dev/docs/`](https://presolve.dev/docs/) is the primary human-readable documentation surface. It presents the current model with worked examples, diagrams, capability matrices, and proof visualizations.

The Markdown documents in this directory remain the canonical implementation/reference and audit trail. Website pages summarize and expand these sources rather than replacing their role in repository history and review.

## Start here

If you are new to Presolve:

1. Read [Application portability](concepts/application-portability.md).
2. Read [Applications](concepts/applications.md) and [Environments](concepts/environments.md).
3. Read [Capabilities and providers](concepts/capabilities-and-providers.md).
4. Read [Resolution and deployment](concepts/resolution-and-deployment.md).
5. Use the [Application Contract](reference/application-contract.md) and [Environment Specification](reference/environment-specification.md) references when authoring Presolve documents.
6. Review the proof documents to see which portability claims have executable evidence.

The local developer workflow is implemented. See [Getting started](getting-started/README.md) for the current CLI surface and its pre-stable boundaries.

## Documentation map

### Getting started

- [Getting started](getting-started/README.md) — current local CLI workflow, compatibility checking, planning, and local execution.

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
- [CLI JSON output](reference/cli-json.md) — versioned machine-readable application, release, compatibility, and plan projections.

### Capability architecture

- [Capability contracts](architecture/capability-contracts.md) — canonical semantic capability IR and bindings.
- [SQL capability semantics](architecture/sql-capability.md) — relational execution, result, transaction, and dialect portability boundary.
- [Work-queue capability semantics](architecture/work-queue-capability.md) — at-least-once leased delivery, settlement ownership, retry, and trigger-topology boundary.

Current built-in semantic contracts are:

```text
presolve:kv/store@0.1.0
presolve:objects/store@0.1.0
presolve:sql/database@0.1.0
presolve:queue/work-queue@0.1.0
```

### Proofs

- [Flagship portability walkthrough](proofs/flagship-portability-walkthrough.md) — KV memory/Redis/Cloudflare proof and central immutable-release invariant.
- [Object-storage portability](proofs/object-storage-portability.md) — memory/S3-compatible substitution and discovered semantic boundaries.
- [SQLite/PostgreSQL portability](proofs/sqlite-postgres-portability.md) — shared SQL conformance plus same-release execution through two engines.
- [Work-queue capability proof](proofs/work-queue-capability-proof.md) — leased-delivery conformance and detached-release execution through the current reference provider.
- [Live Cloudflare realization](proofs/chunk-21-3b-live-cloudflare.md) — recorded live Workers KV realization evidence.

### Architecture

- [Architecture overview](architecture/overview.md) — the complete application-to-runtime model.
- [Runtime and materialization](architecture/runtime-and-materialization.md) — symbolic plans, provider IDs, node assembly, and runtime isolation.
- [TypeScript authoring](architecture/typescript-authoring.md) — transport-neutral SDK semantics and execution-target adapters.
- [Provider realization](architecture/provider-realization.md) — node-side adapter configuration, ProviderId joining, and secret-resolution boundary.
- [Build and package workflow](architecture/build-and-package.md) — environment-independent source/artifact packaging into verified `.presolved` releases.
- [Inspection and planning UX](architecture/inspection-and-planning.md) — semantic application inspection, compatibility, and exact plan surfaces.

### Development

- [Project status](development/project-status.md) — implemented, next, and planned surfaces.
- [Roadmap](development/roadmap.md) — current sequencing of major implementation work.
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
