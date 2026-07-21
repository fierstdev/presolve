# L11-D -- Build trace and structural compile-cost contract

**Status:** Authoritative implementation contract

**Prerequisites:** L10 registry; L11-A capability boundary; L11-B readers; L11-C workspace-product projections; frozen Phase K reports.

**Next boundary:** L11-E artifact-graph contract. This document authorizes no producer, decoder, public command, registry availability change, or persistence change.

## Scope

L11-D defines two future immutable, source-free tooling products: `presolve.build-trace` v1 and `presolve.compile-cost-report` v1. They describe facts already established by a completed compilation path and Phase K's frozen structural reports. They do not alter compiler scheduling, cache authority, workspace publication, generated artifacts, diagnostics, or runtime behavior.

The products are absent and must remain `reserved` in the L10 registry until L11-F delivers their complete producer/reader/fixture/compatibility proof.

## Canonical build trace v1

A trace is a deterministic publication record, not a timer timeline. It contains exactly `schema`, `version`, `trace_id`, `workspace_id`, `compiler_contract`, nullable `snapshot_id`, `outcome`, and ordered unique `stages`.

Each stage has exactly `ordinal`, `kind`, `outcome`, and source-free stable identities required by that kind. The only v1 kinds are `l3_snapshot`, `l5_incremental_plan`, `l6_cache`, `l7_workspace`, `l8_watch`, and `l4_publication`. A producer emits a kind only when its established L3--L8 operation actually occurred; it must not synthesize skipped stages. Stage order is this fixed ordinal order, not observation or wall-clock order.

`l3_snapshot` may carry the validated snapshot identity. `l5_incremental_plan` may carry the existing plan fingerprint/mode/fallback codes. `l6_cache` may carry only existing cache outcome/reason/key identity. `l7_workspace` and `l8_watch` may carry their existing result identities. `l4_publication` may carry the established commit sequence and published snapshot identity. No stage contains source text, paths, filenames, snippets, parser products, timestamps, durations, process IDs, memory values, host values, or new diagnostics.

The trace is transient response/product data. It is not an L4 durable-session file, L5 baseline, L6 payload/key input, L7 manifest, L8 journal entry, or cache entry. A trace failure cannot change compilation outcome.

## Canonical compile-cost report v1

The cost report is a deterministic structural projection over frozen Phase K `OptimizationReportV1` and `RuntimeCostReportV1` facts. It contains exactly `schema`, `version`, `report_id`, `build_id`, `optimization_policy`, report identities, production/eager JavaScript bytes, production artifact bytes, module/table/record counts, static operation units, and canonical optimization counts from the existing report only.

The report is emitted only for an existing successful Phase K production report pair from the same `build_id`. `OptimizationReportV1` is the sole Phase K report in the pair that carries `optimization_policy`; `RuntimeCostReportV1` intentionally has no policy field. Therefore a producer must require the optimization report's frozen production policy and direct same-invocation pairing, but it must not fabricate a policy field or claim a field-to-field policy comparison against the runtime-cost report. A producer rejects mismatched build IDs, missing, malformed, noncanonical, or incompatible report inputs; it does not estimate or reconstruct them from generated files.

There is no canonical field for elapsed time, CPU, wall-clock, heap, RSS, throughput, machine identity, process identity, cache temperature, or benchmark score. Such observations may later be emitted only as a separate machine-labelled noncanonical telemetry attachment. They have no report ID, fixture byte equality, compatibility meaning, CI threshold, cache key, or release gate.

## Producer, reader, and error boundary

L11-F must add both products in one atomic slice. Each product requires a canonical encoder, strict decoder, identity recomputation, schema/version rejection, source-free fixtures, reverse-input determinism, L3--L8/Phase K byte-preservation evidence, and L10 registry transition from `reserved` to `available`. Readers use the L11 explicit product-file boundary and never run compilation to obtain a trace or cost report.

Failures are tooling failures. L11-F reserves `L11T007` for invalid trace provenance, `L11T008` for invalid compile-cost provenance, and `L11T009` for a noncanonical or incompatible source report. They use CLI exit code 6 and never become compiler diagnostics.

## Verification and stop rules

L11-D verifies this contract is indexed and that L10 still keeps both schema names reserved. It makes no Rust implementation changes. L11-E starts only after this contract is committed.

Stop rather than extend this contract if a trace stage would require source analysis, an unrecorded lifecycle fact, a timestamp/duration, or new durable state; or if a cost field cannot be obtained from the frozen Phase K report pair. Those require a separate constitutional/product amendment.
