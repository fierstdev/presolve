# Semantic Graph Model

EdgeZero’s advantage depends on the compiler building a semantic UI model. The graph layer is the product’s technical center.

## Graph 1: Template graph

Represents actual DOM structure and dynamic regions.

Tracks:

- static elements,
- dynamic text bindings,
- dynamic attributes/properties,
- conditional branches,
- lists/keying semantics,
- slots,
- shadow DOM boundaries,
- form control relationships,
- static subtree eligibility,
- hydration/resume requirements.

Example:

```tsx
<button onClick={increment}>Count: {count}</button>
```

Template graph excerpt:

```txt
Element button#n0
  static text "Count: "
  dynamic text binding b0
  event click e0
```

## Graph 2: Reactive graph

Represents state producers, derived values, effects, and DOM consumers.

Tracks:

- state fields,
- derived expressions,
- resource snapshots,
- actions and pending/error states,
- template bindings,
- invalidation edges,
- update modes.

Output example:

```txt
count -> b0
b0 update mode: Text.data patch
```

Policy: changing state should update exact bindings, not re-run the component by default.

## Graph 3: Event graph

Represents user interactions and code required to handle them.

Tracks:

- event type,
- target element,
- handler reference,
- captures/closures,
- client/server eligibility,
- lazy chunk,
- resumability metadata,
- fallback behavior.

Compiler questions:

- Can this handler be lazy-loaded?
- Can it resume from serialized state?
- Does it capture server-only values?
- Does it need eager registration?
- Does it require custom event retargeting for shadow DOM?

## Graph 4: Serialization graph

Represents what can cross server/client boundaries.

Tracks:

- serializable state,
- resource snapshots,
- action references,
- closures and captures,
- non-serializable values,
- server-only imports,
- client-owned mutable state,
- security redactions.

Diagnostics should be specific:

```txt
Cannot resume handler UserProfile.edit because it captures db.users.
Move db access into a server action or mark the branch server-only.
```

## Graph 5: Resource/data graph

Represents data dependencies and invalidation.

Tracks:

- route params,
- resource functions,
- server/client execution eligibility,
- cache keys,
- stale times,
- streamability,
- prefetch hints,
- action invalidations,
- consuming bindings/components.

This enables:

- automatic preloading,
- streaming data regions,
- server/client split,
- cache invalidation,
- resource-specific chunking,
- better devtools introspection.

## Graph 6: Accessibility graph

Represents semantic relationships and interaction obligations.

Tracks:

- accessible names,
- label/control associations,
- error/control associations,
- role validity,
- keyboard affordances,
- focus order,
- modal/focus-trap semantics,
- live-region usage,
- ARIA attribute compatibility,
- image alternative text requirements.

Compiler should produce errors, warnings, and fix suggestions.

## Graph 7: Style graph

Represents CSS ownership and usage.

Tracks:

- scoped styles,
- global styles,
- CSS custom properties,
- theme dependencies,
- selector usage,
- dead selectors,
- critical CSS,
- shadow DOM parts,
- container/query dependencies,
- animations/transitions.

Possible outputs:

- critical CSS per route,
- component CSS for WC package,
- dead-style diagnostics,
- theme-variable manifest.

## Graph 8: Component graph

Represents component dependencies and upgrade requirements.

Tracks:

- component imports,
- component usage sites,
- static-only components,
- interactive components,
- custom-element definition timing,
- lazy upgrade eligibility,
- adapter boundaries,
- foreign framework components.

This graph decides whether a component needs:

- no JS,
- lazy event JS,
- eager definition,
- custom-element registration,
- framework adapter runtime.

## Graph 9: Streaming graph

Represents regions that can flush, suspend, retry, or error independently.

Tracks:

- async regions,
- parent/child ordering,
- placeholders,
- error boundaries,
- retry policies,
- flush priority,
- dependency waterfalls,
- client reorder requirements.

Streaming should be an authoring primitive, not an advanced server trick.

## Graph 10: Debug graph

Maps source to generated output.

Tracks:

- source expressions,
- DOM nodes,
- generated bindings,
- chunks,
- network requests,
- update causes,
- server/client ownership,
- serialization state,
- accessibility checks.

This graph powers:

```bash
edgezero explain
edgezero why client-js
edgezero trace
edgezero inspect
```

## Graph 11: Server/client split graph

Represents execution ownership.

Tracks:

- server-only imports,
- browser-only APIs,
- universal code,
- server actions,
- client handlers,
- resource execution sites,
- edge/node constraints,
- captured values crossing boundaries.

The compiler should infer defaults and require annotations only at capability boundaries.

## Graph 12: Lazy-loading graph

Represents code required by route, resource, or interaction.

Tracks:

- entrypoints,
- event-triggered chunks,
- resource-triggered chunks,
- prefetch opportunities,
- dependency duplication,
- chunk captures,
- eager/lazy tradeoffs.

This enables a command like:

```bash
edgezero size --by-interaction
```

Example output:

```txt
Initial route /checkout
  HTML: 18.2kb
  CSS: 4.1kb
  JS: 0.9kb loader

Interaction: submit checkout-form
  chunk checkout.submit.js: 1.4kb
  chunk payment.validation.js: 2.1kb
```

## Graph invariants

The compiler should enforce these invariants:

1. Every dynamic binding has a known producer or is marked opaque.
2. Every client-resumable event has a valid serialization path.
3. Every server action has a safe invocation boundary.
4. Every form field has a semantic control identity.
5. Every generated chunk has an explanation path.
6. Every static subtree remains static unless a graph edge proves otherwise.
7. Every escape hatch is visible in explain output.
