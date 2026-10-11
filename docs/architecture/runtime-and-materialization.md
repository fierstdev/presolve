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
