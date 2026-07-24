# VS Code and language tooling

Install **Presolve** from the VS Code Marketplace (`fierstdev.presolve-vscode`),
then open the project root created by `pnpm create presolve`.

The extension preserves normal TypeScript and TSX diagnostics from the
workspace TypeScript project. It does not suppress errors, parse generated
JavaScript, or implement a second component analyzer.

The extension's alpha responsibility is deliberately small: it confirms the
workspace configuration and leaves normal TypeScript and TSX checking to the
project-local TypeScript server. Versioned compiler query APIs are available
through the compiler WASM, language-service, and LSP packages for integrations
that have a compiler query product to supply. If a compiler-derived feature is
not available, use normal TypeScript navigation and `presolve check` rather
than expecting a source-analysis fallback.

## Troubleshooting

1. Open the repository folder, not a loose `.tsx` file.
2. Run `pnpm install` so VS Code can resolve `presolve` and TypeScript.
3. Verify that `tsconfig.json` includes `app/**/*.ts` and `app/**/*.tsx`.
4. Run `pnpm check` in the integrated terminal for compiler diagnostics.
5. Confirm the extension is enabled with **Presolve: Show Workspace Status**.
