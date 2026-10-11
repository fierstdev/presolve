# Chunk 17 checkpoint: TypeScript authoring

Chunk 17 adds a TypeScript-facing authoring layer without changing Presolve's language-neutral application, environment, resolution, release, or provider models.

## Canonical boundary

The Application Contract remains:

```text
presolve.toml
```

TypeScript does not introduce a second application declaration.

Application code imports semantic capability APIs:

```ts
import { kv, objects } from "@presolve/sdk";

await kv.set("session", "abc");
await objects.put("assets/logo.bin", bytes);
```

Those calls do not name a provider, deployment environment, registry, transport, or execution substrate.

## Proven architecture

Chunk 17 establishes:

```text
TypeScript application code
        ↓
@presolve/sdk
        ↓
typed KV/object semantics
        ↓
CapabilityTransport
       ╱ ╲
      ╱   ╲
     ▼     ▼
container  Component
 adapter    adapter
     │        │
HTTP/JSON   WIT imports
     ╲        ╱
      ╲      ╱
       ▼    ▼
 existing Presolve capability/provider model
```

### Container projection

`@presolve/sdk/container` projects semantic calls onto the authenticated `presolve-http-json/0.1` bridge.

This path has a real end-to-end OCI proof. A compiled TypeScript workload runs inside Docker, calls `kv.set(...)`, and the host checkpoint observes the write in the exact KV provider selected by `DeploymentPlan`.

### Component projection

`@presolve/sdk/component` projects the same semantic calls onto normalized imports matching Presolve's existing WIT.

KV maps directly to `get`, `set`, and `delete`.

Object storage maps the public `Uint8Array` API to the Component handle protocol:

```text
get → read-session → read* → close-read
put → write* → finish-put
                  ↘ abort-put on failure
```

The adapter is covered by deterministic tests, including chunked reads/writes and failure cleanup.

## Deliberate non-claims

Chunk 17 does **not** claim that Presolve already has an automated TypeScript-to-WebAssembly-Component build pipeline.

A future build workflow still needs to choose or integrate a compatible JavaScript/TypeScript Component toolchain and generate the thin binding/bootstrap layer that satisfies `ComponentCapabilityImports`.

That work belongs to the build/package workflow, not to application semantics.

The TypeScript SDK is also still pre-stable. Its package API, capability bindings, and adapters may change incompatibly before a stable release.

## Reproducible checkpoint

The fast authoring checkpoint is:

```bash
pnpm checkpoint:ts
```

It verifies:

- SDK type safety
- SDK behavior tests
- TypeScript OCI example type safety
- container and Component package exports
- capability-interface identities
- that application-facing `kv`/`objects` modules do not contain execution/provider discovery details

The real OCI execution proof remains:

```bash
pnpm smoke:ts-oci
```

That test requires Docker and exercises the full container path.
