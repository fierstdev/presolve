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

### Component execution adapter

The Component projection is available separately:

```ts
import { installComponentBindings } from "@presolve/sdk/component";

installComponentBindings({
  kv: generatedKvImports,
  objects: generatedObjectImports,
});
```

The binding facade mirrors the existing Presolve WIT semantics. KV calls map directly; object bodies use the existing handle/read/write streaming model.

Application code still imports `kv` and `objects` from `@presolve/sdk`. Only execution bootstrap code imports `@presolve/sdk/component`.

The SDK deliberately does not choose a TypeScript→WebAssembly Component compiler. Later build tooling may use any compatible toolchain as long as its generated WIT bindings are normalized to the `ComponentCapabilityImports` facade.
