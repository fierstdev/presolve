<p align="center">
  <picture>
    <source
      media="(prefers-color-scheme: dark)"
      srcset="./assets/brand/presolve-lockup-dark.svg"
    >
    <source
      media="(prefers-color-scheme: light)"
      srcset="./assets/brand/presolve-lockup-light.svg"
    >
    <img
      alt="Presolve"
      src="./assets/brand/presolve-lockup-light.svg"
      width="360"
    >
  </picture>
</p>

<p align="center">
  <strong>Build software once. Decide where it runs later.</strong>
</p>

Presolve is an open application portability and distribution layer.

It separates **what an application is** from **where and how that application is realized**.

Instead of embedding deployment assumptions into application code, a Presolve application describes its workloads, internal topology, capabilities, resource requirements, and constraints. An environment describes what it can provide. Presolve determines whether the application can run there and produces the plan needed to realize it.

```text
Application
    │
    │ declares semantics and requirements
    ▼
Application Contract
    │
    ▼
Application Release (.presolved)
    │
    ├───────────────┐
    ▼               ▼
Environment     Resolver
Inventory           │
                    ▼
              Deployment Plan
                    │
                    ▼
               Presolve Node
                    │
             ┌──────┴──────┐
             ▼             ▼
          Runtime       Providers
```

The central invariant is:

> **An application release has immutable identity independent of the environment used to realize it.**

## Why Presolve?

Application software often accumulates assumptions about where it will run:

- a particular cloud provider
- a particular database service
- Kubernetes
- serverless infrastructure
- a private network
- an on-premises deployment
- a specific runtime
- environment-specific configuration and integration code

Those choices become difficult to reverse once they leak into application architecture.

Presolve introduces a compatibility boundary between the application and its execution environment.

An application asks for abstract capabilities:

```text
application
├── SQL database
├── key-value storage
├── object storage
├── secrets
├── compute resources
└── network access
```

An environment advertises what it can provide:

```text
environment
├── provider A → SQL
├── provider B → KV
├── provider C → objects
└── available compute/resources
```

The resolver determines whether the environment can satisfy the application and explains why.

The application does not need to know which vendor or infrastructure implementation was selected.

## Application model

A Presolve application can contain multiple workloads.

```text
Application
├── Workloads
│   ├── api
│   └── worker
├── Interfaces
├── Relationships
├── Capabilities
├── Resource requirements
└── Network policy
```

For example:

```toml
contract_version = "0.1"

[application]
name = "commerce"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "component"

[[interfaces]]
name = "jobs"
kind = "request_response"

[[relationships]]
from = "api"
to = "worker"
interface = "jobs"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 128
cpu_millis = 250

[network]
outbound = "deny"
```

This describes application semantics:

```text
commerce@1.0.0

api ──jobs──▶ worker

requires:
  presolve:kv/store ^0.1
  memory >= 128 MiB
  CPU >= 250m
  outbound network denied
```

It does **not** specify whether the application runs on a developer laptop, in a cloud account, on Kubernetes, inside a private VPC, or on some future Presolve-compatible environment.

Those are realization decisions.

## Workloads, capabilities, and interfaces

Presolve deliberately separates three concepts.

### Workloads

A workload is one executable constituent of an application.

The currently implemented execution class is:

- WebAssembly Component

Presolve's application model is designed so additional workload classes can be added without changing application identity semantics. Future execution classes may include OCI containers, native executables, static assets, and other software artifacts.

### Capabilities

Capabilities express dependencies between an application and its environment.

Examples include:

- SQL
- key-value storage
- object storage
- secrets
- queues
- identity
- networking
- device access

Providers satisfy capabilities.

Application code depends on the capability contract rather than a vendor-specific implementation.

### Interfaces and relationships

Interfaces describe communication inside an application.

```text
api ── jobs ──▶ worker
```

The relationship is semantic. It does not require a specific transport.

Depending on the eventual environment and deployment model, the same relationship could be realized through component linking, IPC, networking, service bindings, or another compatible mechanism.

