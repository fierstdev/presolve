# TypeScript Component checkpoint

This example proves the automated Presolve TypeScript-to-WebAssembly Component build path.

Application source remains transport-neutral:

```ts
import { kv } from "@presolve/sdk";
```

The source does not import Jco bindings, WIT interfaces, provider identities, or environment configuration.

From this directory:

```bash
presolve build
```

Presolve detects `package.json` plus `src/main.ts`, type-checks the application, generates build-only Component/WIT bootstrap material under `.presolve/`, invokes the pinned Jco/ComponentizeJS toolchain, and packages the resulting Component into:

```text
dist/typescript-component-checkpoint.presolved
```

`presolve.toml` remains the only Application Contract.
