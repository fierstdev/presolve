# @presolve/sdk

`@presolve/sdk` is the TypeScript authoring layer for Presolve capability semantics.

The SDK does not define a second Application Contract. `presolve.toml` remains the canonical application-level declaration of workloads, capabilities, resources, topology, and network policy.

The SDK's job is narrower:

```text
TypeScript application code
        ↓
typed Presolve capability APIs
        ↓
transport-neutral capability call
        ↓
execution-target adapter
```

Chunk 17.1 establishes only the transport-neutral kernel. KV/object clients and the initial container bridge adapter are added in 17.2.

Application code should import capability APIs from `@presolve/sdk`. Execution adapters use the low-level `@presolve/sdk/runtime` entry point to install a transport during workload bootstrap.

No Docker, HTTP, WIT, provider identity, registry location, or provider credential is part of the public application-facing model.
