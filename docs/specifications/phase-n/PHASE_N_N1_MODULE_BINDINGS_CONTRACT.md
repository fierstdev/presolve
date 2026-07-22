# Phase N N1 module bindings contract

**Status:** N1 implementation authority.

## Admitted compiler semantics

N1 admits the compiler's existing explicit module binding model as a bounded
semantic capability:

- local and relative named, default, and namespace imports;
- local named/default exports and deterministic relative re-exports;
- local type aliases and type-namespace declarations retained by the parser;
- canonical `CompilationUnit`, `ModuleGraph`, `SymbolTable`, and `BindingTable`
  ownership for source paths, module edges, names, and binding diagnostics.

The compiler resolves only caller-supplied source units and relative paths. It
does not discover files, install packages, infer package exports, or read a
package's implementation. A bare package import is not compiler-native until
N1-A resolves it through an explicit semantic package contract.

## Type boundary

N1 does not claim general TypeScript checker compatibility. It retains local
type declaration names and source-faithful aliases used by existing compiler
type rules; generic utilities, structural imported types, conditional/mapped
types, declaration merging, ambient globals, and checker-dependent inference
remain deferred as `advanced_types`.

## Inspection and diagnostics

The N0 capability registry changes `module_bindings` to admitted and retains
explicit deferred records for `advanced_types` and executable
`semantic_package_exports`. N1-A subsequently admits only the separately
contracted import-binding identity; it does not make a package call executable.
Existing `PSBIND1001` through `PSBIND1006` diagnostics remain the canonical
binding failures. This slice does not change ASM, artifact, runtime, or resume
schemas.

## Evidence

The N1 verifier exercises the existing relative named/default/namespace import
and re-export binding fixtures, asserts the capability registry classification,
and checks the canonical CLI projection. It adds no package resolver or source
fallback.
