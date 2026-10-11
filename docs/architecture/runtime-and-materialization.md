# Runtime and materialization

Presolve deliberately separates resolution from concrete runtime assembly.

## Symbolic resolution

The resolver sees provider descriptors:

```text
ProviderId
provider name
capability interface
exact semantic version
semantic contract
supported features
```

It does not see concrete provider implementation objects.

Successful resolution produces a `DeploymentPlan`.

## Stable provider join

Each capability binding in the plan records the selected `ProviderId`.

The node owns a `ProviderRegistry` that associates those same IDs with host-side implementations.

```text
Environment Specification
  prv_123
      │
      ▼
Environment Inventory
  prv_123
      │
      ▼
DeploymentPlan
  prv_123
      │
      ▼
ProviderRegistry
  prv_123 → concrete provider
```

The identifier is the durable join.

Names are not used as the materialization identity.

## Exact identity matters

If the plan selects:

```text
prv_A
```

and the registry only contains an otherwise-compatible implementation under:

```text
prv_B
```

materialization fails.

Presolve does not silently substitute a different provider after resolution.

## Current registry support

The node currently has concrete registry paths for:

- key-value providers
- object-storage providers

The in-memory provider implementations are used to prove end-to-end materialization.

## Runtime construction

`assemble_runtime` walks plan bindings and attaches matching concrete provider handles to the runtime builder.

Materialization can fail when:

- the plan contains a capability the node cannot materialize
- the selected provider ID has no registered implementation
- runtime construction itself fails

## Runtime responsibilities

The runtime executes workloads and exposes bound capability implementations.

It does not:

- parse Environment Specifications
- select providers
- perform resolution
- mutate application identity

This keeps the execution layer downstream of portability decisions.

## Why this boundary matters

A resolver that directly held concrete implementations would couple compatibility decisions to one host process and one implementation mechanism.

The symbolic plan boundary allows:

```text
same application semantics
+ same environment semantics
→ deterministic selection

selection
+ node-local implementation registry
→ concrete runtime
```

That separation is required for future local, cloud, cluster, or other node realizations.

## Container materialization

Container execution uses a separate materialization path rather than extending the Component runtime with backend-specific behavior.

```text
DeploymentPlan
    workload: api/container
        +
.presolved 0.2
    OCI digest + media type + size
        ↓
ContainerImageResolver
        ↓
environment-specific image source
        ↓
ContainerLaunchRequest
        ↓
ContainerExecutor
        ↓
backend instance
```

`presolve-executor-sdk` owns the backend-neutral container execution interface.

### Immutable identity vs source

`ContainerImageDescriptor` is application release identity:

```text
media type
sha256 digest
size
```

`ContainerImageSource` is environment-specific realization data. It may identify a registry path, local image-store reference, cloud artifact locator, or another backend-specific source.

A source may change without changing the application release as long as it still resolves to the same immutable descriptor.

The executor contract requires a backend to verify that located content matches the immutable descriptor before launch.

### Launch request

A materialized `ContainerLaunchRequest` carries:

- workload name
- immutable OCI descriptor
- environment-specific image source
- application resource requirements
- application outbound-network policy

It does not contain provider credentials or registry credentials.

### Capability transport boundary

Container capability transport is intentionally not guessed in 16.3.

If a resolved plan contains capability bindings and also requires container materialization, the current node fails explicitly. Chunk 16.5 will define how capability bindings cross the container boundary.

This prevents a container from being launched successfully while silently lacking capabilities the resolver promised to the application.
