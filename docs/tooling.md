# Editor and language tooling

Install `presolve-vscode` from the VS Code Marketplace. The extension integrates
with the workspace project and preserves normal TypeScript/TSX diagnostics. It
does not create a parallel source analysis pipeline.

Presolve's editor architecture is compiler-product based:

1. the compiler produces a versioned query snapshot;
2. `@presolve/compiler-wasm` answers it;
3. `@presolve/language-service` projects supported queries;
4. `@presolve/lsp` adapts those queries to LSP messages; and
5. the VS Code extension hosts the editor-facing integration.

This alpha exposes a deliberately narrow capability set. Unsupported editor
features must report as unsupported; they must not infer source semantics from
generated JavaScript or rebuild a component graph.
