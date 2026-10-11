# Roadmap

Presolve development is organized around proving architectural boundaries before adding broad product surface.

Roadmap order may change as implementation reveals new constraints.

## Completed foundation and local developer workflow

Chunks 1–15 establish the current portability core and its first usable local developer workflow:

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
15  local developer experience
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

## Chunk 15 — local developer experience (complete)

Implemented:

- `presolve env init`, `show`, and `check`
- generated stable `EnvironmentId` and `ProviderId` values
- environment-scoped provider add/list/remove commands
- ergonomic built-in capability shorthand with exact canonical persistence
- nearest-file discovery for `presolve.toml` and `presolve.env.toml`
- `presolve check`
- actionable PS1xxx/PS2xxx CLI diagnostics
- plan-scoped local in-memory provider realization
- `presolve dev` for the current single-workload Rust Component path
- `--component` escape hatch for prebuilt language-neutral Component artifacts

The initial local workflow is intentionally narrow where broader semantics have not yet been designed. Multi-workload build orchestration remains deferred to the later build/package workflow.

## Chunk 16 — OCI/container execution (complete)

Add a second serious workload execution class.

This is intended to test whether the application/environment model remains valid when workload portability is not provided by WebAssembly.

Current subdivision:

```text
16.1  container workload vocabulary + explicit workload resolution in DeploymentPlan (complete)
16.2  OCI artifact identity and `.presolved` representation (complete)
16.3  container executor/materialization boundary (complete)
16.4  local container runtime adapter (complete)
16.5  container capability transport (complete)
16.6  end-to-end portability proof + checkpoint (complete)
```

16.1 establishes `container` as Application/Environment semantics and makes resolved workloads explicit in `DeploymentPlan`.

16.2 introduces bundle format `0.2`. Component bytes remain embedded, while container workloads are identified by immutable OCI image manifest/index descriptors. Registry/tag/location information is deliberately excluded from application identity. Valid `0.1` Component bundles remain decodable.

16.3 introduces `presolve-executor-sdk`, separating immutable OCI identity, environment-specific image-source resolution, and backend execution. Node materialization now converts a resolved container workload plus `.presolved` release into backend-neutral `ContainerLaunchRequest` values. Capability-bearing container launches remain blocked until 16.5.

16.4 adds `presolve-executor-docker`, the first concrete local executor. It resolves only digest-pinned images already present in the active Docker daemon, launches with `--pull=never`, maps current CPU/memory and deny/allow-all network policy, and rejects outbound allow lists rather than weakening them.

16.5 adds an authenticated HTTP/JSON capability bridge as a second transport projection of the existing semantic capability contracts. The node gateway routes exact resolved ProviderId bindings to KV/object provider traits, and container launch requests carry only bridge discovery/authentication data rather than provider credentials.

16.6 closes the architectural proof with a real OCI workload. The checkpoint resolves a container application with KV, packages an immutable OCI descriptor into `.presolved`, realizes the exact selected provider, launches the image through the Docker `ContainerExecutor`, performs a capability call from inside the container, and verifies the resulting provider state on the host. See [Chunk 16 checkpoint](chunk-16-checkpoint.md).

## Chunk 17 — TypeScript authoring (complete)

Introduce a productive TypeScript-facing application authoring path while preserving the existing semantic boundaries.

This should fit under existing TypeScript tooling rather than requiring Presolve to replace the ecosystem.

Current subdivision:

```text
17.1  TypeScript workspace + transport-neutral SDK kernel (complete)
17.2  typed KV/object clients + container bridge adapter (complete)
17.3  real TypeScript OCI workload proof (complete)
17.4  execution-target adapter boundary / Component projection (complete)
17.5  TypeScript authoring checkpoint + docs (complete)
```

17.1 establishes `@presolve/sdk` without introducing a second Application Contract. `presolve.toml` remains the canonical application declaration, while the SDK defines a transport-neutral capability call boundary that later execution adapters can project onto HTTP/JSON, WIT, or other compatible bindings.

17.2 adds typed functional KV/object clients and the `@presolve/sdk/container` adapter for the authenticated HTTP/JSON capability bridge.

17.3 proves those abstractions from a real TypeScript OCI workload. The workload calls `kv.set(...)` through `@presolve/sdk`, the container adapter projects the call onto the bridge, and the existing OCI checkpoint observes the write in the exact KV provider selected by `DeploymentPlan`.

17.4 adds `@presolve/sdk/component`. It projects the same typed KV/object APIs onto normalized direct Component imports matching Presolve's existing WIT semantics, including handle-based object streaming. The adapter boundary intentionally avoids choosing a TypeScript-to-Component compiler; generated binding glue remains a later build/package concern.

17.5 closes the TypeScript authoring chunk with a reproducible SDK checkpoint and explicit status boundaries. The real OCI path is proven end to end; the Component projection is proven at the adapter/WIT boundary; automated TypeScript-to-Component compilation remains part of the later build/package workflow. See [Chunk 17 checkpoint](chunk-17-checkpoint.md).

## Chunk 18 — real cloud provider proof (complete)

