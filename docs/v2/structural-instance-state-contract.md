# Structural instance state contract

Dynamic component materialization may not reuse a compiler structural-template
instance ID as a live state owner. This contract defines the compiler product
and runtime derivation required before State, computed caches, bindings, or
events may be activated for a structural occurrence.

## Compiler product

Component artifact schema v14 retains the complete compiler-issued
State and computed slot declarations for its target component: declaration
identity, storage identity, initial serialized value, semantic type,
serializability, computed cache declaration, computed dirty declaration, and
the existing ordinary binding/event membership. These are template records,
not live instances, and must be validated against the same component/template
occurrence record that issued the rendered HTML.

The artifact must not publish a synthetic static `component_instance_id` for a
structural template. A missing State/Computed record is invalid whenever the
target component declares that record; an extra or mismatched record is also
invalid.

Both the compiler artifact validator and the browser artifact reader reject an
empty or mismatched template State/Computed identity, duplicate template slot,
or duplicate `(template_instance, declaration/storage)` pair before any
materializer code can consume the record. This validation creates no live
occurrence, State, computed cache, binding, or event registration.

The cold runtime may retain an inactive, invocation-keyed preflight table that
pairs these validated template slots with their exact compiler artifact and
manifest target, binding, and event records. Its entries retain the compiler
template instance ID and are not a live state owner. A materializer must derive
a separate opaque occurrence identity before it can create any live records.

## Runtime derivation

After the materializer creates an opaque occurrence identity, it derives live
State and computed slot IDs by replacing only the structural-template instance
prefix in the compiler-issued template slot record with that opaque occurrence
identity. The declaration/storage suffix is preserved byte-for-byte. The
template instance itself is never inserted into live state maps.

The runtime must create every State slot before registering a binding or event
for the occurrence, initialize each from the compiler serialized initial value,
and create computed cache/dirty slots before the first evaluation. Duplicate
derived IDs, unknown declaration suffixes, non-serializable resume admission,
or incompatible types reject the whole materialization and roll back all
records.

## Lifetime

An occurrence owns all of its derived State/computed records. Removal disposes
them after child occurrences and registrations, before removing the DOM range.
Reinsertion creates new records even if a keyed local key recurs later.
Retention and keyed reordering preserve the exact same occurrence records.

Structural State/Computed records are cold-only until the
[structural occurrence resume contract](structural-resume-contract.md) admits
their occurrence-qualified snapshot product. That product serializes only
compiler-codec-authorized State values and recomputes computed caches; it does
not discover occurrences from DOM.

## Required proof

Before activation, focused compiler and browser fixtures must prove nested and
keyed components with independent State and computed values, retention across
keyed reorder, removal/reinsertion reset, duplicate-ID rejection, rollback,
and the absence of structural records from resume snapshots.
