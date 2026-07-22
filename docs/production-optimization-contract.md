# Production Optimization Contract

Phase K freezes Presolve production optimization policy v1 and production
runtime artifact v1. Production is a compiler-owned projection of the frozen
Phase A-J products; it does not reinterpret source, DOM, authored names, or
runtime registration order.

## Build mode and authority

`ezc_cli build <input> --out <directory> --production` emits the normal
development artifacts plus a production artifact and content-addressed modules
under `production/`. Development builds keep their existing HTML/runtime path.
Production HTML adds the exact packed artifact as `#presolve-production-runtime`
before `runtime.js`; browser validation completes before any authored action,
Form submission, Effect, or resumed activation can execute.

The single authorities are recorded in
[the K20 refinement audit](production-runtime-refinement-audit.md): policy,
fingerprinting, reachability, ordinal packing, module emission, validation,
cleanup, and scheduling each have one compiler implementation.

## Policy v1

The policy is not user configurable. Its fixed thresholds are:

- shared candidate minimum: 2 roots and 1 program;
- shared canonical program bytes: at least 192;
- net saved canonical bytes: at least 256;
- lazy dependency depth: at most 1;
- inline literal maximum: 24 UTF-8 bytes;
- constant-pool reuse minimum: 2 consumers.

Reachability begins from cold boot, capture/restore, activation roots, events,
and frozen artifact program references. Removal requires exact unreachable
proof and preserves validation, failure, codec, cleanup, and public artifact
records. Program aliases require equal canonical opcode bytes and cannot cross
instance ownership, scheduling, native-default, failure-identity, capability,
or protocol boundaries. Constant pooling excludes mutable, identity-observable,
snapshot, and public-artifact values.

Shared chunks are registration-only, depend only on the eager chunk, and are
listed in canonical order before a root activation. The same shared module may
serve independent roots without merging their activation identity or state. A
shared import failure fails only the dependent activation and is not retried,
matching the Phase J failure contract.

## Generated modules

The module layout has exactly one eager `boot.<hash>.js`, zero or more
`shared.<hash>.js`, and zero or more `root.<hash>.js` files. Names are hashes of
canonical final module bytes. Imports/exports are fixed, modules execute no
work on registration, and source contains no comments, `eval`, `Function`, or
dynamic `import()`. Generated module source is never embedded in ASM inspection.

## Production runtime artifact v1

`production.runtime.json` uses camel-case top-level fields and contains:

- `schemaVersion: 1`;
- exact `buildId` and `runtimeProtocolVersion: 1`;
- `optimizationPolicy: "optimization-policy:production-v1"`;
- canonical tables for programs, chunks, activation roots, activations,
  anchors, and events;
- chunk records and the eager/activation entry table;
- artifact and per-table SHA-256 integrity checksums.

Each table is schema v1 with a typed ID, kind, count, minimum legal ordinal
width (`u8`, `u16`, or `u32`), dense zero-based mappings, declared referenced
tables, and checksum. Canonical string IDs are validated at the trust boundary;
closed bootstrap/scheduler products use compiler-emitted ordinals afterward.
The packed artifact is derived output and does not replace any earlier public
artifact or snapshot schema.

Validation is ordered and fail-closed:

1. V0 parse shape and prototype-key rejection;
2. V1 schema version;
3. V2 build/runtime protocol;
4. V3 table metadata and checksums;
5. V4 identity/ordinal bijections;
6. V5 referenced endpoints;
7. V6 fingerprints and aliases;
8. V7 chunks, imports, and exports;
9. V8 eager bootstrap closure;
10. V9 resume/anchor/event agreement;
11. V10 lifecycle closure.

Only the first failed phase is exposed to production boot. Compact failure
records retain class, stable code, build ID, subject kind/value, trust status,
and phase. They contain no source path, snippet, or development provenance.

## Reports

