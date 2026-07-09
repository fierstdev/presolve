# Product Strategy

## Strategic thesis

The next serious competitor to today’s meta-frameworks will not win by combining familiar primitives. “Web Components + TSX + signals” is not enough. The defensible product is a compiler that understands UI deeply enough to make the optimal delivery model the default.

EdgeZero is therefore not “a framework with a compiler.” It is a **compiler-centered web authoring system**.

The compiler owns a richer semantic model than the syntax exposes. TSX, `html`` templates`, class components, signals, resources, actions, and Web Components are authoring/output surfaces over a semantic UI model.

## North-star statement

> The browser receives only what it needs. The developer writes coherent web UI code. The compiler absorbs the accidental complexity.

## Product wedge

Initial wedge:

> HTML-first, resumable forms and components with precise updates and explainable compiler output.

This wedge is more credible than “replace React” as an initial market claim. It creates clear technical differentiation and maps to real pain in dashboards, SaaS CRUD apps, marketing sites with interactive regions, commerce flows, public-sector forms, and component libraries.

## Target users

### Primary early users

1. Performance-sensitive frontend engineers.
2. Design-system teams that need standards-native distribution.
3. Full-stack teams tired of manual server/client boundaries.
4. SaaS teams with heavy forms and authenticated dashboards.
5. Agencies building content-heavy sites with selective interactivity.

### Later users

1. Enterprise teams with accessibility requirements.
2. Platform teams consolidating UI delivery across products.
3. Teams building embeddable widgets and Web Component libraries.
4. AI-assisted UI generation systems that need a semantic compiler target.

## Core promise by audience

### For application developers

- Write normal components.
- Ship HTML first.
- Avoid hydration as the baseline.
- Avoid memoization as a daily concern.
- Keep data, actions, and UI local in one authoring artifact.
- Debug what the compiler inferred.

### For platform teams

- Enforce accessibility and security rules at compile time.
- Generate predictable bundles and source maps.
- Export standards-native components.
- Support multiple deployment targets from one source.
- Analyze size, interactivity, and server/client ownership before production.

### For design-system teams

- Author with rich framework ergonomics.
- Publish Web Components.
- Preserve attributes, properties, slots, parts, CSS custom properties, and form-associated behavior.
- Avoid locking consumers into a single frontend framework.

## Differentiation

The differentiation is the unification of five ideas usually distributed across separate systems:

1. HTML-first delivery.
2. Fine-grained reactivity.
3. Resumability.
4. Standards-native component output.
5. Explainable compiler intelligence.

The product must avoid becoming a checklist framework. The compiler has to prove these claims through visible output:

```bash
edgezero explain src/components/CheckoutForm.tsx
edgezero size --by-interaction
edgezero why client-js
edgezero a11y
edgezero trace --binding b42
```

## Principles for saying “no”

Reject or demote features that cannot satisfy most of these criteria:

1. Can the compiler understand it?
2. Can the compiler explain it?
3. Can it degrade to HTML where possible?
4. Can it avoid client JavaScript where possible?
5. Can it be debugged from source to DOM?
6. Can it interoperate with the platform?

If a feature fails the test but is still needed, mark it as an escape hatch and make the cost visible.

## Strategic anti-goals

EdgeZero should not be:

- a virtual-DOM framework,
- a React clone with Web Component output,
- a Svelte clone with classes,
- a Qwik clone with less explicit syntax,
- a Lit wrapper,
- a purely server-driven LiveView clone,
- an accessibility lint plugin,
- a build tool with branding.

It should be a system where the compiler owns UI semantics end-to-end.

## Positioning hierarchy

### Short pitch

> Write components. Ship HTML first. Load JavaScript only when the user needs it.

### Longer pitch

> EdgeZero is a compiler-first web framework for building resumable, accessible, standards-native interfaces. It analyzes your templates, state, events, resources, styles, accessibility, and server/client boundaries, then ships the smallest useful HTML and JavaScript for each interaction.

### Founder thesis

> Current frameworks make developers coordinate performance, hydration, memoization, data loading, accessibility, and server/client splitting manually. EdgeZero makes those compiler responsibilities.

## What must feel different in the first demo

The first demo must show more than a counter.

A credible demo should include:

1. Server-rendered form with native fallback.
2. Client-enhanced submit with lazy-loaded handler.
3. Fine-grained validation updates.
4. Compiler accessibility diagnostics.
5. `edgezero explain` output mapping source to DOM, state, chunks, and fallback behavior.
6. Export of the same component as a Web Component package.

The demo’s emotional target is:

> “I wrote one coherent artifact, and the compiler turned it into the deployment I would have hand-designed if I had a week.”
