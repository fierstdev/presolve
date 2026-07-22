# Presolve reproducibility lanes

**Status:** L15-C lane manifest. Each lane delegates to existing tests and
records no host-performance result as correctness evidence.

| Lane | Pinned inputs | Local command | Artifact/evidence | Gate |
| --- | --- | --- | --- | --- |
| deterministic-contracts | committed Rust/CLI/tooling fixtures | `just check` | canonical products, schema, CLI, lifecycle checks | required |
| browser-runtime | committed runtime fixture plus configured Chrome | `pnpm test:e2e` | browser runtime result | required when browser fixture applies |
| package-smoke | committed package smoke inputs and local WASM build | `pnpm -r check` | package dependency/smoke boundary | required |
| documented-examples | none before L14; then the L14 corpus | L14 verifier | example command outputs | deferred until L14 |
| observation | declared corpus and host manifest | future report command | noncanonical observation report | never a gate |

The deterministic lane is the complete local reproduction baseline. Browser
evidence is separate because its Chrome binary is environment-owned. Package
smoke runs may create only ignored/generated local build output. The observation
lane cannot compare elapsed time, CPU, memory, machine identity, or benchmark
values as a correctness condition.

L14-A is governed by the [alpha example contract](examples-contract.md). The
example lane stays deferred until each contracted example is proven.
