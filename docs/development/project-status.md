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
| End-to-end OCI/container execution checkpoint | Implemented |
| TypeScript SDK workspace/kernel | Implemented |
| TypeScript KV/object capability APIs | Implemented |
| TypeScript container bridge adapter | Implemented |
| Real TypeScript OCI workload proof | Implemented |
| TypeScript Component/WIT adapter boundary | Implemented |
| TypeScript authoring checkpoint | Implemented |
| Provider realization configuration/secret boundary | Implemented |
| Cloudflare Workers KV provider adapter | Implemented |
| Plan-scoped provider adapter materialization | Implemented |
| Live Cloudflare Workers KV portability proof | Implemented |
| Cloud-provider portability checkpoint | Implemented |

## Next

Chunk 15 local developer experience is complete for the current WebAssembly Component/Rust authoring path.

OCI/container architectural execution work in Chunk 16 is complete. A real OCI workload has an end-to-end local proof through Application Contract resolution, immutable `.presolved` identity, exact provider realization, authenticated capability transport, and Docker execution.

TypeScript authoring Chunk 17 is complete. `@presolve/sdk` provides transport-neutral typed KV/object APIs, with container and Component execution adapters. The OCI TypeScript path has a real end-to-end proof; the Component path is proven at the adapter/WIT boundary. `presolve.toml` remains the canonical language-independent Application Contract.

Chunk 18 real cloud-provider portability is complete. Presolve can resolve a provider-neutral KV requirement, preserve the exact selected `ProviderId` through node realization, materialize a Cloudflare Workers KV adapter, resolve secrets only for selected providers, and exercise that provider against real managed infrastructure. No additional required Presolve configuration file was introduced.

The next roadmap chunk is the first-class build/package workflow around the already implemented `.presolved` release format.

Automated TypeScript-to-Component compilation remains part of the later build/package workflow.

## Planned next

Current roadmap direction includes:

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
