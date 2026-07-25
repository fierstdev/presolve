# Function-summary contract

Function summaries are immutable projections over `ControlFlowGraphV1` and an
explicit `FunctionCallFactsV1` envelope. Call facts are a future lowering
product; a function-summary consumer must never discover calls by scanning
source text, generated JavaScript, or Vite modules.

Schema version `1` records direct reads, writes, callees, and unknown-call
evidence for every CFG function. If call-fact coverage is `complete`, it also
computes transitive reads, writes, callees, and conservative unknown-call
propagation across local and cross-module functions. Recursive call groups
reach a deterministic fixed point.

If coverage is `unavailable`, all transitive fields are omitted. An omitted
field is not an empty set and cannot justify optimization, capture admission,
or purity. An explicit call fact without a callee marks the direct and every
calling summary as conservatively unknown. Invalid caller/callee identities are
rejected rather than silently detached.

The current source/IR lowering has not yet emitted call facts, so it must use
`unavailable`. A later lowering amendment must define the call IR form,
provenance, unknown-call diagnostic, and cross-module corpus before it can
claim complete coverage.
