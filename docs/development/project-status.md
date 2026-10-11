# Project status

Presolve is pre-stable and under active development.

This page is a concise implementation-status view, not a compatibility guarantee.

## Implemented foundations

| Area | Status |
| --- | --- |
| Core IDs, versions, digests, diagnostics | Implemented |
| Application Contract `0.1` | Implemented |
| Application workloads/topology | Implemented |
| WebAssembly Component workload execution | Implemented |
| Wasmtime runtime controls | Implemented |
| Key-value capability | Implemented |
| Object-storage capability | Implemented |
| Canonical capability semantic contracts | Implemented |
| Provider SDK boundary | Implemented |
| Provider conformance suites | Implemented |
| In-memory KV provider | Implemented |
| In-memory object provider | Implemented |
| Environment Inventory | Implemented |
| Environment Specification `0.1` | Implemented |
| Specification normalization/enrichment | Implemented |
| Workload/resource/capability resolution | Implemented |
| Required/preferred feature negotiation | Implemented |
| Symbolic `DeploymentPlan` | Implemented |
| Provider-ID-based node materialization | Implemented |
| Deterministic `.presolved` bundles | Implemented |
| Multi-workload bundle topology | Implemented |
| Adversarial bundle/specification validation | Implemented |

## Next

The next major implementation area is local developer experience.

Expected work includes:

- discovering `presolve.toml`
- discovering/creating `presolve.env.toml`
- automatic `EnvironmentId` and `ProviderId` generation
- local provider configuration
- compatibility checking UX
- local execution workflow
- clearer inspect/diagnostic surfaces

Exact command names are not yet stable.

## Planned after local DX

Current roadmap direction includes:

- OCI/container workload execution
- TypeScript application authoring
- a real cloud-provider portability proof
- higher-level build/package workflow
- richer `check`/`inspect`/`plan` UX

## Not yet a product guarantee

The following are not currently implemented as general Presolve capabilities:

- arbitrary OCI workloads
- native executable workloads
- static workload class
- production cloud-provider adapters
- stable TypeScript authoring SDK
- complete deployment lifecycle/control plane
- stable public compatibility guarantees

See [Roadmap](roadmap.md).
