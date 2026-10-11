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

## Chunk 17 — TypeScript authoring

Introduce a productive TypeScript-facing application authoring path while preserving the existing semantic boundaries.

This should fit under existing TypeScript tooling rather than requiring Presolve to replace the ecosystem.

Current subdivision:

```text
17.1  TypeScript workspace + transport-neutral SDK kernel (complete)
17.2  typed KV/object clients + container bridge adapter (complete)
17.3  real TypeScript OCI workload proof (complete)
17.4  execution-target adapter boundary / Component projection (complete)
17.5  TypeScript authoring checkpoint + docs
```

17.1 establishes `@presolve/sdk` without introducing a second Application Contract. `presolve.toml` remains the canonical application declaration, while the SDK defines a transport-neutral capability call boundary that later execution adapters can project onto HTTP/JSON, WIT, or other compatible bindings.

17.2 adds typed functional KV/object clients and the `@presolve/sdk/container` adapter for the authenticated HTTP/JSON capability bridge.

17.3 proves those abstractions from a real TypeScript OCI workload. The workload calls `kv.set(...)` through `@presolve/sdk`, the container adapter projects the call onto the bridge, and the existing OCI checkpoint observes the write in the exact KV provider selected by `DeploymentPlan`.

17.4 adds `@presolve/sdk/component`. It projects the same typed KV/object APIs onto normalized direct Component imports matching Presolve's existing WIT semantics, including handle-based object streaming. The adapter boundary intentionally avoids choosing a TypeScript-to-Component compiler; generated binding glue remains a later build/package concern.

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
