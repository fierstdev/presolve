# Architecture overview

Presolve separates application identity from deployment realization.

## Canonical flow

```text
                         APPLICATION SIDE

                           presolve.toml
                                │
                                ▼
                      Application Contract
                                │
                                ▼
                          .presolved
                   immutable application release
                                │
                                │
             ┌──────────────────┴──────────────────┐
             │                                     │
             │                           ENVIRONMENT SIDE
             │                                     │
             │                              presolve.env.toml
             │                                     │
             │                                     ▼
             │                        Environment Specification
             │                                     │
             │                                     ▼
             │                          Environment Inventory
             │                                     │
             └──────────────────┬──────────────────┘
                                ▼
                             Resolver
                                │
                                ▼
                         DeploymentPlan
                                │
                                ▼
                          Presolve Node
                                │
                   ┌────────────┴────────────┐
                   ▼                         ▼
                Runtime                  Providers
```

## Application layer

The Application Contract describes environment-independent application semantics.

Packaging combines validated application semantics with immutable workload artifacts into `.presolved`.

The release contains no selected environment or provider.

## Environment layer

The Environment Specification describes one environment's supply:

- identity
- resource capacity
- workload execution support
- providers
- exact capability versions and supported semantic features

Semantic enrichment converts it into the resolver-facing Environment Inventory.

## Resolution layer

The resolver compares requirements with supply and produces either:

```text
ResolutionProblems
```

or:

```text
DeploymentPlan
```

The plan remains symbolic. Provider selections are represented by stable `ProviderId` values.

## Materialization layer

A Presolve node owns concrete provider implementations.

Its registry maps:

```text
ProviderId → implementation
```

Materialization converts the symbolic plan into runtime configuration.

The resolver therefore has no dependency on provider implementation objects.

## Runtime layer

The runtime executes supported workload artifacts with concrete capability bindings.

The initial execution substrate is WebAssembly Components through Wasmtime, with runtime controls including fuel, memory, and deadline isolation.

## Architectural invariants

1. Application identity is independent of deployment environment.
2. Application, Environment, and Deployment are separate concepts.
3. Applications request capability semantics, not vendors.
4. Capability semantics are independent of binding transport.
5. Resolver output remains symbolic until node materialization.
6. The runtime does not own environment resolution.
7. A `.presolved` release must not encode environment-specific provider selection.
