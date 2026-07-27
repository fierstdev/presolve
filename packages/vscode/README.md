# Presolve for Visual Studio Code

Install this extension and open a project created with `pnpm create presolve`.
It confirms that VS Code opened a configured project folder and provides the
**Presolve: Show Workspace Status** command. The generated `tsconfig.json`
selects the supported TypeScript/JSX settings and the public `presolve` package
supplies authoring types.

Presolve preserves ordinary TypeScript and TSX diagnostics. It does not hide
errors or implement a second compiler. Compiler-product tooling APIs are
published separately for integrations that can supply a compiler query product.
