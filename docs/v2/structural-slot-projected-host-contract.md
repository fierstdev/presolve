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
declared lexical content owner. Projected component invocations are published
as exact membership but remain activation-ineligible until a separate
State/Effect identity contract defines their clone semantics. A non-empty
`nested_invocations` projection therefore fails closed; no caller, callee,
Slot name, or DOM position is substituted at runtime.

The runtime may replace only compiler-declared structural occurrence and keyed
local-occurrence placeholders. It must validate the exact projection binding,
template parent relationship, renderer membership, and invocation markers
before mutating the host range. It may not flatten, clone, or move arbitrary
caller nodes.

## Transaction and lifecycle

Projection rendering is one phase of the enclosing structural materialization
transaction. A projection failure rolls back its staged targets, bindings,
events, enclosing structural occurrences, Effects, and DOM nodes with the same
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

The v20 gate is active for authority-proven V2 `slot()` fields. Compiler
semantic-span ownership plus exact emitted IDs partition default and named
Slot targets, text bindings, and events into the conditional or keyed fragment
that actually renders them. Ordinary cold boot excludes those dormant caller
records; structural materialization registers them transactionally, qualifies
keyed identities, retains them across reorder, and removes their subscriptions
on trim or branch cleanup.

The decorator-free browser fixture proves conditional reveal/hide, keyed
materialization/reorder/trim, shared caller State/action/binding/event
behavior, callee structural lifecycle cleanup, and fail-closed rejection for
fabricated binding, ownership, and renderer-marker evidence. Projected
component invocation cloning and Slot capture/resume remain fail-closed
follow-on contracts.
