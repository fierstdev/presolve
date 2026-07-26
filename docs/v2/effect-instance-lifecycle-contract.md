# V2 effect instance-lifecycle contract

V2 effect cleanup is per component *instance*, not per component declaration.
This contract closes the ownership gap between the declaration-keyed effect
artifact and the existing compiler-owned component-instance topology.

## Problem boundary

`ComponentInstancePlan` and the runtime component artifact already identify
each static instance, its parent, depth, and structural destroy order. The
current effect artifact is keyed only by `SemanticId::effect`, so a cleanup
registry keyed by that declaration cannot distinguish repeated component
instances. It must therefore remain unavailable for cleanup-bearing V2 fields.

## Required instance projection

The compiler instance-effect registry now adds one canonical
instance-execution record per planned component instance and matching V2 effect
declaration. The next effect artifact schema serializes those records. Each record
must contain:

- an instance-qualified effect execution identity;
- the immutable effect declaration ID and owning component instance ID;
- parent instance ID and depth from `ComponentInstancePlan`;
- declaration-order position supplied by the V2 effect semantic entity; and
- the compiler-generated main and cleanup program references.

The projection may only join existing canonical effect, component-instance,
and IR facts. It must not infer ownership from DOM positions, class names,
effect names, or source text.

## Runtime requirements

The browser runtime must key subscriptions, cleanup registrations, evidence,
and capability execution context by the instance-qualified effect identity.
Initial activation follows parent-before-child component initialization.
Structural removal and explicit application disposal use the component
artifact's child-before-parent destroy order; within one component instance,
cleanups run in reverse V2 field declaration order.

An effect program that reads State, computed values, Context, or Slots must
receive its owning instance context explicitly. Until that execution context
is emitted and consumed, a cleanup-bearing V2 field remains a compile-time
error. Server publication and execution remain prohibited.

## Incremental acceptance

1. The compiler registry emits deterministic instance-effect records for
   nested and repeated components, including parent/depth and field order.
2. The generated browser runtime validates every record against the component
   artifact before activation.
3. A repeated-child fixture proves independent cold runs, rerun cleanup, and
   cleanup storage for two instances of the same declaration.
4. A nested fixture proves child-before-parent cleanup on structural removal
   and application disposal.
5. A resume fixture proves one eligible V2 run per restored instance, with no
   legacy decorator behavior change and no server execution.