Implement at least one real provider/environment path outside the in-memory/local proof.

The goal is to validate portability against infrastructure Presolve does not control.

Current subdivision:

```text
18.1  provider realization profile + secret-resolution boundary (complete)
18.2  first real cloud adapter: Cloudflare Workers KV (complete)
18.3  adapter registry + plan-scoped cloud materialization (complete)
18.4  live cloud portability proof (complete)
18.5  cloud-provider checkpoint + docs (complete)
```

18.1 preserves the existing Environment Specification boundary: provider credentials, endpoints, SDK configuration, and adapter identity remain outside resolver-facing environment supply. A separate node-side `ProviderRealizationProfile` maps stable `ProviderId` values to adapter identity, non-secret settings, and secret references. No persisted node configuration format is locked yet.

18.2 adds the first real managed-infrastructure adapter, `presolve-provider-kv-cloudflare`, against the Cloudflare Workers KV REST API. The adapter preserves provider-neutral application semantics and keeps account/namespace/token configuration at the realization boundary. It also exposes an important portability question: semantic API compatibility does not imply identical consistency or operational characteristics, so the adapter is not falsely labeled as passing the current immediate-consistency conformance harness.

18.3 adds an installed `ProviderAdapterRegistry` and plan-scoped node materialization. The exact `ProviderId` selected by `DeploymentPlan` is joined to its realization profile, adapter, secret resolver, and finally the concrete `ProviderRegistry` implementation. Unselected provider profiles are never instantiated or secret-resolved.

18.4 adds an opt-in live checkpoint against a real Cloudflare Workers KV namespace. The proof resolves a provider-neutral KV requirement, materializes the exact selected `ProviderId` through the Cloudflare adapter, writes and reads a unique live key with a bounded eventual-consistency poll, and deletes the proof key. Credentials remain process-local realization inputs and are never written to Presolve application/environment files.

18.5 closes the cloud-provider proof with a reproducible non-live checkpoint and explicit architectural conclusions. Chunk 18 does not add another required Presolve configuration file and does not prematurely standardize provider consistency/latency/quota vocabulary after observing only one external provider. See [Chunk 18 checkpoint](chunk-18-checkpoint.md).

## Chunk 19 — build/package workflow (complete)

Make creation of `.presolved` releases a first-class developer workflow around the already implemented canonical bundle format.

Current subdivision:

```text
19.1  environment-independent `presolve build` + single Component packaging (complete)
19.2  explicit multi-workload artifact mapping (complete)
19.3  OCI/container release inputs (complete)
19.4  TypeScript source-to-Component build integration (complete)
19.5  reproducibility checkpoint + docs (complete)
```

19.1 adds the first public release-construction command. `presolve build` discovers only the Application Contract, builds the current Rust Component authoring path in release mode (or accepts `--component PATH`), verifies canonical bundle round-trip identity, and writes `dist/<application>.presolved` by default. Environment discovery is deliberately absent from the build path.

19.2 adds repeatable explicit Component artifact mapping through `--component-for WORKLOAD=PATH`. Multi-workload builds require exact coverage of declared Component workloads; unknown, duplicate, and missing mappings fail. Mapping order does not affect release identity, and the existing `--component PATH` single-workload shorthand remains compatible.

19.3 adds immutable OCI descriptor inputs through `--oci-image-for WORKLOAD=TYPE,DIGEST,SIZE`. Container-only and mixed Component/container releases now flow through the same canonical `PresolveBundle::from_workload_artifacts` path. Registry names, tags, credentials, and pull endpoints remain outside application release identity.

19.4 adds automatic TypeScript-to-Component builds using a project-local pinned Jco/ComponentizeJS toolchain. Presolve type-checks source, derives the supported WIT imports from the Application Contract, generates build-only bootstrap material under `.presolve/`, and packages the resulting Component through the existing canonical bundle path. Jco is an authoring adapter, not application semantics.

19.5 closes the build/package chunk with CLI-level byte reproducibility checks across Component, multi-Component, OCI, mixed, and pinned TypeScript source builds. The checkpoint distinguishes Presolve packaging determinism from the broader reproducibility properties of external source compilers. See [Chunk 19 checkpoint](chunk-19-checkpoint.md).

## Chunk 20 — check/inspect/plan UX

Expose the resolver and application/environment models as useful operator/developer inspection tools.

Current subdivision:

```text
20.1  `presolve inspect` application semantic view (complete)
20.2  `presolve plan` exact DeploymentPlan view
20.3  structured `--json` output for check/inspect/plan
20.4  immutable `.presolved` release inspection
20.5  UX checkpoint + docs
```

20.1 adds an environment-independent `presolve inspect` command. It discovers only the nearest Application Contract and renders the normalized application semantics: workloads, capabilities and feature requirements, topology, resources, and network policy. It does not load an Environment Specification or perform provider resolution.

## Longer-term directions

Possible later work includes:

- additional cloud and infrastructure providers
- more capability contracts
- additional workload classes
- generated bindings from capability semantics
- richer deployment lifecycle management
- broader language authoring support

These are directions, not committed current product contracts.
