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
  the symbol as an alias, and a project-relative declaration-module identity;
- type and contextual-type displays derived by TypeScript;
- resolved call signatures with parameter and return types;
- TypeScript assignability results for requested source/target positions; and
- module symbols obtained at import, export, or package-specifier positions,
  including their resolved declaration modules; and
- ordered class base-symbol chains for explicit component-heritage queries.

The request carries a configuration file plus explicit file/UTF-16 positions.
Returned symbols and modules expose declaration paths, not TypeScript process
handles or compiler-internal IDs. This keeps the boundary deterministic and
serializable while retaining TypeScript as the authority.

The identity is a normalized declaration-module vector plus TypeScript symbol
name and flags. It deliberately is not a durable Presolve semantic ID; stable
cross-edit IDs are a later incremental-compilation product. Consumers that need
the canonical target of an import must use `aliasTarget.identity`, not the
author's local spelling.

Component-heritage queries serialize each direct and indirect base symbol after
TypeScript alias resolution. They do not classify framework meaning. The
canonical intrinsic registry compares the serialized identities with the
resolved `Component` export; this preserves the beta rule that aliases and
indirect subclasses are components without spelling-based recognition.

`analyzeV2Authoring` is the schema-v2 bridge for the implemented source forms.
Its caller supplies parser-selected positions for the canonical framework
exports and candidate component heritage, State, Action, and Effect sites. The bridge
returns only registry-classified resolved evidence plus native diagnostics; it
does not parse source, create compiler identities, or lower runtime behavior.
The installed `presolve-typescript-authority` executable exposes that same
schema as one JSON request on stdin and one JSON response on stdout.

## Ownership

The adapter may use TypeScript-native APIs. The parser remains responsible for
source-faithful syntax; later compiler normalization consumes syntax together
with this adapter's query products. Vite is not part of this boundary. A later
Vite integration must compare its resolution result with the authority product
for compiler-owned modules and diagnose divergence.

## Proof

The adapter test uses the versioned compatibility corpus to prove package
exports, aliases, generic types, contextual typing, call overload resolution,
assignability, native diagnostic preservation, and indirect component
inheritance. The package is included in
the workspace test graph, so its boundary test runs with ordinary package
verification.