`optimization-report.json` v1 records build/policy identity, exact optimization
counts, runtime table count, development and production bytes, retained
exclusions, and validation status. `runtime-cost-report.json` v1 records exact
module/artifact/table/record counts plus compiler-defined static operation
units. Neither report is executable authority. Neither contains timestamps,
durations, milliseconds, or host measurements.

## Budgets and maintenance

[`fixtures/phase-k-benchmarks/corpus.json`](../fixtures/phase-k-benchmarks/corpus.json)
defines the 16-case correctness corpus.
[`budgets.json`](../fixtures/phase-k-benchmarks/budgets.json) contains the exact
K15 ceilings for production JavaScript, eager bytes, artifact bytes, runtime
records, static operation units, and module count. The shared representative
also records the Phase J relative comparison; lifecycle stress is exactly 100
cycles.

A baseline can change only for an explained correctness change, with generated
outputs and review evidence updated in the same commit. It must never be raised
solely to make a regression pass. Wall-clock timing remains informational.

## Inspection and diagnostics

ASM inspection v12 projects policy, reachability, fingerprints/aliases,
constant-pool and shared-candidate facts, chunk topology, packed tables,
artifact identity, cleanup closure, V0-V10 phases, static costs, blocks, and
exclusions. Full and selected inspection share the same immutable projection;
invalid source shows blocks without fabricated production IDs.

`PSC1112` through `PSC1127` are the ordered public production diagnostics.
Projection deduplicates by code, exact identity, and primary span. Identity and
provenance appear only when established, and secondary evidence is sorted.
Check JSON remains v6 because its existing identity/provenance envelope is
sufficient. `PSASM1385` through `PSASM1512` remain reserved for internal Phase K
integrity failures.

## Frozen versions

- semantic graph v6;
- template manifest v4;
- component runtime artifact v3;
- Context runtime artifact v2;
- Forms and Effect runtime artifacts v1;
- resume manifest v6, snapshot v1, protocol/registry v1;
- production runtime artifact/table v1;
- optimization report and runtime cost report v1;
- production optimization policy v1;
- ASM inspection v12 and check JSON v6.

## Phase K freeze evidence

The normative K16 corpus identity is
`6a9362085b04c9c708729762f6ce0a93e99c79154611745abb590654b5bff7ad`;
its budget identity is
`9f67f17a8e3d481dec0c43537990b60328844cec0f106e550d004cd374b09553`.
The corrected K0 artifact baseline frozen by K21 is
`47c4bd1497e4883b47ba28842d84737952143c9eded6d6c8c20bd76de4d43fdb`.

Representative K16 ceilings are:

- static: 136 production/eager JavaScript bytes, 5,510 artifact bytes,
  11 runtime records, and 9 static operations;
- Context/Effect multi-module: 666 production JavaScript bytes, 136 eager
  bytes, 14,019 artifact bytes, 31 records, 26 operations, and 3 modules;
- lazy/shared-candidate: 136 production/eager JavaScript bytes, 23,309
  artifact bytes, 26 records, and 28 operations, versus 145,794 Phase J
  executable bytes and 389 Phase J eager bytes.

The positive canonical shared-candidate proof saves exactly 552 bytes. The
representative corpus build truthfully extracts zero shared chunks because its
frozen executable products do not satisfy the complete extraction proof. The
100-cycle lifecycle proof returns the instance-owned registry to 0 after every
cycle while the immutable global program cache remains at 2; the real-browser
stress proof also preserves every compiler-owned Map/Set count.

The K21 final gate passes 686 serialized workspace tests, including all 37
real-browser probes and 448 core tests, followed independently by `just check`.

## Deliberate exclusions

Phase K does not add cryptographic signing, wall-clock performance gates,
cross-build resume migration, visible/manual activation, arbitrary JavaScript
serialization, runtime semantic discovery, DOM/virtual-DOM diffing, source-map
or source-snippet production output, configurable optimization thresholds,
lazy-to-lazy dependencies, speculative retry, or generalized minification.
