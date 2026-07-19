# Competitive Notes

This document should remain grounded in public docs and observed framework behavior. It is not a marketing attack document. Its purpose is to define what EdgeZero should learn, copy, avoid, and exceed.

## React

### What to learn

React wins through component composition, ecosystem gravity, and ordinary JavaScript expressiveness. React Compiler’s direction validates that automatic optimization and memoization are now considered a core developer-experience issue.

### Steal

- component composition,
- language expressiveness,
- ecosystem pragmatism,
- adapter strategy,
- mature error boundaries and dev tooling expectations.

### Avoid

- rerender-as-default mental model,
- hydration as unavoidable baseline,
- hooks-style invisible ordering constraints,
- user-managed memoization as normal practice,
- dependency-array reasoning as everyday work.

### EdgeZero response

Make “state changed → exact DOM bindings update” obvious, inspectable, and default.

## Svelte

### What to learn

Compiler-first authoring can reduce runtime work and simplify syntax. But compiler magic must have boundaries and explanations.

### Steal

- compile-time DOM generation,
- concise authoring,
- small runtime philosophy,
- willingness to make compiler-visible reactivity primitives.

### Avoid

- magic that only works in special file types,
- migration churn that invalidates old mental models,
- rules that are hard to explain outside the compiler.

### EdgeZero response

Be magical at output time, not mysterious at authoring time.

## Solid

### What to learn

Fine-grained reactivity is the right local update model for precise UI updates.

### Steal

- signal graph,
- precise DOM binding updates,
- no virtual DOM diff as default,
- composable primitives.

### Avoid

- leaking too many low-level primitives into app code,
- making users decide manually where every reactive boundary belongs.

### EdgeZero response

Provide Solid-like precision with Svelte-like compiler ergonomics and Web Component output.

## Qwik

### What to learn

Resumability is a category-level idea: avoid replaying server work and loading all app logic before interaction.

### Steal

- resumability,
- lazy event-handler loading,
- serialized server/client boundaries,
- HTML as continuation format.

### Avoid

- visible optimizer ceremony,
- making users think about serialization too often,
- APIs that primarily serve optimizer constraints.

### EdgeZero response

Make resumability a compiler feature with direct diagnostics, not a user ceremony.

## Astro

### What to learn

Most pages should not pay SPA complexity tax. HTML-first delivery and selective interactivity are correct defaults.

### Steal

- static HTML by default,
- partial interactivity,
- route/page-level performance discipline,
- multi-framework pragmatism.

### Avoid

- fragmented island state where app continuity is needed,
- making interactivity boundaries too manual.

### EdgeZero response

Infer interactivity boundaries from the component graph and preserve state continuity where needed.

## Lit

### What to learn

Web Components are a strong distribution target. Reactive properties, declarative templates, and no-build usage are useful qualities.

### Steal

- platform alignment,
- attributes/properties/slots/parts discipline,
- small runtime philosophy,
- standards-native component APIs.

### Avoid

- requiring users to hand-author too much platform plumbing,
- treating optional compilation as an ergonomic ceiling.

### EdgeZero response

Use Lit’s platform discipline with a compiler that removes boilerplate and emits stronger artifacts.

## Angular and Vue

### What to learn

Integrated systems matter: routing, forms, data, tooling, and official conventions reduce decision fatigue. Signals and reactivity models validate dependency tracking as a mainstream framework direction.

### Steal

- integrated batteries,
- serious tooling,
- forms/data conventions,
- strong team-scale posture.

### Avoid

- large conceptual surface area,
- framework-specific worldviews with expensive escape hatches,
- proxy/deep-reactivity surprises,
- too much runtime abstraction weight.

### EdgeZero response

Offer integrated batteries that compile away where possible.

## Marko

### What to learn

Streaming and async rendering can be language/compiler-level problems, not bolted-on server framework behavior.

### Steal

- streaming-first mental model,
- async rendering primitives,
- compiler-owned rendering correctness.

### Avoid

- niche language feel that slows adoption.

### EdgeZero response

Make async UI streaming first-class while keeping TSX/html-template authoring familiar.

## Elm

### What to learn

A system can make classes of bugs structurally hard. Compiler diagnostics and architectural constraints can create confidence.

### Steal

- explicit state transitions where helpful,
- excellent diagnostic philosophy,
- strong model clarity.

### Avoid

- ecosystem isolation,
- excessive purity barriers for ordinary web work.

### EdgeZero response

Offer Elm-like confidence without forcing a new language.

## HTMX and LiveView

### What to learn

HTML, links, forms, server-rendered fragments, and hypermedia interactions remain powerful. Not every app should become a client-side SPA.

### Steal

- HTML as an application protocol,
- forms and links as real primitives,
- server-rendered interaction where appropriate,
- low-JavaScript paths.

### Avoid

- treating every workflow as a client-state/cache-invalidation problem,
- making simple CRUD pay SPA complexity tax.

### EdgeZero response

Support client-rich, server-rich, and hybrid interaction from one component model.

## Competitive matrix

| Competitor | Strongest lesson | EdgeZero differentiator |
|---|---|---|
| React | Composition and ecosystem | no hydration baseline, no manual memoization baseline, explainable fine-grained updates |
| Svelte | Compiler-first UI | resumability, multi-target output, semantic inspector |
| Solid | Fine-grained reactivity | compiler ergonomics, resource/action/form graph ownership |
| Qwik | Resumability | less serialization ceremony, class/template locality |
| Astro | HTML-first islands | inferred interactivity and state continuity |
| Lit | Web Component discipline | compiler-owned DX and server/resume/form semantics |
| Angular/Vue | Integrated systems | smaller runtime/mental footprint, compile-away batteries |
| Marko | Streaming | familiar TSX/html authoring with broader graph ownership |
| HTMX/LiveView | HTML/server interaction | richer client component model when needed |

## Strategic conclusion

Do not build another frontend framework.

Build a compiler whose framework surface lets developers author coherent web UI while the compiler owns delivery, reactivity, resumability, accessibility, resources, styling, and debug explainability.
