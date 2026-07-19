# Interop and Output Targets

## Interop thesis

A new framework cannot require the world to rewrite everything. EdgeZero must interoperate with the platform and with existing ecosystems.

## Required interop targets

1. Plain Web Components.
2. Lit components.
3. React components.
4. Vanilla JavaScript libraries.
5. CSS frameworks.
6. REST APIs.
7. GraphQL APIs.
8. RPC/server-action systems.
9. Existing server frameworks.
10. npm packages.
11. Design-system package consumers.

## Web Component output

Web Components are a native output target, not the whole authoring model.

```bash
edgezero build --target wc-library
```

Compiler should emit:

- custom element class,
- type declarations,
- attributes/properties manifest,
- slots manifest,
- CSS parts manifest,
- events manifest,
- form-associated behavior when requested,
- minimal runtime dependency.

Example manifest:

```json
{
  "tagName": "checkout-form",
  "properties": {
    "cartId": { "type": "string", "attribute": "cart-id" }
  },
  "slots": ["header", "footer"],
  "parts": ["button", "error"],
  "events": {
    "checkout-success": { "detail": "CheckoutResult" }
  }
}
```

## Consuming Web Components

EdgeZero should consume custom elements naturally:

```tsx
<stripe-pricing-table pricing-table-id="..." publishable-key="..." />
```

Compiler should treat unknown elements as platform elements unless configured with manifests.

If a manifest exists, the compiler can type-check attributes/properties/events.

## React interop

Initial policy:

- allow React islands through adapter,
- mark React subtree opaque for compiler guarantees,
- make JS cost visible,
- do not pretend React subtree is resumable unless adapter proves it.

Example:

```tsx
<ReactIsland component={LegacyChart} props={{ data: this.series }} />
```

Explain output:

```txt
Opaque interop: ReactIsland LegacyChart
Compiler guarantees inside subtree: reduced
Initial JS: legacy-chart.react.js 42kb
Reason: React adapter requires hydration for this island
```

## Lit interop

Lit components are custom elements, so normal platform interop should work. If a Lit component exposes a custom-elements manifest, use it for type checking and docs.

## Vanilla JS interop

Use explicit lifecycle or client-only blocks:

```tsx
<div ref={this.chartHost} clientOnly />
```

```ts
onClientMount(async () => {
  const { Chart } = await import("chart.js");
  new Chart(this.chartHost, options);
});
```

Opaque mutations should be marked so the debug and accessibility graphs know where confidence drops.

## CSS interop

Support:

- global CSS,
- scoped component CSS,
- CSS modules,
- CSS custom properties,
- design tokens,
- Tailwind-style utility CSS,
- vanilla CSS files,
- shadow DOM styles for WC output.

The compiler should not force a proprietary styling system.

## Server framework adapters

Adapters should target:

- Node HTTP,
- Express/Fastify/Hono-like runtimes,
- Vercel/Netlify/Cloudflare-style edge functions,
- static hosting,
- Bun/Deno where feasible.

Adapter responsibilities:

- SSR entry,
- streaming response,
- action routing,
- CSRF/session hooks,
- resource cache hooks,
- asset manifest serving.

## Output target matrix

| Target | HTML | JS default | Actions | Streaming | WC output | Primary use |
|---|---|---|---|---|---|---|
| static | build-time | none/lazy | no | limited | optional | marketing/docs |
| ssr | request-time | lazy | yes | optional | optional | apps |
| streaming-ssr | incremental | lazy | yes | yes | optional | dashboards/content |
| resumable-web | server-rendered | resume loader | yes | yes | optional | core target |
| wc-library | component HTML/JS | component runtime | optional | no/limited | yes | design systems |
| island | mostly static | per island | optional | optional | optional | content sites |
| client-only | browser | app JS | client only | no | optional | offline/SPAs |
| server-live | server HTML diffs | channel runtime | yes | yes | no/limited | real-time/server-rich |

## Escape hatches and honesty

Interop can weaken compiler guarantees. The product should be honest and inspectable.

When using opaque foreign code, explain output should say:

```txt
This subtree is opaque to EdgeZero.
Unavailable guarantees:
  - binding-level update tracing
  - compiler accessibility proof
  - resumability proof
  - dead CSS pruning inside subtree
```

This honesty is better than pretending interop has no cost.
