# @presolve/sdk

`@presolve/sdk` is the TypeScript authoring layer for Presolve capability semantics.

The SDK does not define a second Application Contract. `presolve.toml` remains the canonical application-level declaration of workloads, capabilities, resources, topology, and network policy.

Application code uses the main package surface:

```ts
import { kv, objects } from "@presolve/sdk";

await kv.set("session", "abc");
const value = await kv.get("session");

await objects.put("images/logo.bin", Uint8Array.from([1, 2, 3]));
const object = await objects.get("images/logo.bin");
```

These APIs are transport-neutral. They invoke semantic capability operations rather than a concrete provider or runtime.

Execution adapters are separate. The current OCI/container projection is exposed through:

```ts
import { installContainerBridge } from "@presolve/sdk/container";

installContainerBridge();
```

That adapter discovers `PRESOLVE_CAPABILITY_ENDPOINT`, `PRESOLVE_CAPABILITY_TOKEN`, and `PRESOLVE_CAPABILITY_PROTOCOL`, then installs the authenticated `presolve-http-json/0.1` transport.

Ordinary application imports do not expose Docker, HTTP, WIT, provider identity, registry location, or provider credentials.

The low-level `@presolve/sdk/runtime` export remains available for execution adapters and deterministic tests.
