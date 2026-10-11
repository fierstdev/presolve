# Roadmap

Presolve development is organized around proving architectural boundaries before adding broad product surface.

Roadmap order may change as implementation reveals new constraints.

## Completed foundation

Chunks 1–14 establish the current portability core:

```text
1   core domain
2   Application Contract
3   WebAssembly Component execution
4   runtime sandbox/execution controls
5   KV capability/provider boundary
6   Environment Inventory
7   resolver
8   DeploymentPlan
9   plan materialization
10  canonical application bundle
11  multi-workload applications
12  application topology/interfaces
13  second serious capability + semantic capability contracts
14  Environment Specification
```

Together these prove:

```text
application semantics
        +
declarative environment supply
        ↓
semantic compatibility resolution
        ↓
symbolic deployment plan
        ↓
provider-ID-based materialization
        ↓
runtime
```

## Chunk 15 — local developer experience

Goal: make the implemented architecture usable without manually wiring library objects.

Expected scope:

- environment file discovery
- environment creation
- generated stable IDs
- local provider realization
- local application execution
- compatibility/check UX
- actionable diagnostics

The exact CLI contract should be determined by implementation rather than documented prematurely.

## Chunk 16 — OCI/container execution

Add a second serious workload execution class.

This is intended to test whether the application/environment model remains valid when workload portability is not provided by WebAssembly.

## Chunk 17 — TypeScript authoring

Introduce a productive TypeScript-facing application authoring path while preserving the existing semantic boundaries.

This should fit under existing TypeScript tooling rather than requiring Presolve to replace the ecosystem.

## Chunk 18 — real cloud provider proof

Implement at least one real provider/environment path outside the in-memory/local proof.

The goal is to validate portability against infrastructure Presolve does not control.

## Chunk 19 — build/package workflow

Make creation of `.presolved` releases a first-class developer workflow around the already implemented canonical bundle format.

## Chunk 20 — check/inspect/plan UX

Expose the resolver and application/environment models as useful operator/developer inspection tools.

## Longer-term directions

Possible later work includes:

- additional cloud and infrastructure providers
- more capability contracts
- additional workload classes
- generated bindings from capability semantics
- richer deployment lifecycle management
- broader language authoring support

These are directions, not committed current product contracts.
