# Phase N N6-A Resource identity and lifecycle foundation

N6 introduces a compiler-owned Resource only through a declaration with a
stable identity, explicit execution boundary, declared serializable data/error
types, compiler-derived input dependencies, lifecycle state, cancellation
policy, retry/invalidation policy, runtime activation record, and resume codec.
Arbitrary Promises, `fetch` calls inside Computed/render, implicit server
execution, raw cache access, and generic async callbacks are not Resources.

N6-A establishes the first executable compiler product for that family:
`ResourceDeclaration` and its component-instance-qualified
`ResourceActivation`. This is deliberately *not* source syntax or a runtime
feature. The `resources` capability remains deferred until the compiler can
lower a declaration through endpoint/capability selection, activation,
cancellation, artifact publication, and resume restoration.

## Identity and ownership

`ResourceId::for_owner(component, name)` is a declaration identity in the
component's semantic namespace:

```text
component:x-profile/resource:profile
```

The declaration never doubles as the mutable execution record. Each component
instance receives a separately stable `ResourceActivationId`:

```text
root:component:x-profile/resource-activation:component:x-profile/resource:profile
```

That separation is required for repeated component instances and future keyed
structural ownership. A source-level key is part of a declaration, not a DOM
position or an ambient cache key.

## Declaration contract

`ResourceDeclaration::new` requires a non-empty name and key, serializable
data and error types, an explicit `Client`, `Server`, or `Shared` boundary,
compiler-derived input dependency identities, a retry policy, an invalidation
policy, and source provenance. Nonserializable data or error declarations are
rejected before any activation can exist.

N6-A admits only two policy vocabulary values for each concern:

| Concern | Values |
| --- | --- |
| Retry | `Never`, `ExplicitOnly` |
| Invalidation | `OnInputChange`, `ExplicitOnly` |

These values are descriptive foundation data, not a hidden scheduler. The
later lowering slice owns any timing, backoff, transport, or cache behavior.

## Lifecycle contract

An activation starts `Idle`. The only valid transitions are:

```text
Idle --Activate--> Pending(generation 1)
Pending(n) --Resolve--> Ready(n)
Pending(n) --Reject--> Failed(n)
Pending(n) --Cancel--> Cancelled(n)
Ready(n) --Invalidate--> Pending(n + 1)
Failed(n) --Invalidate--> Pending(n + 1)
Cancelled(n) --Activate|Invalidate--> Pending(n + 1)
```

Every other transition is rejected by the compiler product. A terminal result
can therefore never be attached to a cancelled or superseded generation.

## Deferred surface

N6-A does not admit `@resource`, `resource(...)`, `fetch`, `Promise`, server
actions, a generic cache, or runtime artifacts. It creates no compatibility
shim and no invisible JavaScript async runtime. A future N6 slice must add all
of the following as one canonical path:

1. compiler-recognized source form;
2. registered service endpoint or capability identity;
3. deterministic input-dependency plan;
4. activation and cancellation artifact;
5. serializable data/error resume codec; and
6. browser/server boundary execution proof.

Until that happens, `Resource<T, E>` remains type metadata and
`ResourceDeclaration` is an internal semantic planning product—not an
application-authoring capability.
