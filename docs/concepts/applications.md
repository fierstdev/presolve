# Applications

A Presolve Application is the environment-independent description of a software product release.

The canonical authored document is [`presolve.toml`](../reference/application-contract.md).

## Application model

```text
Application
├── metadata
├── workloads
├── interfaces
├── relationships
├── capability requirements
├── resource requirements
└── network policy
```

These fields describe application semantics rather than one deployment.

## Identity

Application metadata contains a stable application name and product version.

Packaging converts validated application semantics and workload artifacts into an immutable [`.presolved` release](../reference/presolved-bundle.md).

Release identity is derived from the canonical bundle manifest. Workload artifact bytes participate transitively through their recorded content digests.

Changing application semantics or a workload artifact therefore changes release identity.

Changing the environment does not.

## Workloads

A workload is one logical executable constituent of an application.

The currently implemented workload kind is:

```text
component
```

which represents a WebAssembly Component workload.

A workload kind identifies required execution semantics. It does not identify a cloud, host, or provider.

## Internal interfaces

Application-internal interfaces describe logical communication semantics between workloads.

Implemented interface kinds are:

```text
request_response
event
```

They intentionally do not specify HTTP, TCP, IPC, component linking, or another transport.

## Relationships

A relationship connects two distinct workloads through one declared application-local interface:

```text
api ── jobs ──▶ worker
```

The relationship belongs to application topology.

How the relationship is realized belongs to deployment.

## Capability requirements

Capabilities describe services the application expects from its environment.

A requirement can express:

- capability interface identity
- compatible semantic version range
- whether the capability is optional
- required semantic features
- preferred semantic features

Required features are compatibility constraints. Preferred features rank otherwise-compatible providers.

See [Capabilities and providers](capabilities-and-providers.md).

## Resources and network policy

The Application Contract can declare minimum memory and CPU requirements and an outbound network policy.

These are application requirements. They are compared with environment supply during resolution.

## What does not belong in an Application

The Application Contract should not contain:

- environment IDs
- provider IDs
- cloud resource names
- credentials or secrets
- provider endpoints
- deployment placement
- build-system filesystem paths
- a selected concrete provider

A useful design test is:

> Does this describe the application, or one particular way of realizing the application?

If it is realization-specific, it generally belongs outside the Application Contract.
