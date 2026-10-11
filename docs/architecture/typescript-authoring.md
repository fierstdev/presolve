# TypeScript authoring

Presolve's TypeScript authoring layer is a language-facing projection of the existing application and capability model. It is not a new application model.

## Source of truth

The canonical application declaration remains:

```text
presolve.toml
```

TypeScript configuration does not replace or duplicate the Application Contract.

This keeps application identity and environment resolution language-neutral.

## SDK boundary

TypeScript application code will use `@presolve/sdk`:

```text
TypeScript source
        ↓
@presolve/sdk
        ↓
typed capability client
        ↓
CapabilityTransport
        ↓
execution-target adapter
```

The SDK kernel defines `CapabilityCall` and `CapabilityTransport`.

A capability call contains only:

```text
interface
operation
input
```

It does not contain:

```text
provider identity
provider credentials
Docker configuration
HTTP endpoints
registry references
WIT implementation details
environment selection
```

Those remain outside application authoring.

## Runtime installation

The low-level `@presolve/sdk/runtime` entry point allows an execution adapter to install one `CapabilityTransport`.

Application code normally does not install transports itself.

This supports multiple future projections without changing capability-facing application code:

```text
@presolve/sdk
      │
      ├── container bridge adapter
      ├── Component/WIT adapter
      └── future compatible transports
```

Chunk 17.1 establishes this boundary. It deliberately does not yet expose KV/object convenience APIs or bind the SDK to the existing HTTP/JSON bridge.

## Package tooling

The repository's existing pnpm workspace hosts TypeScript packages and examples alongside the Cargo workspace.

The Rust workspace remains authoritative for Rust crates and execution internals; pnpm exists only for language-facing packages and examples.

## Typed baseline capabilities

Chunk 17.2 adds functional TypeScript APIs for Presolve's baseline KV and object-storage capabilities:

```ts
import { kv, objects } from "@presolve/sdk";

await kv.set("session", "abc");
const session = await kv.get("session");

await objects.put("images/logo.bin", bytes);
const logo = await objects.get("images/logo.bin");
```

The public object API uses `Uint8Array`. The HTTP/JSON bridge's byte-array encoding remains an adapter detail.

Typed clients validate transport output before returning it to application code. Malformed output raises `CapabilityProtocolError`; provider/bridge error envelopes raise `CapabilityInvocationError`.

## Container adapter

The initial execution-target adapter lives at `@presolve/sdk/container`.

It implements `CapabilityTransport` using the existing `presolve-http-json/0.1` bridge and injected Presolve discovery variables.

```text
@presolve/sdk             application-facing semantics
@presolve/sdk/runtime     low-level transport installation
@presolve/sdk/container   OCI/container HTTP/JSON projection
```

The adapter is separate from the main package surface, so normal TypeScript application code remains transport-neutral.

Chunk 17.3 uses this adapter from a real TypeScript OCI workload.

## Real TypeScript OCI workload proof

Chunk 17.3 validates the TypeScript authoring path against the real OCI/container execution architecture proven in Chunk 16.

The proof workload is ordinary TypeScript:

```ts
import { kv } from "@presolve/sdk";
import { installContainerBridge } from "@presolve/sdk/container";

installContainerBridge();
await kv.set(
  "presolve.chunk16.checkpoint",
  "container-capability-ok",
);
```

The image is built from the pnpm workspace, exported with OCI media types, and identified by its immutable OCI descriptor. That descriptor is then passed through the existing Presolve container checkpoint.

The host-side checkpoint:

1. resolves the container application and KV requirement,
2. constructs `.presolved` identity from the OCI descriptor,
3. realizes the exact selected KV provider,
4. starts the authenticated capability gateway,
5. materializes and launches the image through `ContainerExecutor`,
6. observes the value written by the TypeScript workload in the selected provider.

This proves the path:

```text
TypeScript source
    ↓
@presolve/sdk
    ↓
@presolve/sdk/container
    ↓
presolve-http-json/0.1
    ↓
exact DeploymentPlan ProviderId
    ↓
KeyValueProvider
```

The proof does not introduce a TypeScript-specific deployment model. `presolve.toml`, Environment resolution, `.presolved`, and `DeploymentPlan` remain unchanged.
