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

<p align="center">
  <a href="https://github.com/fierstdev/presolve/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/fierstdev/presolve/actions/workflows/ci.yml/badge.svg"></a>
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
    │               Environment Specification
    │                         │
    │                         ▼
    │                  Environment Inventory
    │                         │
    └───────────────┬─────────┘
                    ▼
                 Resolver
                    │
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

```toml
[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
```

The environment declares providers that can satisfy those capabilities:

```toml
[[providers]]
id = "prv_..."
name = "memory-services"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
```

The resolver either rejects the pairing with stable diagnostics or returns a deterministic `DeploymentPlan`.

## Current state

Presolve is pre-stable. The current repository proves the core portability model end to end across WebAssembly Components, OCI/container workload identity, environment specifications, capability negotiation, provider selection, deterministic release packaging, local execution, TypeScript authoring, and a real Cloudflare Workers KV provider adapter.

The public CI workflow runs the secret-free baseline continuously: workspace Rust formatting/tests/Clippy, TypeScript authoring checks, deterministic build/package checks, inspection/planning checks, cloud-provider boundary tests, and documentation validation. Docker-backed and live-cloud execution proofs remain explicit follow-up checkpoints rather than hidden CI requirements.

See [Project status](docs/development/project-status.md) for the current implementation boundary.

## Application Contract

`presolve.toml` is the language-independent Application Contract.

Example:

```toml
contract_version = "0.1"

[application]
name = "orders"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 128
cpu_millis = 250

[network]
outbound = "deny"
```

Application code does not choose a provider, region, credential, endpoint, or deployment target.

## Environment Specification

`presolve.env.toml` describes execution supply.

Example:

```toml
format_version = "0.1"

[environment]
id = "env_..."
name = "local"

[resources]
memory_mib = 4096
cpu_millis = 4000

[execution]
workloads = ["component"]

[[providers]]
id = "prv_..."
name = "memory-services"

[[providers.capabilities]]
interface = "presolve:kv/store"
version = "0.1.0"
```

The resolver normalizes this specification into an `EnvironmentInventory` and evaluates it against the application requirements.

## Immutable application releases

`presolve build` constructs an environment-independent `.presolved` release.

```bash
presolve build
```

The same normalized Application Contract and the same workload artifact identity produce the same canonical manifest, release digest, and archive bytes.

A `.presolved` release may contain embedded WebAssembly Component artifacts and/or immutable OCI image descriptors. It does not contain target-environment bindings, provider credentials, registry credentials, runtime instance IDs, or infrastructure placement.

Inspect a release directly:

```bash
presolve inspect dist/orders.presolved
presolve inspect dist/orders.presolved --json
```

## Inspection, compatibility, and planning

The CLI separates four questions:

```text
presolve inspect
    What does this application require?

presolve build
    What immutable application release are we distributing?

presolve check
    Can this environment satisfy it?

presolve plan
    What exact symbolic selections did the resolver make?
```

Machine-readable automation surfaces are available for inspection, compatibility, and planning:

```bash
presolve inspect --json
presolve check --json
presolve plan --json
```

Rejected `check --json` and `plan --json` invocations still return non-zero after emitting structured stable problem data to stdout.

See [CLI JSON output](docs/reference/cli-json.md).

## Workloads

Current workload vocabulary includes:

- WebAssembly Components
- OCI/container image descriptors

WebAssembly Component execution is implemented end to end. OCI container execution also has an end-to-end local architectural proof through immutable release identity, backend-neutral materialization, Docker execution, and authenticated capability transport.

OCI image bytes are not embedded in `.presolved`; the release records immutable descriptor identity and resolves an environment-specific image source later.

## Capabilities

Current built-in semantic contracts include:

### Key-value

```text
presolve:kv/store@0.1.0
```

Operations:

```text
get
set
delete
```

### Object storage

```text
presolve:objects/store@0.1.0
```

Operations:

```text
get
put
delete
head
list
```

Optional features include:

```text
range-read
conditional-write
user-metadata
```

Applications can distinguish hard feature requirements from preferred features. The resolver rejects missing required semantics and ranks compatible providers using preferred features.

## Providers

Providers implement capability contracts.

Presolve currently includes:

- in-memory KV
- in-memory object storage
- Cloudflare Workers KV adapter

Provider selection is represented by stable `ProviderId` values in `DeploymentPlan`. Runtime realization joins those IDs to concrete provider instances.

The Cloudflare adapter is a real portability proof, not a production-support claim. Workers KV has operational characteristics that differ from the canonical KV conformance assumptions, including distributed eventual consistency and per-key write limits.

See [Provider realization](docs/architecture/provider-realization.md).

## TypeScript

`@presolve/sdk` provides transport-neutral application-facing capability APIs.

```ts
import { kv } from "@presolve/sdk";

await kv.set("order:42", bytes);
const value = await kv.get("order:42");
```

The application-facing API does not expose Docker, HTTP, WIT, provider IDs, credentials, or environment discovery.

Execution adapters live behind explicit package boundaries:

```text
@presolve/sdk/container
@presolve/sdk/component
```

`presolve.toml` remains the canonical Application Contract. TypeScript does not introduce a second semantic configuration file.

## Local development

Create a local environment:

```bash
presolve env init
```

Declare a provider:

```bash
presolve env provider add memory-services --provides kv
```

Check compatibility:

```bash
presolve check
```

Inspect the exact resolver output:

```bash
presolve plan
```

Run a local Component workload:

```bash
presolve dev
```

## Build examples

Build a precompiled Component:

```bash
presolve build --component app.wasm
```

Build explicit multi-workload Components:

```bash
presolve build \
  --component-for api=api.wasm \
  --component-for worker=worker.wasm
```

Build an OCI descriptor-backed workload:

```bash
presolve build \
  --oci-image-for api=manifest,sha256:<digest>,<size>
```

Build from TypeScript Component source:

```bash
presolve build --typescript-entry src/application.ts
```

Build from Rust Component source:

```bash
presolve build --cargo-package my-component
```

## Repository structure

```text
crates/
    presolve-core
    presolve-capability
    presolve-contract
    presolve-runtime
    presolve-bundle
    presolve-policy
    presolve-resolver
    presolve-provider-sdk
    presolve-provider-conformance
    presolve-provider-realization
    presolve-node
    presolve-cli
    presolve-environment
    presolve-executor-sdk
    presolve-capability-bridge

providers/
    kv-memory
    kv-cloudflare
    objects-memory

executors/
    docker

packages/
    sdk

examples/
    ...
```

## Documentation

Start with:

- [Documentation index](docs/README.md)
- [Application portability](docs/concepts/application-portability.md)
- [Applications](docs/concepts/applications.md)
- [Environments](docs/concepts/environments.md)
- [Capabilities and providers](docs/concepts/capabilities-and-providers.md)
- [Resolution and deployment](docs/concepts/resolution-and-deployment.md)
- [Application Contract](docs/reference/application-contract.md)
- [Environment Specification](docs/reference/environment-specification.md)
- [`.presolved` bundle](docs/reference/presolved-bundle.md)
- [CLI JSON output](docs/reference/cli-json.md)

## Configuration model

Normal durable root-level Presolve configuration remains intentionally small:

```text
presolve.toml
presolve.env.toml
```

Provider realization configuration belongs at the provider/node boundary rather than in another required application configuration file.

## License

Apache-2.0
