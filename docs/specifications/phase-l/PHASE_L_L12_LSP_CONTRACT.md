# L12-D -- LSP adapter contract

**Status:** Authoritative contract. No LSP server, transport, editor extension,
source discovery, compiler invocation, or update implementation is authorized
by this document alone.

## Authority and lifecycle

The future adapter is a stateless translation layer over one caller-supplied
`@presolve/language-service` query. It owns LSP JSON-RPC framing and client
capability negotiation only. It retains no product bytes, document text, URI
mapping, workspace state, cache, request task, or compiler session. The client
continues to own URI-to-`SourceUnitId` and UTF-16-to-UTF-8 offset translation;
the adapter may not infer either from a path or document.

Each LSP request carries caller-provided canonical query-product bytes plus the
already translated source-unit ID/offset or opaque query semantic ID. The
adapter passes those unchanged to the language-service and maps its response
without reordering records/ranges or changing errors. A replacement product is
a new independent request; L12-D has no didOpen/didChange/didClose, watch,
filesystem, or incremental-update protocol.

## Supported protocol mapping

| LSP method | Language-service operation | Result |
| --- | --- | --- |
| `textDocument/definition` | `definition` with caller-selected opaque ID | one existing range record |
| `textDocument/references` | `references` with caller-selected opaque ID | existing references in product order |
| `textDocument/documentSymbol` | `documentSymbols` with caller source unit | flat existing records in product order |
| `textDocument/publishDiagnostics` projection | `diagnostics` with caller source unit | existing primary-range diagnostics in product order |
| position lookup extension | `position` with caller source unit and UTF-8 offset | all matching records in product order |

The adapter returns language-service `error` results as LSP error responses
whose code is the exact stable language-service code. There is no retry,
fallback parse, alternate compiler, or path/source lookup. Since L12-C queries
are synchronous and stateless, cancellation before dispatch is caller-owned;
after dispatch the adapter returns the completed response and has no partial or
cancelled result.

## Unsupported methods and capability negotiation

The adapter advertises only definition, references, document symbols, and
diagnostics/position projection. Hover, rename, completion, signature help,
semantic tokens, source mapping, edits, code actions, workspace symbols,
document synchronization, and every unknown method return the stable
`unsupported` language-service result without dispatching any compiler work.
Client capabilities do not widen this list.

## Required implementation evidence

The adapter slice must add canonical request/response fixtures for each mapping,
every language-service error, every unsupported method, capability negotiation,
and cancellation-before-dispatch. It must prove byte/order/range preservation,
absence of product decoding/source/URI/path handling, no document lifecycle or
retained state, and no LSP/VSCode dependency cycle. L12-E remains unstarted
until those fixtures and the adapter verifier are complete.
