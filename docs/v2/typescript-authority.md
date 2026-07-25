# TypeScript semantic-authority boundary

Schema version: `1`<br>
Primary TypeScript version: `7.0.2`

`@presolve/typescript-authority` is the sole integration point for concrete
TypeScript semantic APIs. Compiler domain modules consume its serializable
products; they must not import TypeScript directly or recreate general-purpose
type checking, alias resolution, or module resolution.

## Request and response

`analyzeTypeScriptProject` opens one configured TypeScript project and returns
the following schema-v1 products:

- project configuration and sorted root files;
- native config, program, syntactic, bind, and semantic diagnostics, each with
  its original TypeScript diagnostic code and source span;
- resolved symbols, including an alias target only when TypeScript classifies
  the symbol as an alias;
- type and contextual-type displays derived by TypeScript;
- resolved call signatures with parameter and return types;
- TypeScript assignability results for requested source/target positions; and
- module symbols obtained at import, export, or package-specifier positions.

The request carries a configuration file plus explicit file/UTF-16 positions.
Returned symbols and modules expose declaration paths, not TypeScript process
handles or compiler-internal IDs. This keeps the boundary deterministic and
serializable while retaining TypeScript as the authority.

## Ownership

The adapter may use TypeScript-native APIs. The parser remains responsible for
source-faithful syntax; later compiler normalization consumes syntax together
with this adapter's query products. Vite is not part of this boundary. A later
Vite integration must compare its resolution result with the authority product
for compiler-owned modules and diagnose divergence.

## Proof

The adapter test uses the versioned compatibility corpus to prove package
exports, aliases, generic types, contextual typing, call overload resolution,
assignability, and native diagnostic preservation. The package is included in
the workspace test graph, so its boundary test runs with ordinary package
verification.
