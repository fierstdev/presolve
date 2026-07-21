# L12-A -- Editor capability audit

**Status:** Authoritative audit and stop contract

**Prerequisites:** frozen Phase A--K products; L3--L8 platform products; L10 registry; L11 readers and product projections.

## Result

No current immutable public product is sufficient to implement hover, definition,
references, rename, completion, signature help, semantic tokens, or source
mapping. L3 snapshot/graph products intentionally exclude authored source,
position/range models, semantic query identities, and edit authority. Phase K
artifact, trace, cost, and artifact-graph products are source-free production
facts and cannot answer editor queries without fabricating semantics.

Existing ASM/explain behavior is a legacy source-oriented compiler compatibility
path, not an immutable query product. It must not be repurposed as an editor
backend, reparsed outside the compiler, or treated as a language-service cache.

## Capability matrix

| Capability | Required compiler-owned fact | Current product status | Result |
| --- | --- | --- | --- |
| Hover, definition, references | stable semantic identity plus source range and position model | absent from public immutable products | blocked |
| Rename | editable reference set, collision policy, and source edit authority | absent | blocked |
| Completion, signature help | position-sensitive scope/type/query facts | absent | blocked |
| Diagnostics, symbols, semantic tokens | canonical range-indexed semantic/diagnostic records | no captured query product | blocked |
| Source mapping | stable generated-to-source range/provenance model | absent | blocked |

L5 incremental reuse and L7 workspace ordering are not editor invalidation APIs.
They cannot be adapted into one without a separately approved product.

## Next boundary

L12-B must author the smallest immutable compiler-produced query snapshot and
constitutional amendment before any language-service, LSP, editor package, or
query implementation begins. The amendment must define source provenance and
privacy, position/range representation, query identities, schema/version,
canonical order, invalidation provenance, persistence limits, diagnostics, and
Phase K compatibility. If it is not accepted, L12 stops and editor features
remain unavailable for alpha.
