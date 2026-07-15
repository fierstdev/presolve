# Component contract

Phase H freezes Components and Slots as compiler-owned composition. A component
definition, a component invocation, and a component instance are distinct
canonical identity domains. The compiler resolves every relationship and emits
closed plans; the browser runtime executes those plans without discovering
components, Slots, Providers, parents, or ownership.

## Language syntax

A component remains an explicitly decorated class:

```tsx
@component("x-card")
class Card extends Component {
  @slot()
  children!: SlotContent;

  @slot()
  header!: SlotContent;

  render() {
    return <article><slot name="header" /><slot /></article>;
  }
}
```

`@slot()` accepts no arguments and applies only to a non-static,
declaration-only, definite-assignment field of exact built-in type
`SlotContent`. The field name is the Slot name. `children` is the default Slot;
all other valid fields are named Slots.

A PascalCase JSX identifier invokes a local or imported component definition.
Named content uses a direct child `<template slot="name">` wrapper. Other
direct children are default content. `<slot />` is the default outlet and
`<slot name="name" />` is a named outlet. These wrappers and outlets are
compiler directives, not runtime name-matching APIs.

## Canonical identity domains

| Domain | Canonical identity |
| --- | --- |
| Component definition | `SemanticId` ending in `component:<name>` |
| Component invocation | `ComponentInvocationId` |
| Component build root | `ComponentRootId` |
| Component instance | `ComponentInstanceId` |
| Structural region | `ComponentStructuralRegionId` |
| Slot declaration | `SlotId` |
| Slot content | `SlotContentFragmentId` |
| Slot outlet | `SlotOutletId` |
| Slot binding | `SlotBindingId` |
| Provider instance | `ProviderInstanceId` |
| Consumer instance | `ConsumerInstanceId` |
| Instance Context source | `ContextSourceInstanceId` |
| Instance Context value | `InstanceContextValueSlotId` |

Definition identity never substitutes for invocation or instance identity.
Repeated invocations of one definition create distinct instance, storage,
cache, Context, runtime, and resume identities. Conditional and keyed-list
component uses retain a structural-region identity; an index is never a
component identity fallback.

## Canonical authorities

| Concern | Sole authority |
| --- | --- |
| Slot declarations | H1 Slot declaration candidates and `SlotEntity` |
| Invocation resolution | H2 `ComponentInvocationEntity` |
| Slot fragments and outlets | H3 canonical content/outlet products |
| Instance planning and identity | H4 `ComponentInstancePlan` |
| Executable ancestry | H5 `ComponentInstanceScopeGraph` |
| Instance Context selection | H6 `InstanceContextRegistry` |
| Slot binding | H7 `SlotBindingRegistry` |
| Type and boundary compatibility | H8 `CompositionTypeProducts` |
| Composition cycles | H9 `ComponentCompositionAnalysis` |
| Initialization scheduling | H10 `ComponentInitializationPlan` |
| Component and Slot IR | H11 `ComponentIrReport` |
| Component IR optimization | H12 `OptimizedComponentIrReport` |
| Runtime metadata | H13 `RuntimeComponentRegistry` |
| Runtime serialization | H14/H16 `RuntimeComponentArtifact` |
| Initial browser execution | H15 closed runtime tables |
| Structural updates | H16 compiler structural programs and template plans |
| Resume metadata | H17 shared resume plan and manifest |
| Inspection | H18 shared ASM inspection projection |
| Diagnostics | H19 canonical component diagnostic projector |
| Fixtures and determinism | H20 component fixture matrix |

Tag resolution occurs only while H2 lowers canonical template entities. Slot
names are interpreted only while H3 creates fragments/outlets and H7 creates
exact bindings. Instance and structural-region IDs are constructed only by H4.
Later compiler, CLI, runtime, and resume products consume those typed facts.

## Ownership and Slot placement

Slot content is authored and evaluated in the caller component and remains
owned by the caller instance. Its rendered placement is the exact outlet of the
callee instance selected by the H7 binding. A binding carries compiler IDs for
the caller, callee, Slot, fragment, and outlet. Runtime placement never matches
a Slot name or rebinds caller expressions to the callee.

Unknown Slots, duplicate content, duplicate outlets, missing outlets, invalid
ownership, and blocked invocations remain explicit compiler facts. An empty
valid Slot is also explicit and does not acquire fallback content.

## Instance Context

