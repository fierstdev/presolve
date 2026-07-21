# L12-C -- Language-service binding prerequisite audit

**Status:** Authoritative boundary audit. No language-service, adapter, package,
or transport implementation is authorized by this document.

## Result

L12-C cannot yet implement the roadmap's `@presolve/language-service` package.
The only canonical `presolve.query-snapshot` decoder is Rust
`decode_tooling_query_snapshot_v1`, and the only producer is the transient Rust
`CommittedCompilation.query_snapshot` result. The repository has no existing
compiler-owned WASM ABI, native addon, IPC protocol, or JavaScript binding that
can deliver or strictly decode that product. The existing `@presolve/runtime`
package is runtime-only and establishes none of those authorities.

Consequently, a JavaScript package cannot validate the caller-supplied product
without either duplicating the strict decoder, invoking/embedding an alternate
compiler, or inventing a transport/persistence path. Each is prohibited by the
L12-B amendment and Phase L invariant of one authoritative producer/decoder.

## Preserved product boundary

The L12-C package, once authorized, must accept only a compiler-validated,
caller-supplied query snapshot through a compiler-owned binding. It may project
only position lookup, definition, references, document symbols, and compiler
diagnostics. It must not load a path, URI, source document, project, cache,
watch session, or generated artifact; parse/bind/analyze source; synthesize an
update; persist bytes; or expose hover, rename, completion, signature help,
semantic tokens, source mapping, edits, or code actions.

## Required next contract

An explicit binding contract must select exactly one delivery authority before
L12-C implementation begins:

1. a compiler-owned WASM ABI that retains Rust decoding and read-only query
   projection;
2. a compiler-owned platform-native addon with the same Rust-owned decoder and
   query projection; or
3. a Rust-native language-service API, accompanied by an explicit amendment
   that changes the roadmap's `@presolve/language-service` package target.

The selected contract must define the package/crate owner, exact request and
response schema, product-byte ownership, error and cancellation categories,
unsupported response, version negotiation, process lifetime, packaging, and
fixtures. It must prove that the host cannot bypass
`decode_tooling_query_snapshot_v1` or introduce source discovery, an independent
decoder, a semantic cache, or persistence.

Until that contract is selected, L12 stops after the query-product gate. LSP
and VSCode remain unstarted.
