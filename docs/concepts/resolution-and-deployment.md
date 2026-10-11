# Resolution and deployment

Resolution answers:

> Can this application run in this environment, and if so, which environment providers satisfy its requirements?

It does not execute the application.

## Inputs

The resolver receives two semantic inputs:

```text
Application Contract
    application requirements

Environment Inventory
    normalized environment supply
```

The Environment Inventory is produced from an [Environment Specification](../reference/environment-specification.md).

## Compatibility

The current resolver evaluates:

- workload execution support
- minimum memory
- minimum CPU
- capability interface identity
- capability semantic version compatibility
- required capability features
- preferred capability features

Optional capability requirements may remain unbound without rejecting the application.

## Provider selection

Required features act as filters.

Preferred features rank otherwise-compatible providers.

Provider selection remains symbolic: the resolver records the chosen provider ID and semantic binding but never obtains the provider implementation object.

## Resolution failures

The current resolver reports stable `PS2xxx` problem codes for:

```text
PS2001  missing capability
PS2002  insufficient memory
PS2003  insufficient CPU
PS2004  missing required capability features
PS2005  unsupported workload kind
```

See [Diagnostics](../reference/diagnostics.md).

## DeploymentPlan

Successful resolution produces a `DeploymentPlan`.

A plan records:

- selected environment identity
- capability bindings
- selected provider IDs and names
- exact provider capability versions
- canonical semantic contract where known
- negotiated requested features
- optional capabilities that remained unbound

A plan is environment-specific.

It is therefore deliberately not part of the immutable `.presolved` application identity.

## Materialization

A Presolve node materializes the symbolic plan:

```text
DeploymentPlan
      │
      │ ProviderId
      ▼
ProviderRegistry
      │
      ▼
concrete providers
      │
      ▼
Runtime
```

If the node does not have a concrete implementation registered under the selected provider ID, materialization fails rather than silently substituting another provider.

## Deployment

Conceptually:

```text
Application
    what the software is

Environment
    what can execute and satisfy it

DeploymentPlan
    how this application is resolved here

Deployment
    the realized execution
```

Presolve currently has the core resolution and materialization layers. Higher-level deployment lifecycle and operator UX remain future work.
