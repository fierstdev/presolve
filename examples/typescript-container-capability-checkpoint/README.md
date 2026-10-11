# TypeScript container capability checkpoint

This is the Chunk 17.3 proof workload.

The application code intentionally uses only the TypeScript authoring layer:

```ts
import { kv } from "@presolve/sdk";
import { installContainerBridge } from "@presolve/sdk/container";

installContainerBridge();
await kv.set(
  "presolve.chunk16.checkpoint",
  "container-capability-ok",
);
```

The main `@presolve/sdk` call remains provider- and transport-neutral. The container-specific bootstrap is isolated to `@presolve/sdk/container`.

The resulting OCI image is fed into the same host-side Presolve checkpoint used by Chunk 16. The host verifies that the value written by TypeScript arrived in the exact KV provider selected by the `DeploymentPlan`.
