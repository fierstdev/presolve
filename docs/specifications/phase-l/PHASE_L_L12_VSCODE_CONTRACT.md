# L12-E -- VSCode extension contract

**Status:** Authoritative boundary contract. No extension package, activation,
editor integration, source access, compiler path, or language analysis is
authorized by this document alone.

## Authority

`@presolve/vscode` depends exclusively on `@presolve/lsp`. It may initialize
the completed in-process LSP dispatcher and forward caller-supplied product
bytes and already translated source-unit/offset or opaque-ID request facts. It
may render returned records, diagnostics, errors, and stable unsupported
results, but it may not decode/produce a query product, parse/bind/analyze
source, discover a workspace, map a URI/path, retain document text, open a
compiler session, start a transport, cache/persist facts, or synthesize edits.

## Capabilities

The extension advertises only definition, references, flat document symbols,
diagnostics, and position projection when its caller supplies the exact L12
facts. Hover, rename, completion, signature help, semantic tokens, source
mapping, edits, code actions, workspace symbols, synchronization, and every
unknown command render/return the stable LSP unsupported result. Editor client
capabilities do not widen this list.

## Lifecycle and verification

Activation is explicit and receives one WASM module plus the LSP dependency;
the returned extension object retains neither product bytes nor a document
model. Each command call is independent and delegates directly to LSP. The
implementation must include a pinned editor-shaped fixture proving method,
range/order/error preservation and unsupported behavior, plus an audit proving
the package imports only `@presolve/lsp` and no compiler/language-service
internals, filesystem/network/watch/editor text APIs, cache, persistence, or
edit capability. The Phase L freeze may begin only after that proof and the
phase-wide release audit pass.