## Application releases

Presolve packages immutable application releases as `.presolved` bundles.

A release contains application semantics and workload artifacts, but does not contain deployment-specific provider selections or infrastructure configuration.

```text
application.presolved
├── manifest.json
└── artifacts/
    ├── api.wasm
    └── worker.wasm
```

Release identity is derived from the canonical manifest.

The manifest records cryptographic digests for the artifacts it references, forming a content-addressed relationship between application semantics and executable content.

Changing an artifact or changing application semantics changes the release identity.

Repacking the same logical release does not.

## Resolution and deployment

Presolve distinguishes three first-class concepts:

```text
Application
    what the software is

Environment
    what execution resources and capabilities are available

Deployment
    one realization of an Application Release in an Environment
```

The resolver consumes application requirements and an environment inventory and produces a symbolic deployment plan.

```text
Application Contract
        +
Environment Inventory
        │
        ▼
     Resolver
        │
        ▼
Deployment Plan
```

The resolver does not instantiate providers and does not execute workloads.

A Presolve node later materializes the plan into concrete runtime and provider bindings.

## Current implementation

Presolve is in active early development.

Current version:

```text
0.0.1
```

Implemented foundations include:

- Application Contract parsing and validation
- stable Presolve diagnostics
- WebAssembly Component workloads
- Wasmtime-based execution
- fuel, memory, and deadline isolation
- capability/provider abstraction
- in-memory key-value provider
- environment inventories
- deterministic capability resolution
- symbolic deployment plans
- deployment-plan materialization
- deterministic `.presolved` bundles
- SHA-256 application release identity
- multiple workloads per application
- application-internal interfaces and topology at the contract layer
- deterministic packaging and integrity verification

The project is still pre-stable.

No API, protocol, bundle format, capability interface, or contract schema should be considered stable until explicitly documented as such.

## Design principles

Presolve development follows several architectural constraints:

1. **Application identity is independent of deployment environment.**
2. **Application, Environment, and Deployment are distinct concepts.**
3. **Applications request capabilities rather than vendors.**
4. **Providers implement capabilities.**
5. **The resolver decides compatibility and should explain its decisions.**
6. **The runtime does not depend on the resolver or a control plane.**
7. **Deployment plans remain symbolic until materialization.**
8. **Presolve applications must remain usable without a Fierst-hosted service.**
9. **Adding a new environment should not require rewriting application code.**
10. **Build-system and developer-filesystem details are not application semantics.**

A useful test for new features is:

> Does this describe the application, or does it describe one particular way of deploying the application?

If the answer is the latter, it generally does not belong in the Application Contract.

## Repository structure

```text
crates/
  presolve-core/          Shared vocabulary and foundational types
  presolve-contract/      Application Contract
  presolve-runtime/       WebAssembly Component runtime
  presolve-bundle/        Canonical application release bundles
  presolve-policy/        Deployment and security policy
  presolve-resolver/      Environment compatibility and resolution
  presolve-provider-sdk/  Provider contracts
  presolve-node/          Deployment-plan materialization
  presolve-cli/           Presolve command-line interface

providers/
  kv-memory/              In-memory key-value provider

wit/                      Versioned WebAssembly Component interfaces
examples/                 Example Presolve applications
scripts/                  Validation and end-to-end checks
```

## Project direction

WebAssembly Components are the first execution substrate because they provide a strong portable foundation. They are not intended to define the outer boundary of Presolve.

The longer-term goal is for the Presolve application model to describe software independently of its eventual execution substrate:

```text
Application
├── Component workload
├── OCI workload
├── Native workload
├── Static workload
└── ...
```

while preserving the same separation between application identity and deployment realization.

## License

Presolve is licensed under the **Apache License, Version 2.0**.

See [`LICENSE`](LICENSE).

## Project

Presolve is developed by **Fierst LLC**.

- Website: https://presolve.dev
- Repository: https://github.com/fierstdev/presolve
