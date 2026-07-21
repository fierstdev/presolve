# L12-C-1 -- Compiler-owned WASM language-service binding contract

**Status:** Authoritative L12-C host-binding contract. It authorizes only the
subsequent compiler-owned WASM ABI slice; it does not itself add a WASM crate,
JavaScript package, LSP adapter, VSCode extension, compiler invocation, or
editor update path.

## Selected delivery authority

The delivery authority is a compiler-owned WASM ABI, provisionally packaged as
`@presolve/compiler-wasm`. Its implementation is compiled from the Rust
`presolve-compiler` authority and is the only browser or JavaScript-facing
boundary permitted to decode a supplied `presolve.query-snapshot` v1 document.
The ABI must call Rust `decode_tooling_query_snapshot_v1` before interpreting
any request. It may then make read-only projections from the decoded value.
There is no JavaScript product decoder, alternate compiler invocation, native
addon, IPC transport, filesystem access, source discovery, semantic cache, or
persistence path.

`@presolve/language-service`, when introduced, is a thin package over this
compiler-owned ABI. It owns URI/path translation at its outer client boundary,
but passes only compiler-issued source-unit IDs, UTF-8 byte offsets, opaque
query semantic IDs, and caller-retained product bytes into WASM. It cannot
accept source text, paths, URIs, project configuration, edit deltas, generated
artifacts, a cache handle, or a compiler service handle. It must not decode,
copy into durable storage, or re-emit the query snapshot.

## ABI and byte ownership

The v1 artifact exports exactly one read-only operation:

```text
query_snapshot_v1(product_bytes: Uint8Array, request_bytes: Uint8Array) -> Uint8Array
```

Both inputs are caller-owned immutable byte sequences. WASM makes no claim to
them after the call returns; it retains no product, request, response, or query
state. The returned bytes are a newly allocated, caller-owned canonical UTF-8
JSON response. The host may discard them immediately. The ABI has no
constructor, mutable session, update, dispose, background task, callback,
network, clock, filesystem, or environment entry point.

The implementation first invokes the Rust strict decoder over `product_bytes`.
No request processing is permitted if that decoder rejects. Product schema
negotiation is therefore exactly the existing `presolve.query-snapshot` v1
schema/version validation; absent, reserved, future, or noncanonical product
bytes yield `invalid_product`. A future product version requires a distinct ABI
entry point and contract; v1 never guesses or downcasts a version.

`request_bytes` and every returned response use canonical JSON: UTF-8,
`serde_json` field order as documented below, no unknown fields, and one final
newline. The request/response envelopes are binding transport records, not a
second compiler product and carry no semantic identity, source text, path, URI,
or durable lifecycle state.

## Request and response records

Every request is a v1 envelope with exactly these common fields:

```json
{"schema":"presolve.language-service-wasm-request","version":1,"operation":"..."}
```

The schema/version fields are required. The operation is one of the following
records; field names are exact camel case and no additional fields are allowed.

| Operation | Additional fields | Deterministic projection |
| --- | --- | --- |
| `position` | `sourceUnitId`, `offset` | Every semantic record whose range contains `start <= offset < end`, in the product's existing canonical semantic-record order. A zero-length range never matches. The response does not choose a “best” record. |
| `definition` | `querySemanticId` | The one semantic record with that opaque ID. This returns an existing record; it does not resolve a name at a position. |
| `references` | `querySemanticId` | Every existing resolved reference whose target opaque ID equals the requested ID, in existing product reference order. |
| `documentSymbols` | `sourceUnitId` | Every semantic record whose range has that source-unit ID, in existing semantic-record order. The v1 result is intentionally flat: no ownership hierarchy is inferred. |
| `diagnostics` | `sourceUnitId` | Every existing diagnostic with a primary range in that source unit, in existing diagnostic order. Diagnostics with no primary range are not inventedly assigned to a document. |
| `hover`, `rename`, `completion`, `signatureHelp`, `semanticTokens`, `sourceMapping`, `edits`, `codeActions` | none | Stable `unsupported` response; none of these requests may cause parsing, binding, analysis, source access, or update synthesis. |

