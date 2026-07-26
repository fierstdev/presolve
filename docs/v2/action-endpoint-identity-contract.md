# V2 action endpoint identity contract

The alpha runtime groups state writes and event bindings by a decorated
`ComponentMethod` identity. A V2 `action(handler)` field has no such method;
creating one would hide a source translation and make decorators an implicit
semantic dependency. This contract defines the narrow compatibility migration
for runtime action endpoints.

## Identity

Each canonical V2 Action field `Component.field` has one source endpoint with
the distinct semantic identity
`component/.../action-endpoint:field`. Its individual writes keep the existing
`component/.../action:field:index` identities. The endpoint, not a synthetic
method, owns those writes and is the identity recorded by the action-batch
plan.

Legacy decorated action methods retain their existing method identity as their
endpoint identity. This preserves alpha artifact and resume identities while
making downstream consumers operate on the common concept of an action
endpoint.

## Admission and lowering

The V2 graph may create an endpoint only after all of the following are true:

1. the field is a canonical V2 Action and belongs to a canonical Component;
2. `ParsedInitializerCall::inline_handler` exists with a block body and no
   unsupported statements;
3. every parsed update targets a canonical V2 State of that Component;
4. no update requires a handler parameter or another unsupported runtime
   operand; and
5. an `ActionAuthorityV1` record for that endpoint is admissible.

For this first runtime slice, the closed handler subset consists of synchronous
`this.<canonical-state>` updates whose operands are already represented by
`ParsedStateOperation`. This subset has no free-variable capture, no server
import, and no cancellation parameter; those facts are supplied explicitly to
`ActionAuthorityV1`. Async handlers and any broader capture or cancellation
form remain rejected until their owning analysis/lowering exists.

The adapter then emits ordinary `ComponentAction` write records with the field
name as a compatibility display/binding label and the action-endpoint as their
owner. It does not emit a `ComponentMethod`.

## Consumer migration

`ActionBatch` is generalized from `authored_action_method` to
`authored_action_endpoint`. Template manifests, ordinary-instance event
records, ASM validation, lazy chunks, effect triggers, and runtime code use
that endpoint ID as the executable binding key. Their source-facing label may
remain the action field name during the schema-compatible migration.

An event reference resolves only when its exact `this.field` label joins one
endpoint and one action batch in the same component. A missing, ambiguous, or
method-only fallback is a validation failure. Legacy method endpoints continue
to resolve through the same endpoint lookup.

## Acceptance

- A canonical V2 `action(() => { this.count += 1; })` bound from render emits
  the same ordered write/batch behavior as a legacy decorated action, without
  adding a method record. The focused compiler and CLI fixtures exercise this
  projection with an installed TypeScript authority bridge.
- The V2 endpoint, individual writes, state storage ID, action batch, event
  binding, and resume boundary remain identical across cold execution and
  resumed execution.
- Async handlers, free captures, server imports, unsupported statements,
  parameter assignments, lookalike calls, missing state ownership, and
  inadmissible authority facts fail before publication.
- Existing decorated-action fixtures retain their method IDs and generated
  products unchanged.

The real-browser fixture builds a decorator-free component, confirms its cold
event update, restores a compiler-authored resume snapshot, confirms the
`action-endpoint:increment` binding identity, and performs the resumed update.
It is acceptance evidence for the closed synchronous subset only, not a
translation authority for broader JavaScript handlers.
