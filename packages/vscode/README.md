# Presolve for Visual Studio Code

Presolve uses the workspace TypeScript project for TypeScript and TSX syntax
checking. Install this extension and open a project created with
`pnpm create presolve`; its generated `tsconfig.json` selects the supported
TypeScript/JSX settings and the public `presolve` package supplies authoring
types.

The extension does not suppress TypeScript diagnostics and does not implement
a second compiler. Compiler-derived navigation and diagnostics are introduced
only through versioned compiler language-service products.
