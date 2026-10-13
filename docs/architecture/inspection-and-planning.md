# Inspection and planning UX

Presolve separates four developer questions:

```text
presolve inspect
    What is this application?

presolve check
    Can this application run in this environment?

presolve plan
    What exact workload/provider selections would Presolve make here?

presolve build
    What immutable application release is this?
```

These operations share underlying models but have different dependency boundaries.

## Inspection boundary

`presolve inspect` is application-only.

It discovers and validates the nearest:

```text
presolve.toml
```

It does not discover or require:

```text
presolve.env.toml
```

Inspection renders the semantic Application Contract rather than merely echoing source TOML. The current human-readable view includes:

- application name, version, description, and contract version
- explicit workloads or the legacy implicit `main` Component workload
- capability requirements, optionality, and semantic feature requirements/preferences
- application-internal interfaces and relationships
- resource requirements
- outbound network policy

Environment supply, provider selection, and deployment bindings are intentionally absent. Those belong to `check` and `plan`.

## DeploymentPlan boundary

`presolve plan` is environment-dependent.

It performs the same Application Contract + Environment Specification resolution used by `presolve check`, but its successful output is centered on the exact `DeploymentPlan` rather than compatibility messaging.

The human-readable plan view includes:

- application and environment context
- the exact resolved `EnvironmentId`
- resolved workload names and execution kinds
- capability interface and exact selected provider version
- selected provider name and stable `ProviderId`
- negotiated semantic features
- the canonical semantic capability contract attached to the binding, when available
- unbound optional capabilities

For known semantic contracts, the plan view also exposes the contract's canonical operations, guarantees, and optional feature vocabulary. This metadata comes directly from the resolver's `CapabilityBinding`; the CLI does not reconstruct it.

A plan does **not** currently contain:

- workload placement
- service endpoints
- provider credentials
- provider realization adapter identity
- registry locations
- runtime process/container IDs
- infrastructure provisioning actions

`presolve plan` therefore does not display or infer those concepts.

When resolution fails, no `DeploymentPlan` exists. The command surfaces the same stable PS2xxx resolver diagnostics used by `presolve check` and exits unsuccessfully.

This preserves the distinction:

```text
check
    compatibility-oriented explanation

plan
    exact successful resolver output
```

## Chunk 20 direction

Chunk 20 incrementally exposes existing Presolve state rather than creating a second operator model:

```text
20.1  application inspection
20.2  DeploymentPlan inspection
20.3  structured JSON output
20.4  immutable `.presolved` release inspection
20.5  UX checkpoint + docs
```

Structured output will be added after the human-facing semantic boundaries are settled so machine-readable representations reflect the same model rather than driving it prematurely.
