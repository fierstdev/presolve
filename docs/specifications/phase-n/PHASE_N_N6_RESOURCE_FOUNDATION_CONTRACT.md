# Phase N N6 Resource foundation contract

N6 introduces a compiler-owned Resource only through a declaration with a
stable identity, explicit execution boundary, declared serializable data/error
types, compiler-derived input dependencies, lifecycle state, cancellation
policy, retry/invalidation policy, runtime activation record, and resume codec.
Arbitrary Promises, `fetch` calls inside Computed/render, implicit server
execution, raw cache access, and generic async callbacks are not Resources.

The first implementation slice must construct identity/lifecycle products before
admitting a source decorator or capability. That ordering prevents type-only
`Resource` metadata from being mistaken for executable semantics.
