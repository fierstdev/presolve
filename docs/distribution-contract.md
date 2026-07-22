# Distribution contract

**Status:** L17-A inventory only. No package in this repository is publishable;
every package manifest sets `private: true`.

| Package | Version | Export | Direct workspace dependency | Provenance |
| --- | --- | --- | --- | --- |
| `@presolve/compiler-wasm` | `0.1.0-alpha` | `./dist/presolve_compiler_wasm.js` | none | compiler-owned WASM build verifier |
| `@presolve/language-service` | `0.1.0-alpha` | `./src/index.js` | compiler-wasm | L12-C package verifier |
| `@presolve/lsp` | `0.1.0-alpha` | `./src/index.js` | language-service | L12-D package verifier |
| `@presolve/vscode` | `0.1.0-alpha` | `./src/index.js` | lsp | L12-E package verifier |
| `@presolve/testing` | `0.1.0-alpha` | `./src/index.js` | none | L15-B package verifier |
| `@presolve/runtime` | `0.1.0-alpha` | `./src/index.ts` | none | package manifest only |

Dependency direction is compiler-wasm → language-service → lsp → vscode; no
other workspace dependency is declared. The committed manifests and pinned
WASM/package verifier hashes are the present provenance evidence. `pnpm install
--offline` followed by `pnpm -r check` is the existing local package-smoke
evidence. There is no package checksum manifest, publish, signing, upload, or
release artifact until L17-B supplies a separately verified dry run.
