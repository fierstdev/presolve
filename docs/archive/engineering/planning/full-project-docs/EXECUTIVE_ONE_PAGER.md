# EdgeZero Executive One-Pager

## Name

Recommended working name: **EdgeZero**  
Primary domain: **EdgeZero.dev**  
Brand promise: **Zero wasted JavaScript.**

Legal/trademark diligence is required before public launch because “Edge Zero” is already used outside the web-framework category.

## Category

EdgeZero is a **compiler-centered web authoring system**.

It is not merely a frontend framework and not merely a compiler plugin. The compiler understands UI semantics deeply enough to decide what should be HTML, what should stream, what should resume, what should lazy-load, what should update, and what should never ship to the browser.

## Core pitch

> Write ordinary components. Ship HTML first. Load JavaScript only when the user needs it. Get precise updates without manual memoization. Export standards-native components.

## Strategic thesis

The next major web framework competitor will not win by saying “Web Components + TSX + signals.” That is table stakes. The differentiator is compiler-owned intent:

- semantic UI graph,
- reactive dependency graph,
- resumability graph,
- accessibility graph,
- resource/data graph,
- style graph,
- server/client split graph,
- lazy-loading graph,
- debug graph.

## Product shape

```txt
Authoring:
  TSX and html`` templates
  class-based components
  state/resources/actions/forms
  Web Components as output target

Compiler:
  semantic graph analysis
  fine-grained update planning
  SSR/streaming/resumability
  server/client splitting
  accessibility and style validation
  chunking and explain metadata

Runtime:
  tiny scheduler
  signal engine
  event delegation
  DOM patching
  lazy import resolver
  resumability loader
  optional custom-element upgrader
```

## Differentiation

EdgeZero unifies five ideas that usually live separately:

1. HTML-first delivery.
2. Fine-grained reactivity.
3. Resumability.
4. Standards-native Web Component output.
5. Explainable compiler intelligence.

## First wedge

Build the best compiler-owned forms and actions story:

- native form fallback,
- enhanced submit,
- server actions,
- resource invalidation,
- pending/error state,
- accessibility by construction,
- lazy JS only on interaction,
- explain output that shows exactly what happened.

## First serious demo

A user-profile or checkout flow:

- SSR HTML rendered initially,
- no component JS until interaction,
- click/edit loads a small handler chunk,
- submit uses native fallback without JS,
- enhanced submit with JS streams validation errors,
- compiler catches missing labels/buttons,
- same component can export as a Web Component,
- `edgezero explain` shows state, bindings, events, chunks, accessibility, and fallback behavior.

## Non-negotiables

- No virtual DOM as default.
- No mandatory full hydration.
- No manual memoization as normal practice.
- No accessibility as optional lint.
- No opaque compiler magic.
- No closed ecosystem.
- No “Rust” as the main selling point.

## MVP requirements

1. TSX class components.
2. Compiler-built template graph.
3. Fine-grained state-to-binding updates.
4. SSR HTML output.
5. Lazy event handler loading.
6. Native form fallback plus enhanced server action.
7. Basic resource primitive.
8. Accessibility diagnostics.
9. `edgezero explain`.
10. Simple Web Component output.

## Public tagline candidates

- HTML first. JavaScript when needed. Compiler by default.
- Write components. Ship intent.
- Zero wasted JavaScript.
- The web compiler for resumable, accessible interfaces.
