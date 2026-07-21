# L12-B -- Compiler query-snapshot constitutional amendment

**Status:** Authoritative implementation contract; no implementation is authorized by this document alone.

## Amendment

L3--L8 remain source-free at their existing public boundaries. L12 adds one new,
transient, compiler-produced `presolve.query-snapshot` product only after a
successful explicit compiler invocation has already produced a validated L3
workspace snapshot. It is neither an L3 replacement nor a durable session,
cache payload/key input, watch journal, build artifact, or source store.

The product carries no authored source text, normalized path, filesystem path,
URI, snippet, generated code, host datum, timestamp, or edit. A client supplies
an exact compiler-issued `SourceUnitId` and UTF-8 byte offset; it owns document
URI/path translation outside the compiler. Unknown, stale, ambiguous, or
out-of-range source-unit/offset input is a deterministic query error.

## Snapshot v1

The v1 document contains exactly schema/version/snapshot identity, bound L3
workspace and snapshot identities, ordered source-unit revisions, ordered
semantic records, ordered resolved references, and ordered compiler diagnostics.
Every range is a half-open UTF-8 byte range paired with its exact
`SourceUnitId`. A semantic record contains only existing `SemanticId`, existing
kind, and compiler provenance range. A reference contains existing source and
target semantic IDs plus its compiler provenance range. A diagnostic contains
the existing compiler code/severity/message and canonical primary/secondary
ranges when those facts exist.

The producer may map a semantic provenance path to a source unit only through
the same validated L3 workspace snapshot's canonical membership. It fails
closed if that mapping is absent or non-unique. Records sort by source unit,
start, end, then canonical identity; identity is SHA-256 over the canonical
document with its identity field omitted. Strict decoding recomputes identity,
ordering, workspace/snapshot binding, reference targets, and ranges.

## Supported and unsupported queries

The future L12-C API may expose only position-to-existing-semantic-record,
definition, references, document symbols, and compiler diagnostics. Hover,
rename, completion, signature help, semantic tokens, source mapping, edits,
and code actions remain unsupported: current compiler products do not establish
their required semantic or edit facts. The API must return a stable unsupported
result rather than infer, parse, bind, or analyze source independently.

## Privacy, lifecycle, and invalidation

The snapshot is caller-retained transient data. It may not be persisted by L4,
L5, L6, L7, L8, the CLI, the future language service, or an editor extension.
It is invalid when its bound L3 workspace/snapshot identity or any source-unit
revision differs. Incremental update behavior must be supplied by a future
producer product; L12-C may not synthesize source changes or reuse facts across
unbound snapshots.

## Next boundary

L12-C may begin only after an implementation slice adds the canonical producer,
strict decoder, source-free fixtures, identity/provenance/reverse-order proof,
and L10 registry amendment for `presolve.query-snapshot`. No language-service,
LSP, or editor package begins before that product gate completes.
