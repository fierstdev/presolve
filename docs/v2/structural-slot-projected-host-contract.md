# Structural Slot-projected host contract

This amendment activates a structural host only when its caller-owned Slot
projection can be represented entirely from existing compiler products. It
extends, but does not reinterpret, the schema-v1 Slot projection graph and the
structural host renderer scope contract.

## Product

Component artifact schema v20 adds `slot_projection_programs` to every
eligible conditional or keyed host fragment. Each program is selected by one
canonical Slot-binding ID and includes the exact caller instance, lexical
content owner, and ordered caller-owned target/binding/event/nested-invocation
membership reachable only through that projection. The canonical Slot-binding
product remains the authority for the callee, outlet, content fragment, and
direct-child facts; the host fragment remains the authority for the
compiler-rendered HTML that contains the projection.

The program is present only for a canonical `Bound` Slot binding. `Empty` is
represented by an exact empty projection; blocked, invalid, missing-outlet,
or duplicate projections make the structural host ineligible. The renderer
does not read source or inspect caller DOM to determine content.

## Identity and ownership

At materialization, the callee host uses its already-authorized opaque
occurrence identity. Every projected member remains owned by the compiler
declared lexical content owner. A projected child that is itself structural
uses an occurrence identity derived from that owner scope and the exact
compiler template instance; no caller, callee, Slot name, or DOM position is
substituted at runtime.

The runtime may replace only compiler-declared structural occurrence and keyed
local-occurrence placeholders. It must validate the exact projection binding,
template parent relationship, renderer membership, and invocation markers
before mutating the host range. It may not flatten, clone, or move arbitrary
caller nodes.

## Transaction and lifecycle

Projection rendering is one phase of the enclosing structural materialization
transaction. A projection failure rolls back its staged component records,
bindings, events, Effects, nested occurrences, and DOM nodes with the same
child-before-parent order used by ordinary structural removal. A successful
keyed reorder retains the same projection transaction and all occurrence
identities. Resume remains fail-closed for slot-projected hosts until a
separate Slot capture contract supplies exact coverage.

## Proof

The decorator-free, TypeScript-authority browser fixture must cover a
conditional and a keyed structural host inside a component whose Slot content
is caller-owned. It proves projected State/action/binding/event behavior,
keyed retention/reorder, child-first cleanup, and rejection of a fabricated
binding, substituted projection marker, or caller/callee ownership mismatch.
No source translation, selector-based DOM discovery, virtual DOM, or parallel
Slot decoder is permitted.

## Current acceptance boundary

The v20 slice publishes and validates the compiler-selected membership and
registers it transactionally for eligible structural host replacement. The
required end-to-end browser fixture remains blocked on an authored V2,
decorator-free Slot declaration/authority contract: current canonical Slot
declarations use `@slot()`, so introducing a replacement spelling here would
invent source semantics. Resume remains fail-closed for all Slot-projected
structural hosts.
