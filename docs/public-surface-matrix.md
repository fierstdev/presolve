# Public surface matrix

This **Reference** is fixture-validated from CLI dispatch, the L10 registry, and package manifests. It does not add commands, schemas, or exports.

## CLI dispatch

The current help registry is `version`, `build`, `check`, `clean`, `cache`, `workspace`, `watch`, `dev`, `create`, `explain`, `inspect`, `graph`, `trace`, `profile`, `benchmark`, and `doctor`. `create`, `dev`, `benchmark`, and `doctor` are reserved and exit `6`; the remaining documented L9/L11 adapters are linked in the [CLI reference](cli-reference.md).

## Available L10 schemas

All current registry entries are available at v1: `presolve.workspace-configuration`, `presolve.workspace-snapshot`, `presolve.workspace-graph`, `presolve.compiler-service-protocol`, `presolve.persistent-artifact-cache`, `presolve.cache-inspection-report.v1`, `presolve.workspace-manifest`, `presolve.watch-session-configuration`, `presolve.watch-change-batch`, `presolve.watch-execution-plan`, `presolve.watch-event`, `presolve.watch-session-snapshot`, `presolve.watch-execution-report`, `presolve.build-trace`, `presolve.compile-cost-report`, `presolve.artifact-graph`, and `presolve.query-snapshot`.

## Package exports

| Package | Export |
| --- | --- |
| `@presolve/compiler-wasm` | `./dist/presolve_compiler_wasm.js` |
| `@presolve/language-service` | `./src/index.js` |
| `@presolve/lsp` | `./src/index.js` |
| `@presolve/vscode` | `./src/index.js` |
| `@presolve/testing` | `./src/index.js` |
| `@presolve/runtime` | `./src/index.ts` |
