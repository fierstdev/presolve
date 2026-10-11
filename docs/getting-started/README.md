# Getting started

Presolve's portability foundation is implemented, and the local developer workflow is now being built on top of it.

The implemented local CLI surface now includes environment management and compatibility checking:

```text
presolve env init
presolve env show
presolve check
```

`presolve environment` is accepted as a visible long-form alias for `presolve env`. Public documentation uses the shorter `env` form.

## The three primary artifacts

A Presolve workflow is built around three distinct artifacts:

```text
presolve.toml
    Application Contract
    what the application is and requires

presolve.env.toml
    Environment Specification
    what one environment offers

application.presolved
    immutable application release
    application semantics + workload artifacts
```

A fourth artifact is produced by resolution:

```text
DeploymentPlan
    how one application is resolved in one environment
```

A `DeploymentPlan` is not part of the immutable application release.

## Create a local environment

From the directory where you want the Environment Specification:

```bash
presolve env init
```

This creates:

```text
presolve.env.toml
```

with:

- a generated stable `EnvironmentId`
- the human-readable name `local`
- WebAssembly Component execution support
- a default local Presolve allocation budget of 1024 MiB and 1000 CPU millicores
- no providers yet

The default capacity is a Presolve allocation policy, not automatic host-resource detection.

Choose a different name or capacity when creating the environment:

```bash
presolve env init local-dev --memory-mib 4096 --cpu-millis 4000
```

Presolve refuses to overwrite an existing `presolve.env.toml`.

## Inspect the environment

Run:

```bash
presolve env show
```

Presolve searches the current directory and then each parent directory for the nearest `presolve.env.toml`.

The discovered file is parsed and validated before its identity, resources, execution support, and providers are displayed.

This upward discovery allows commands to work from nested application directories without duplicating the Environment Specification.

## Current implementation path

The repository currently proves this flow:

```text
presolve.toml
      │
      ▼
Application Contract
      │
      ├───────────────┐
      │               │
      ▼               │
.presolved            │
                      │
presolve.env.toml     │
      │               │
      ▼               │
Environment           │
Specification         │
      │               │
      ▼               │
Environment Inventory │
      │               │
      └───────┬───────┘
              ▼
           Resolver
              │
              ▼
       DeploymentPlan
              │
              ▼
        Presolve Node
              │
              ▼
           Runtime
```

## What is next

Chunk 15 is continuing with:

- broader actionable CLI diagnostics

Commands are documented here only after they are implemented.

Reference documentation:

- [Application Contract](../reference/application-contract.md)
- [Environment Specification](../reference/environment-specification.md)
- [`.presolved` bundle](../reference/presolved-bundle.md)

For the underlying model, start with [Application portability](../concepts/application-portability.md).

## Declare providers

An Environment Specification describes provider **semantic supply**. It still does not contain concrete runtime implementation configuration.

For built-in Presolve capabilities, use the short form:

```bash
presolve env provider add memory-services \
  --provides kv \
  --provides objects
```

`--provides` can also use a comma-separated list:

```bash
presolve env provider add memory-services --provides kv,objects
```

Current built-in aliases are:

| Alias | Canonical capability |
| --- | --- |
| `kv` | current `presolve:kv/store` contract |
| `objects` | current `presolve:objects/store` contract |

The CLI resolves these aliases through Presolve's canonical built-in capability contracts. The generated `presolve.env.toml` still records the exact interface and semantic version.

Advertise built-in optional features with:

```bash
presolve env provider add object-services \
  --provides objects \
  --features objects=range-read,user-metadata
```

For custom capabilities or when exact control is needed, the explicit form remains available:

```bash
presolve env provider add search-services \
  --capability example:search/index@1.2.3
```

The low-level feature form also remains available:

```bash
presolve env provider add object-services \
  --capability presolve:objects/store@0.1.0 \
  --feature presolve:objects/store@0.1.0=range-read
```

The two forms compose. For example, shorthand features may target an explicitly supplied built-in capability:

```bash
presolve env provider add object-services \
  --capability presolve:objects/store@0.1.0 \
  --features objects=range-read,user-metadata
```

Presolve generates and persists a stable `ProviderId`.

Known Presolve capability versions and feature claims are checked against canonical semantic contracts before the Environment Specification is rewritten.

