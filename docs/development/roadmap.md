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

## Chunk 16 — OCI/container execution

Add a second serious workload execution class.

This is intended to test whether the application/environment model remains valid when workload portability is not provided by WebAssembly.

Current subdivision:

```text
16.1  container workload vocabulary + explicit workload resolution in DeploymentPlan (complete)
16.2  OCI artifact identity and `.presolved` representation (complete)
16.3  container executor/materialization boundary (complete)
16.4  local container runtime adapter
16.5  container capability transport
16.6  end-to-end portability proof + checkpoint
```

16.1 establishes `container` as Application/Environment semantics and makes resolved workloads explicit in `DeploymentPlan`.

16.2 introduces bundle format `0.2`. Component bytes remain embedded, while container workloads are identified by immutable OCI image manifest/index descriptors. Registry/tag/location information is deliberately excluded from application identity. Valid `0.1` Component bundles remain decodable.

16.3 introduces `presolve-executor-sdk`, separating immutable OCI identity, environment-specific image-source resolution, and backend execution. Node materialization now converts a resolved container workload plus `.presolved` release into backend-neutral `ContainerLaunchRequest` values. Capability-bearing container launches remain blocked until 16.5.

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
