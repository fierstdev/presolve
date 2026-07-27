# Tooling query contract

The compiler-produced `ToolingQuerySnapshotV1` remains the only query fact
source. Compiler WASM, `@presolve/language-service`, and `@presolve/lsp` pass
its versioned bytes through the same strict request/response protocol; they do
not parse application source or resolve semantic IDs independently.

The supported beta projections are position lookup, hover, definition,
references, document symbols, and diagnostics. Hover accepts a canonical
`querySemanticId` and returns the exact record already present in the snapshot.
It does not infer TypeScript display text, evaluate a value, or invent a source
range. Unknown semantic IDs fail with `unknown_query_semantic_id`; unsupported
LSP methods stay explicit unsupported capabilities.

`presolve explain` remains the native compiler inspection interface and the
source of detailed provenance, ownership, type, artifact, and diagnostic
explanations. Tooling query records deliberately provide stable opaque source
unit identities; a client combines them with compiler provenance only through
an explicit compiler product.