`position` first confirms that `sourceUnitId` is one of the strictly decoded
product source units and that `offset <= sourceLength`. `definition` and
`references` first confirm that the requested opaque ID is an existing semantic
record. `documentSymbols` and `diagnostics` confirm the source unit exists.
The ABI never accepts a raw compiler `SemanticId`, semantic kind as a selector,
range supplied by a caller, path, URI, line/column, or authored text.

A completed request returns exactly one canonical envelope:

```json
{"schema":"presolve.language-service-wasm-response","version":1,"operation":"...","status":"ok","result":...}
```

For `ok`, `result` is respectively `{"records":[...]}`, `{"record":...}`,
`{"references":[...]}`, `{"records":[...]}`, or `{"diagnostics":[...]}`.
The records, references, diagnostics, kinds, ranges, and opaque IDs are exact
subsets of the strict-decoded product records with their existing JSON shape and
ordering. No newly computed range, identity, diagnostic, source mapping, or
symbol hierarchy may appear.

An unsupported known operation returns:

```json
{"schema":"presolve.language-service-wasm-response","version":1,"operation":"hover","status":"unsupported","capability":"hover"}
```

`capability` equals the request operation. Unsupported behavior is a result,
not a fallback error, and never depends on product contents.

## Errors, cancellation, and lifetime

All expected failures return a canonical response with `status: "error"` and
an exact `code`; they do not throw, discover inputs, or mutate state:

| Code | Condition |
| --- | --- |
| `invalid_request` | malformed/noncanonical request, unknown field, absent field, unsupported envelope schema/version, or unknown operation |
| `invalid_product` | `decode_tooling_query_snapshot_v1` rejects the supplied product bytes |
| `unknown_source_unit` | a requested source unit is absent from the decoded product |
| `offset_out_of_range` | a position offset exceeds the decoded source-unit length |
| `unknown_query_semantic_id` | a requested opaque ID is absent from decoded semantic records |

The error envelope is
`{"schema":"presolve.language-service-wasm-response","version":1,"operation":"...","status":"error","code":"..."}`.
For an unparseable request, `operation` is the empty string. The ABI does not
surface decoder internals, source paths, source text, host errors, timing, or
stack traces.

Every v1 request is synchronous, bounded by its supplied bytes, and performs
no compiler work. Consequently there is no in-flight operation to cancel:
cancellation is caller-owned before the ABI call, and v1 has no cancellation
request, response, partial result, callback, or retained work. A host that
abandons a call receives no stateful cleanup obligation.

The product is valid only for its own bound workspace/snapshot/source-unit
revisions, as already enforced by the compiler product. The ABI cannot compare
those values to an external document and cannot reuse an old result after the
caller replaces bytes. A future producer-owned incremental product is required
before an update API can exist.

## Packaging and implementation boundary

The subsequent implementation may add only a compiler-owned Rust-to-WASM
adapter and the `@presolve/compiler-wasm` build/package boundary needed to ship
this one operation. `@presolve/language-service` may follow as an outer thin
wrapper after ABI fixtures prove it cannot bypass WASM decoding. Neither package
may export a query-snapshot decoder, producer, cache, compiler entry point, or
source/document API. The runtime package remains unrelated.

The ABI implementation must factor its query projection in Rust so native Rust
tests and the WASM adapter exercise the same strict-decode-first authority.
It may not fork product validation or semantic-selection logic into TypeScript
or generated glue. The only supported browser/JS consumption is this artifact;
Node filesystem, subprocess, HTTP, worker, and browser editor integration are
out of scope.

## Required implementation evidence

The WASM ABI slice must add frozen canonical request/response fixtures covering
each supported operation, empty result sets, each unsupported capability, and
each error code. It must prove that a noncanonical or structurally valid but
identity-invalid product is rejected before query projection; unknown source
units, out-of-range offsets, and unknown opaque IDs fail closed; and source
enumeration order yields byte-identical responses.

The verifier must inspect the binding/package surface for no JavaScript
query-snapshot decoder, no source/path/URI/text field, no filesystem/network/
clock API, no compiler invocation, no cache or persistence, and no LSP/VSCode
dependency. It must run the existing L12 product gate, focused Rust projection
tests, WASM adapter tests, package checks, and `git diff --check`. L12-D and
L12-E remain unstarted until a separately authored LSP contract and its stable
fixture suite exist.
