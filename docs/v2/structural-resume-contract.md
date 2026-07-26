# Structural occurrence resume contract

This amendment admits resumability for live structural component occurrences.
It replaces the former cold-only boundary only for compiler-issued conditional
and keyed occurrences whose complete topology, State values, DOM anchors, and
Effect ownership can be proven without selector or source recovery.

## Versioned products

Component artifact schema v18 (carried forward by schema v20) adds the compiler-issued resume codec to every
serializable occurrence State slot and the exact parent template-instance ID to
every structural occurrence template. Resume manifest v7 requires snapshot
schema v2. A schema-v2 snapshot retains the existing fixed boundary values and
adds ordered `structuralOccurrences` records. Each record contains only:

1. the opaque occurrence identity and its four codec inputs;
2. the compiler template instance and structural region;
3. exact State-slot values, keyed by their occurrence-qualified slot IDs; and
4. no DOM node, selector, authored key expression, source text, callback, or
   effect cleanup value.

The snapshot record order is parent-before-child and is part of its integrity
contract. Its State values use the component artifact's exact compiler codec;
an unserializable State slot makes the complete structural snapshot ineligible.

## Restore transaction

The resume runtime reconstructs live records from the snapshot and the
validated compiler artifact, never from application DOM shape. For every
record it must:

1. decode and re-encode the opaque identity, proving the four inputs agree;
2. join the template instance, region, target component, and parent template
   to exactly one compiler occurrence;
3. require an already-restored parent occurrence where the compiler parent is
   structural, or the exact planned static parent otherwise;
4. derive State/computed, target, binding, event, and Effect identities only
   by replacing the template prefix with the validated occurrence identity;
5. validate every exact compiler target/binding/event anchor already present
   in the DOM; and
6. restore State, recompute computed values, register bindings/events, and
   activate eligible V2 Effects parent-before-child.

No HTML is rendered, inserted, moved, or removed during resume. A retained
keyed occurrence is reattached to its compiler-issued keyed host record so a
later reconciliation retains the same occurrence and does not re-run its
Effects. Conditional and keyed host values must exactly account for the
snapshot's top-level occurrences; missing, extra, duplicate, substituted, or
out-of-order records cause one cold fallback.

## Teardown and Effects

Restored Effect ownership uses the same occurrence-qualified active-instance
registry as cold materialization. Cleanup is never snapshotted. A successful
resume installs the cleanup program for each eligible Effect exactly once;
subsequent conditional/keyed removal uses the shared child-before-parent
disposal path. A V2 Effect that is not eligible on resume stays inactive.

## Required proof

The acceptance fixture must be decorator-free and TypeScript-authority-backed.
It proves resume of conditional, keyed, and nested occurrences with changed
child State; no DOM reconstruction; parent-first resumed Effect activation;
child-first cleanup after a later removal; keyed retention through a later
reorder; and cold fallback for malformed identity, parent topology, State
codec, or exact DOM anchor evidence.

Slot-projected structural hosts remain excluded. This amendment does not
authorize DOM discovery, source translation, a parallel renderer, or a global
cleanup sweep.
