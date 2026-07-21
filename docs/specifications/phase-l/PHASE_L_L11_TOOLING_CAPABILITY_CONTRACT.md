# L11-A -- Tooling capability and reader contract

**Status:** Authoritative implementation contract

**Phase:** L11-A

**Prerequisites:** Phase K frozen; L3--L10 complete; the L10-B capability
inventory and compatibility corpus accepted.

**Next boundary:** L11-B read-only existing-product readers. No command may be
activated by this contract.

## 1. Scope

L11 turns existing immutable compiler-platform facts into future developer
tooling without creating another compiler. This L11-A slice is a capability
and input-boundary contract only. It adds no compiler pass, artifact graph,
trace collector, cost collector, cache format, durable state, service
transport, language-service behavior, command implementation, package, or
schema-registry availability change.

Phase A--K compiler semantics, generated artifacts, diagnostics, runtime
behavior, L3--L8 bytes, and L10 registry meanings remain frozen.

## 2. Non-negotiable reader boundary

L11-B readers may accept only a caller-named, explicit product file with:

```text
presolve <tool> --schema <registered-schema> --product <caller-named-file>
```

The command layer reads the named file once as opaque bytes. It first performs
L10 negotiation for the requested schema/version, then dispatches only to the
schema's established strict canonical reader. A reader must reject a schema
that is reserved, unknown, unavailable for public reading, malformed,
non-canonical, version-incompatible, identity-mismatched, or not provably
produced by its registered owner.

`--product` is a caller-authorized immutable product input, never a source
root, configuration file, workspace manifest discovery target, or directory to
scan. Readers do not parse, bind, analyze, lower, optimize, generate,
recompile, discover, glob, poll, resolve project membership, or mutate any
product. They may render a validated product to deterministic JSON, text, or
DOT only when the specific view contract authorizes it.

L11-B may add a strict reader only where an existing canonical producer's
format and identity validation are sufficient. It must not add a public L3
workspace-configuration decoder: L9's strict public configuration codec
remains distinct from the frozen internal L3 representation.

The shared tooling error namespace is reserved for later implementations:

| Code | Meaning |
| --- | --- |
| `L11T001` | schema negotiation rejected |
| `L11T002` | product reference is missing or structurally invalid |
| `L11T003` | product bytes fail strict canonical validation |
| `L11T004` | required producer provenance or identity cannot be established |
| `L11T005` | requested view is unsupported for the validated product |
| `L11T006` | product schema, version, identity, or requested view mismatch |

These are tooling errors and therefore use the permanently reserved CLI exit
code 6. No compiler diagnostic is created or translated.

## 3. Capability matrix

“Registry available” only means L10 can negotiate a schema name/version. It
does not assert that a public canonical reader, product-file contract, or tool
view exists. The matrix below is the complete L11 planning authority.

| Command or surface | Existing immutable fact | L11-A classification | Earliest permitted follow-up |
| --- | --- | --- | --- |
| `inspect workspace-snapshot` | L3 `WorkspaceSnapshot` with strict canonical decoder and `snapshot_id` | reader-ready | L11-B reader, then L11-C projection |
| `inspect workspace-graph` | L3 `WorkspaceGraph` with strict canonical decoder and matching `snapshot_id` | reader-ready | L11-B reader, then L11-C projection |
| `graph workspace` | L3 workspace graph's canonical compile-dependency edges | reader-ready only for the exact existing graph | L11-B reader, then L11-C JSON/text/DOT projection |
| `inspect cache` | L6 canonical inspection report, already projected unchanged by `presolve cache inspect` | command-specific projection already exists; standalone product reader not yet defined | preserve L9; add only if L11-B authors strict reader evidence |
| `inspect service`, `inspect workspace-plan`, `inspect watch-*` | L4/L7/L8 in-process products and encoders | registry available but no accepted public canonical decoder/provenance-file contract | L11-B must author exact strict reader proof or leave unavailable |
| `doctor` | L9 strict configuration validation; L6 cache inspection; L7 workspace validation | facts exist, but no one canonical aggregate health product | L11-G after a deterministic input/report contract; no project discovery |
| `explain` | legacy source-oriented compiler command and frozen Phase A--K inspection output | not a platform-product projector | keep legacy behavior isolated; a public projector requires a separate contract and captured product provenance |
| `graph semantic`, `graph artifact` | Phase A--K semantic/production facts; no L10 artifact-graph product | semantic graph is not yet a captured public tooling product; artifact graph is reserved | semantic path requires a reader/product contract; artifact path begins no earlier than L11-E/F |
| `trace` | L8 events/report are scheduling facts, not `presolve.build-trace` | unavailable; build trace is reserved | L11-D then L11-F |
| `profile` | Phase K structural reports and L5/L6/L8 facts | no canonical cross-product compile-cost report | L11-D then L11-F |
| `benchmark` | Phase K corpus/budgets; host-dependent observations | unavailable as a deterministic platform command | L11-G only after declared corpus/repetition/environment contract; timings are non-canonical |
| `create`, `dev` | no scaffold contract; no public server/HMR/transport product | alpha-excluded at this boundary | later dedicated contract; neither may be inferred from L8 |
| Language service / editor | no accepted immutable query product for the required semantic/range facts | blocked | L12-A capability audit, then amendment if required |

## 4. Product provenance rules

For the L11-C reader-ready views, the validated document itself is the source
of identity: a workspace graph must validate canonically and bind its
`snapshot_id`; a workspace snapshot must validate canonically and bind its
`snapshot_id`. A tool may report that identity but never replace it with a
path-derived, timestamp-derived, counter-derived, or content-reconstructed
identity.

Every future reader contract must state all of the following before code:

1. registered schema and exact negotiated version;
2. existing canonical producer and canonical encoder;
3. strict decoder/validator and identity or provenance field;
4. whether the product is transient, process-local, durable source-free, or
   caller-owned;
5. allowed views and deterministic rendering order; and
6. malformed, unavailable, unknown-version, and identity-mismatch behavior.

If any item is absent, the view remains unavailable with `L11T005`; a reader
or command must not manufacture a fallback.

## 5. L11 sequence and completion gates

L11-B implements no command. It introduces only the reader boundary for the
two reader-ready L3 products, with L10 negotiation, strict decoding, source-
free explicit-product fixtures, reverse-input determinism, and L3--L10 byte
preservation. It stops before L11-C.

L11-C may activate exactly `inspect workspace-snapshot`, `inspect
workspace-graph`, and the corresponding `graph workspace` views after their
human/JSON/DOT, help, exit-code, malformed-input, and provenance fixtures
pass. It must preserve L9's exit-6 behavior for every other requested view.

L11-D through L11-G remain governed by the revised Phase L roadmap. Reserved
schema availability changes only alongside an accepted producer, strict
reader, exact fixtures, and compatibility proof.

L11-A completes when this contract, the L10 midpoint inventory, and the Phase
L index are present; its verifier is included in `just check`; no runtime or
compiler source implementation changes; and the worktree is clean after the
atomic commit.
