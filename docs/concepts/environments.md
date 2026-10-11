# Environments

A Presolve Environment describes execution supply.

Its canonical authored document is [`presolve.env.toml`](../reference/environment-specification.md).

## Environment model

```text
Environment
├── stable identity
├── human-facing name
├── resource capacity
├── supported workload classes
└── providers
    └── supplied capabilities
```

An Environment does not describe one application's deployment.

## Environment Specification vs Environment Inventory

Presolve separates two representations:

```text
Environment Specification
    authored, versioned declaration

        ↓ validate + normalize + enrich

Environment Inventory
    normalized resolver-facing semantic supply
```

The resolver consumes the Inventory rather than raw TOML.

This allows the authored format and the internal compatibility model to remain distinct.

## Stable identity

An environment has an opaque `EnvironmentId`.

Providers have opaque `ProviderId` values.

Names are human-facing and may change. IDs are machine identities and remain stable for the lifetime of the represented instance.

For example:

```text
memory-services  prv_aaa...
       │ rename
       ▼
local-services   prv_aaa...
```

Deleting that provider and later creating another provider with the same name represents a new instance and should produce a new ID.

The intended user experience is for Presolve tooling to generate these IDs and persist them into `presolve.env.toml`. The current parser requires IDs to already be present; automatic creation belongs to the upcoming local-development UX.

## Execution support

An environment explicitly advertises workload classes it can execute.

Today the only implemented class is:

```text
component
```

The resolver rejects application workloads whose class is not supported by the environment.

## Providers

A provider is one stable source of capability supply inside an environment.

One provider may expose multiple capabilities:

```text
provider: memory-services
├── presolve:kv/store@0.1.0
└── presolve:objects/store@0.1.0
```

Provider identity and capability identity are deliberately separate.

## Resource capacity

The environment advertises resolver-visible memory and CPU capacity.

Zero capacity is a valid declaration; for example, an environment can be valid but currently drained.

The current model is compatibility supply, not a complete multi-application reservation or scheduling system.

## What does not belong in an Environment Specification

The specification does not contain:

- application-specific bindings
- credentials or secrets
- concrete provider implementation objects
- provider SDK configuration
- infrastructure realization state

Those concerns remain at the node/provider realization boundary.

See [Runtime and materialization](../architecture/runtime-and-materialization.md).
