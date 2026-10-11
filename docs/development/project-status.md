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
| CLI Environment Specification discovery/creation | Implemented |
| CLI provider declaration and generated provider IDs | Implemented |
| `presolve env check` Environment Specification validation | Implemented |
| `presolve check` application/environment compatibility | Implemented |
| Plan-scoped local provider realization | Implemented |
| `presolve dev` single-workload local Component execution | Implemented |
| Container workload schema/resolution vocabulary | Implemented |
| Explicit resolved workloads in `DeploymentPlan` | Implemented |
| `.presolved` bundle format `0.2` | Implemented |
| OCI image descriptor workload identity | Implemented |
| Bundle `0.1` decode compatibility | Implemented |
| Backend-neutral container executor SDK | Implemented |
| OCI image source-resolution boundary | Implemented |
| Container launch-request materialization | Implemented |
| Docker CLI local container executor | Implemented |
| Docker local digest-pinned image resolver | Implemented |
| Container HTTP/JSON capability bridge `0.1` | Implemented |
| Container KV/object provider routing | Implemented |
| Authenticated container capability discovery | Implemented |

## Next

Chunk 15 local developer experience is complete for the current WebAssembly Component/Rust authoring path.

OCI/container workload execution is in progress. The Application Contract, Environment Specification, resolver, `DeploymentPlan`, and `.presolved` bundle now preserve container workload identity. The executor/materialization boundary is implemented. The local Docker container adapter and initial container capability transport are implemented. The next step is the end-to-end OCI portability proof and Chunk 16 checkpoint.

## Planned after OCI/container execution

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
