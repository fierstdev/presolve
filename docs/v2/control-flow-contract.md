# Control-flow contract

This contract supplies the previously missing implementation detail for the
control-flow foundation. It is deliberately an IR projection, not a second
source frontend: `presolve_parser` owns syntax, TypeScript owns TypeScript
control flow, and `presolve_compiler` owns the normalized IR facts exposed here.

## Schema V1

`ControlFlowGraphV1` has schema version `1` and projects each canonical
`IrFunction` into deterministic function records containing:

- stable function and basic-block IDs, source provenance, entry block, and
  branch edges;
- natural-loop membership and exits already represented by canonical IR;
- per-block data reads and writes, including IR values, storages, Context slots,
  computed values, and Resources; and
- per-function coverage statuses.

The projection is sorted by source module path and stable IDs. It never derives
control flow by inspecting decorator names, generated JavaScript, or ESTree
JSON. A Vite package may consume a later publication projection, but it has no
authority over this model.

## Coverage and fail-closed boundary

V1 makes only facts already present in canonical IR available. Definite
read/write dataflow, ordinary branch topology, and natural loops are available.
Exception paths, async suspension, generic unknown calls, capture/escape facts,
and Resource cancellation are explicitly reported as unavailable because the
current IR does not encode them.

No downstream analysis may treat an unavailable status as an empty set or a
safe result. The later summaries, purity, capture, and Resource slices must add
the corresponding IR fact and move only that coverage status to available with
focused corpus evidence. This is the conservative representation required for
unknown calls and unsupported source semantics.

## Source-lowering rule

When general source lowering reaches a function construct it cannot represent
with the current IR, it must preserve the source diagnostic and leave the
affected coverage unavailable. It must not synthesize exception, suspension,
or unknown-call edges from textual heuristics. A future expansion must amend
this contract with the new IR instruction/edge form, its provenance, exact
data-flow effect, diagnostics, and a control-flow fixture.

## Current consumers

The V1 product is an immutable foundation for function summaries and later
analysis. It does not alter runtime behavior, publication artifacts, resume
artifacts, or current ASM lowering.