Unknown/custom capability interfaces may be declared without semantic feature claims. Presolve does not accept feature claims for contracts whose semantics it does not know.

List providers:

```bash
presolve env provider list
```

Remove a provider by its unique human-facing name:

```bash
presolve env provider remove memory-services
```

Removing and later re-adding a provider creates a new `ProviderId`. Renaming provider declarations is not implemented yet.

These commands mutate only `presolve.env.toml`. Concrete local provider realization remains a separate node-side concern and will be added later in Chunk 15.

## Check application compatibility

From an application directory or any descendant, run:

```bash
presolve check
```

Presolve independently discovers the nearest:

```text
presolve.toml
presolve.env.toml
```

by walking upward from the current directory.

The Application Contract is parsed and validated. The Environment Specification is parsed, normalized into resolver inventory, and compared against the application through the same resolver used to produce a `DeploymentPlan`.

A successful check reports selected provider bindings:

```text
Application: orders 1.0.0
Environment: local
Status: compatible
  capability bindings:
    presolve:kv/store -> memory-services (prv_...) @ 0.1.0
```

An incompatible application exits unsuccessfully and reports stable resolver diagnostic codes such as:

```text
PS2001  missing required capability
PS2002  insufficient memory
PS2003  insufficient CPU
PS2004  missing required capability features
PS2005  unsupported workload execution class
```

`presolve check` does not materialize providers or execute workloads. It answers one question:

> Can this application be resolved in this environment?

Environment selection is currently discovery-based. Explicit environment selection can be added later without changing this default.

## Local provider realization

Presolve now has a node-side local realization profile for resolved plans.

The resolver still selects symbolic provider identities from the Environment Specification:

```text
Application Contract
        +
Environment Specification
        ↓
DeploymentPlan
        ↓
provider id + capability binding
```

For local development, Presolve can then realize the selected baseline bindings with built-in process-local implementations:

```text
presolve:kv/store       → in-memory key/value provider
presolve:objects/store  → in-memory object provider
```

This does **not** change `presolve.env.toml` and does not infer an implementation from a provider's name. The realization is an execution-time local policy applied only after resolution.

The current in-memory object provider implements baseline object-storage semantics only. If a resolved application negotiates optional features such as `range-read`, `conditional-write`, or `user-metadata`, local realization fails rather than silently weakening the capability contract.

Custom capabilities and unsupported built-in versions likewise remain resolvable at the semantic layer when an Environment Specification supplies them, but the built-in local profile cannot materialize them until a matching local implementation exists.

`presolve check` remains a semantic compatibility check. The next local workflow step, `presolve dev`, will use this realization path to assemble and execute the resolved application.


## Run an application locally

For the currently implemented single-workload Rust authoring path, local development is now one command:

```bash
presolve dev
```

Presolve:

1. discovers the nearest `presolve.toml`
2. discovers the nearest `presolve.env.toml`
3. resolves application requirements against the environment
4. stops with the same `PS2xxx` compatibility problems as `presolve check` if resolution fails
5. builds a colocated Rust `cdylib` for `wasm32-wasip2`
6. realizes the selected baseline providers with the local provider profile
7. assembles the runtime
8. invokes the component's `run(string) -> string` export

Pass input to the current application entry point with:

```bash
presolve dev --input "Austin"
```

The automatic Rust build expects `Cargo.toml` beside `presolve.toml` and exactly one `cdylib` target. Cargo metadata is used to discover the target directory and library target rather than assuming a hard-coded package name.

An already-built WebAssembly Component can bypass the Rust build convention:

```bash
presolve dev --component ./path/to/application.wasm --input "Austin"
```

Relative component paths are interpreted from the directory where `presolve dev` is invoked.

The explicit component path is the language-neutral escape hatch for the current development runner. Rust build discovery is a convenience for today's implemented authoring path, not part of the Application Contract and not a permanent requirement of Presolve applications.

The initial `presolve dev` runner supports zero or one declared workload. A contract with no explicit workload uses the existing implicit `main` workload. Multi-workload local execution requires workload build/artifact orchestration and is intentionally deferred rather than guessed.

The local providers are process-local. Their state exists for the lifetime of the `presolve dev` process and is not persisted to `presolve.env.toml`.
