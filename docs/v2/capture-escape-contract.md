# Capture and escape contract

`presolve_compiler::capture_escape` is schema v1 of the V2 capture/escape
analysis product. It records only explicit capture and escape evidence supplied
by a future lowering that owns closure and suspension semantics. It does not
scan source text, infer captures from identifier spelling, or reinterpret the
existing generated snapshot transport in `resume_capture`.

## Inputs and validation

`CaptureEscapeFactsV1` carries a closed set of facts, each naming one canonical
CFG function, one exact CFG-visible access, a capture or escape kind, and source
provenance. The builder rejects an unknown function or an access not present in
that function's CFG. This keeps facts tied to compiler identities rather than a
second analysis authority.

The facts have `complete` or `unavailable` coverage. `complete` means the
owning lowering accounted for every capture and escape in the selected scope;
it is not inferred from an empty fact list.

## Resume admission

The product lists deterministic captures and escapes for every CFG function and
assigns one admission result. `admissible` requires all of the following:

- explicit capture/escape facts are complete;
- CFG capture/escape, async-suspension, unknown-call, and resource-cancellation
  coverage is available;
- function-call coverage is complete; and
- the transitive summary has no unknown call.

Any unavailable prerequisite produces `rejected_unavailable_coverage`; a known
unknown call produces `rejected_unknown_call`. A rejection is a proof boundary,
not a claim that no capture exists.

Current canonical IR marks the required async, unknown-call, capture/escape,
and cancellation dimensions unavailable, so its functions are correctly
rejected for resume admission until an owning lowering provides those facts.
The existing `resume_capture` runtime transport remains unchanged; this product
is analysis evidence only and does not serialize or restore application values.