Phase G declaration-level Context products remain inspectable, but component
runtime products use H6 instance-qualified resolution. For each Consumer
instance the compiler walks the canonical H5 scope graph from self to root,
selects the nearest exact Provider-instance scope, preserves same-scope
ambiguity, and otherwise uses a root-qualified Context default.

Typing, serialization, boundary, lifetime, lowering, or runtime failure never
reselects a Provider. Every executable Consumer binding carries its exact
selected source and `InstanceContextValueSlotId`. Two instances of one Consumer
definition may therefore bind to different Provider instances without sharing
a declaration-level runtime slot.

## Runtime ordering and closed tables

Cold component boot is compiler ordered:

```text
validate artifacts
  -> create instances in compiler batches
  -> initialize instance-local storage and caches
  -> install instance Context and Slot bindings
  -> materialize compiler template regions
  -> expose deterministic debug evidence
  -> execute initial Context sources
  -> execute initial effects
```

Completed actions retain the established ordering:

```text
State writes
  -> Computed update batches
  -> Context update batches
  -> structural selectors
  -> outgoing child-before-parent destruction
  -> incoming parent-before-child creation
  -> Context and Slot bindings
  -> DOM materialization
  -> effects for newly active work
```

The browser receives closed ID-keyed tables for component instances, Slot
bindings, instance Context bindings, and structural regions. Structural DOM
changes use compiler-emitted conditional/list anchors and structural programs;
they do not use generic virtual-DOM diffing. Runtime debug evidence exposes
initialization runs, the instance tree, Slot binding runs, and failures without
becoming semantic authority.

## Frozen schemas

Phase H completes with these actual versions:

| Serialized boundary | Version |
| --- | ---: |
| Component runtime artifact | 2 |
| Resume manifest | 4 |
| ASM inspection | 8 |
| Check JSON | 4 |
| Template manifest | 2 |
| Context runtime artifact | 2 |
| Semantic graph | 5 |

The internal runtime component registry contract is version 1. Component
runtime artifact v1 was the H14 initial-execution shape; H16 advanced it to v2
for structural programs. Template-manifest v1 remains a legacy output for
sources that do not require v2 action-batch metadata. Phase H adds no component
nodes to frozen semantic-graph schema v5 and no instance-qualified fields to
Context runtime artifact v2; those bindings live in the component artifact.

## Diagnostics

The frozen component diagnostic range is:

| Code | Meaning |
| --- | --- |
| `EZC1068` | Invalid Slot declaration |
| `EZC1069` | Invalid component invocation |
| `EZC1070` | Unresolved component symbol |
| `EZC1071` | Component composition cycle |
| `EZC1072` | Component inheritance is unsupported |
| `EZC1073` | Inherited semantic declaration is unsupported |
| `EZC1074` | Unknown Slot |
| `EZC1075` | Duplicate Slot content |
| `EZC1076` | Duplicate Slot outlet |
| `EZC1077` | Missing Slot outlet |
| `EZC1078` | Invalid Slot content ownership |
| `EZC1079` | Slot type or boundary incompatibility |
| `EZC1080` | Component instance planning failure |
| `EZC1081` | Instance-aware Context binding unavailable |
| `EZC1082` | Structural region cannot be planned |
| `EZC1083` | Component or Slot source cannot be lowered |

Check, full ASM, selected-entity ASM, and explain use the same canonical
diagnostic vector. Diagnostics never reparse source, reconstruct relationships,
or fabricate identities for declarations and targets that did not lower.

## Unsupported semantics

The frozen Phase H contract does not support:

- dynamic component expressions;
- component props, spread props, callbacks as component arguments, or dynamic
  component argument transport;
- component inheritance, user-authored base classes, mixins, traits, or
  inherited EdgeZero semantics;
- fallback Slot content, required Slots, duplicate Slot outlets, or Slot
  forwarding;
- runtime Slot matching or runtime component discovery;
- generic virtual-DOM diffing;
- index-keyed component identity;
- lifecycle cleanup;
- async component initialization;
- portals or teleportation;
- multi-parent component instances;
- server or shared component boundaries;
- live component restoration before Phase J.

Unsupported authored forms remain retained compiler facts and diagnostics where
the frozen catalog defines one. The runtime never falls back to tag names, Slot
names, DOM ancestry, Provider search, source parsing, or replacement-component
discovery. Any new component behavior requires an explicit later roadmap slice
and a deliberate schema/version review.
