# Runtime Model

## Runtime philosophy

The runtime should be a small execution substrate for compiler decisions. It should not be a large framework that discovers application structure in the browser.

Primary runtime responsibilities:

1. Signal storage and invalidation.
2. Scheduling updates.
3. Applying DOM patches.
4. Delegating events.
5. Loading lazy chunks.
6. Resuming serialized state.
7. Coordinating form/action state.
8. Upgrading custom elements when needed.
9. Exposing development trace hooks.

## Non-goals

The runtime should not:

- use virtual DOM diffing as its default update mechanism,
- re-run component render functions for ordinary local state changes,
- hydrate whole component trees just to attach handlers,
- require manual memoization for ordinary computed values,
- include a large router/data layer in the browser by default,
- force custom elements for internal app rendering unless target requires it.

## Runtime modules

```txt
runtime/
  signal.ts
  scheduler.ts
  patch.ts
  events.ts
  lazy.ts
  resume.ts
  forms.ts
  custom-elements.ts
  dev-trace.ts
```

Each module should be independently tree-shakable.

## Signal engine

Minimal model:

```ts
type Signal<T> = {
  get(): T;
  set(value: T): void;
  subscribe(effect: Effect): Unsubscribe;
};
```

The public authoring API does not need to expose this shape everywhere. Class fields can compile to signals.

Signal engine requirements:

- synchronous local propagation by default,
- batched DOM patch scheduling,
- deterministic ordering,
- no hidden component re-execution,
- dev-mode dependency tracing,
- support for serialized initial values.

## Scheduler

The scheduler coordinates:

- state updates,
- DOM patches,
- event phases,
- async resource updates,
- form pending/error state,
- transition hooks.

Default policy:

- update state synchronously,
- batch DOM patches in microtasks where safe,
- preserve input responsiveness,
- allow eager patches for input bindings and form controls.

## DOM patching

The compiler should emit concrete patch instructions:

```ts
patchText(nodeRef.b0, state.count);
setAttr(nodeRef.n2, "aria-expanded", state.open);
replaceBranch(branchRef.editor, renderEditor);
```

Runtime DOM operations should be boring, explicit, and benchmarkable.

Patch modes:

- text data update,
- attribute set/remove,
- property set,
- class toggle,
- style property set,
- branch replace,
- keyed list move/insert/remove,
- slot projection update,
- form control value sync.

## Event delegation

Default event model:

- one delegated listener per event type per root where possible,
- handler metadata encoded in HTML or manifest,
- lazy import on first interaction,
- resume state before invoking handler,
- preserve native event semantics.

Example compiled HTML:

```html
<button data-ez-on="click:c0.e0">Edit</button>
```

Runtime flow:

```txt
click event
  ↓
find event marker
  ↓
resolve handler chunk
  ↓
load chunk if absent
  ↓
resume required state
  ↓
invoke handler
  ↓
patch affected bindings
```

## Lazy import resolver

Responsibilities:

- map event/resource identifiers to chunks,
- dedupe concurrent imports,
- prefetch when compiler hints say useful,
- expose chunk-load errors to boundaries,
- work under CSP constraints.

Manifest example:

```json
{
  "events": {
    "c0.e0": "/assets/counter.increment.abc123.js"
  }
}
```

## Resumability loader

Responsibilities:

- parse serialized state,
- attach event delegation without full component hydration,
- map DOM markers to component/binding identifiers,
- load code on demand,
- restore state slots needed by the interaction.

Serialized state should be minimal and secure. Sensitive server data must not be serialized unless explicitly allowed.

## Forms runtime

The forms runtime should be optional and small.

Responsibilities:

- intercept enhanced form submissions,
- preserve native fallback,
- manage pending/error state,
- stream validation errors where supported,
- patch associated error regions,
- support optimistic updates,
- rollback on failure where configured.

## Custom-element upgrader

For Web Component targets:

- define custom elements,
- map attributes/properties,
- preserve slots and parts,
- coordinate shadow DOM where enabled,
- handle form-associated custom elements if requested,
- lazy-upgrade components where possible.

App-mode rendering should not be forced through custom elements unless it is the chosen target.

## Development trace hooks

Dev-only hooks should report:

- state write,
- dependency invalidation,
- binding patch,
- chunk load,
- event resume,
- resource update,
- form submit lifecycle,
- accessibility runtime warnings when static proof is impossible.

Trace output:

```txt
state write: CheckoutForm.submit.pending false -> true
invalidates: binding b12, attribute disabled on button#n8
patch: setProperty(button#n8, "disabled", true)
source: src/routes/checkout.tsx:31:17
```

## Runtime test matrix

Test runtime behavior under:

- no JavaScript,
- slow JavaScript,
- chunk-load failure,
- CSP restrictions,
- shadow DOM retargeting,
- nested forms invalid authoring diagnostics,
- back/forward cache,
- streaming server response interruption,
- concurrent form submissions,
- network offline during action,
- SSR to client transition,
- custom-element consumer environments.
