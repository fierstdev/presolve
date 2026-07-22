# L10 — Tooling Schemas and Platform APIs

Status: Authoritative implementation contract

Authority: authored under the Phase L constitution after L9. This contract
supersedes the heading-only L10 wording where it supplies implementation detail.

## Scope and non-goals

L10 freezes the transport-neutral tooling-schema boundary. It does not add a
compiler pass, language-service behavior, IDE protocol, network transport,
artifact generator, trace collector, cost collector, cache format, or durable
state migration. L3–L8 serializers, identities, diagnostics, artifacts, cache
entries, session files, and event journals remain byte-for-byte unchanged.

## Canonical schema registry v1

L10-A adds one immutable registry describing supported versions; registry data
is not a new compiler product and does not enter L4/L5/L6/L7/L8 state. Every
entry contains exactly a stable schema name, version 1, and a status of either
`available` or `reserved`.

Available entries are the existing canonical products: workspace configuration,
workspace snapshot, workspace graph, compiler-service protocol, persistent
artifact cache, cache inspection report, workspace manifest, and L8 watch
schemas. Their existing encoders/decoders remain the only content authority.

Reserved entries are `presolve.build-trace`, `presolve.compile-cost-report`,
and `presolve.artifact-graph`. They advertise no payload shape and must be
rejected as unavailable. L11 may make one available only by adding a canonical
producer, exact serializer, validation fixture, and compatibility proof.

## Negotiation v1

The request contains `schema` and `versions`, where versions is a non-empty,
unique descending list of positive integers. The response is either an accepted
version or a typed rejection. v1 accepts only version 1 for an available entry.
Unknown names, reserved names, empty/duplicate/zero versions, and no shared
version reject deterministically. Unknown fields are rejected. Negotiation
performs no filesystem, compiler, cache, workspace, or network operation.

Forward compatibility is explicit: a future version requires a new registry
entry/version and must never reinterpret v1 payload bytes. Readers must reject
unknown versions rather than guess or silently downgrade.

## Verification and completion

L10-A requires registry determinism, request validation, every rejection path,
and proof that negotiation imports no compiler service or durable/cache module.
L10-B documents the registry and adds it to `just check`. Its compatibility
corpus is `crates/presolve_compiler/fixtures/tooling-schema/` and its frozen midpoint
inventory is [`docs/tooling-capability-inventory.md`](../../tooling-capability-inventory.md).
L10 completes only after compatibility fixtures prove existing canonical
serializer bytes are unchanged, the L3–L8 audits pass, and the Phase L midpoint
gate is run. L11 is the first phase permitted to implement a reserved tooling
product.
