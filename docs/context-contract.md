# Context contract

Phase G freezes Context as a compiler-owned semantic channel. The compiler is
the sole authority for Context identity, Provider selection, compatibility,
ownership, dependencies, lifetime, evaluation, lowering, runtime slots,
updates, inspection, and diagnostics. The runtime executes only immutable
compiler-generated programs and bindings.

## Canonical authorities

| Concern | Sole authority |
| --- | --- |
| Context identity | G1 `ContextEntity` / `ContextId` |
| Provider identity | G2 `ProviderEntity` / `ProviderId` |
| Consumer identity | G3 `ConsumerEntity` / `ConsumerId` |
| Context designator resolution | shared compile-time Context-designator resolver |
| Provider visibility and selection | G4 `ContextResolution` |
| Type, serialization, and boundary compatibility | G5 Context type products |
| Ownership | G6 `ContextOwnershipGraph` |
| Direct dependency topology | G7 `ContextDependencyGraph` |
| Lifetime compatibility | G8 `ContextLifetimeAnalysis` |
| Availability and evaluation order | G9 `ContextEvaluationPlan` |
| Context slots, source functions, and Consumer loads | G10 Context IR |
| Context source optimization | G11 `OptimizedContextIrReport` |
| Runtime metadata | G12 `RuntimeContextRegistry` |
| Runtime programs and update batches | G13/G15 Context runtime artifact |
| Initial execution | G14 runtime artifact evaluator |
| Completed-action updates | G15 `ContextUpdatePlan` |
| Resume metadata | G16 `ContextResumePlan` |
| Inspection | G17 `ContextInspectionRegistry` |
| Diagnostics | G18 retained candidates and canonical G4/G5/G8/G9/G10 products |
| Fixtures and determinism | G19 Context fixture matrix |

Provider and Consumer declarations share one compile-time Context-designator
resolver. Provider selection occurs only in G4 and is never retried because of
typing, serialization, boundary, lifetime, planning, lowering, or runtime
failure. Context defaults remain distinct value sources and never become
Providers.

## Runtime contract

Consumer access is keyed only by compiler-generated `ContextValueSlotId`
bindings. Runtime source evaluation follows compiler-emitted source programs,
evaluation batches, and action-batch update plans. It does not infer a binding,
Provider, owner, dependency, lifetime, or evaluation order.

Cold boot ordering is frozen as:

```text
State initialization
  -> Computed initialization
  -> Context source evaluation
  -> Context slot initialization
  -> Consumer bindings available
  -> initial render complete
  -> initial effects
```

Completed-action ordering is frozen as:

```text
Action state writes
  -> Computed update batches
  -> Context source update batches
  -> Context slot updates
  -> Consumer bindings observe current slots
  -> completed-action effects
```

Context updates are compiler-generated and keyed by `ActionBatchId`. Each
invalidated source executes at most once for the batch. Unrelated actions omit
the source. A source failure records failure evidence and preserves the exact
compiler binding; it never triggers fallback or reselection.

## Frozen schemas

Phase G completes with these versions:

| Serialized boundary | Version |
| --- | ---: |
| Semantic graph | 5 |
| Context runtime artifact | 2 |
| Template manifest | 2 |
| Resume manifest | 3 |
| ASM inspection | 6 |
| Check JSON | 3 |

The internal runtime Context registry contract remains version 1. Existing
template-manifest v1 compatibility for outputs without v2 effect action-batch
metadata is unchanged; Context adds no template-manifest fields.

## Unsupported semantics

The Phase G contract explicitly does not support:

- runtime Provider lookup or Provider reselection;
- runtime component-tree traversal or ancestry reconstruction;
- string Context lookup keys or implicit global Contexts;
- constructor injection;
- Consumer initializers or Consumer-authored fallbacks;
- Provider methods or getters;
- async Provider evaluation;
- cleanup or subscription lifecycles;
- dynamic Context designators;
- multiple same-Context Providers in one component;
- composition-derived cross-component visibility until Phase H supplies
  canonical compiler-owned scope edges;
- treating Context defaults as Providers;
- runtime dependency discovery or reverse dependency construction;
- live Context resume restoration before Phase J.

Any additional Context behavior requires a new explicit roadmap slice and must
preserve the compiler-only authority invariant.
