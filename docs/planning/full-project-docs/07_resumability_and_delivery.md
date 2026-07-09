# Resumability and Delivery

## Definition

In EdgeZero, resumability means:

> The server can render meaningful HTML and serialize enough semantic state for the browser to continue specific interactions without replaying the whole component tree or downloading all application logic upfront.

Resumability is not a user-facing syntax ceremony. It is a compiler and runtime capability.

## Default delivery policy

1. Render HTML on the server where possible.
2. Include no component JavaScript for static regions.
3. Include a tiny event/resume loader for interactive regions.
4. Load interaction code only when the user interacts or prefetching is justified.
5. Patch exact bindings after state changes.
6. Preserve native form/link fallback.

## What is serialized

Serialize only what is needed:

- state slots required for resumable interaction,
- resource snapshots safe for the client,
- binding/component IDs,
- action/event references,
- route params where needed,
- form pending/error state where needed.

Do not serialize:

- database handles,
- secrets,
- server-only environment values,
- arbitrary closures,
- full component instances unless target requires it,
- data not consumed by client-resumable interactions.

## HTML as continuation format

HTML should carry enough markers to resume interaction, but those markers must be minimal and inspectable.

Example:

```html
<form data-ez-c="checkout-form" data-ez-action="a0" action="/actions/checkout">
  <button data-ez-bind="b4">Pay</button>
</form>
<script type="application/ez-state" nonce="...">
  {"c0":{"submit.pending":false}}
</script>
```

The exact marker syntax should be optimized later. Requirements:

- small,
- CSP-compatible,
- stream-friendly,
- stable enough for DevTools,
- not required in no-JS static-only output.

## Event resumability

Compiled event flow:

```txt
1. User clicks element.
2. Runtime finds event marker.
3. Runtime resolves handler chunk.
4. Runtime loads chunk.
5. Runtime resumes required state slots.
6. Handler runs.
7. Signal graph invalidates exact bindings.
8. DOM patcher updates nodes.
```

The author should not write resumability markers manually.

## Server actions

Server actions provide a controlled mutation boundary.

Requirements:

- native form POST fallback,
- enhanced fetch/WebSocket submission where configured,
- CSRF protection integration,
- validation integration,
- redirect handling,
- streamed errors where possible,
- invalidation of resource graph,
- optimistic update hooks,
- no accidental client bundling of server-only imports.

## Progressive enhancement

Every form and link should start as real HTML.

```tsx
<form action={this.save} method="post">
  <input name="email" type="email" required />
  <button>Save</button>
</form>
```

Compilation modes:

- no JS: native form POST,
- basic JS: enhanced fetch submit,
- resumable: lazy action handler and pending/error patches,
- streaming: validation/result fragments stream back into regions,
- live: server-driven updates where target supports it.

## Interactivity boundary inference

The compiler should infer interactive regions from:

- event handlers,
- client-only APIs,
- mutable client state,
- form enhancement,
- browser resources,
- custom-element export requirements.

Manual boundary annotations should exist only for override cases:

```tsx
<Chart clientOnly />
<ExpensivePanel eager />
<StaticMarketingBlock noClient />
```

## Chunking strategy

Chunk by user-visible interaction where possible.

Examples:

```txt
Initial /checkout
  loader.js
  checkout.css
  HTML document

On click "Apply coupon"
  coupon.apply.js

On submit checkout form
  checkout.submit.js
  payment.validation.js
```

The `edgezero size --by-interaction` command should make this visible.

## Streaming

Streaming is a first-class target behavior.

Authoring:

```tsx
<Await resource={this.recommendations} fallback={<Skeleton />}>
  {items => <Recommendations items={items} />}
</Await>
```

Compiler inference:

```txt
region recommendations
  can flush placeholder immediately
  resource can stream
  error boundary: nearest ErrorBoundary
  client reorder: not required
```

## Failure behavior

A serious delivery model must define failure modes.

### JavaScript disabled

- HTML remains usable where possible.
- Forms post natively.
- Links navigate natively.
- Nonessential client widgets degrade visibly.

### Chunk load failure

- Retry according to policy.
- Surface error to nearest boundary.
- Preserve native fallback where possible.
- Log diagnostic in development.

### Serialization failure

- Compilation fails when statically provable.
- Runtime fails closed when dynamic and unsafe.
- Diagnostic names captured value and boundary.

### Network failure during action

- Pending state resolves to error.
- Optimistic state rolls back if configured.
- Form data is preserved where possible.

## Security constraints

Resumability must not become a serialization vulnerability.

Rules:

1. Server-only values are never serialized.
2. Action IDs are not authorization.
3. CSRF protections are integrated by target adapter.
4. Serialized state is escaped and CSP-compatible.
5. Resource snapshots require explicit public exposure.
6. Dev-only metadata is stripped in production unless opted in.

## Target-specific behavior

### Static target

- Generate static HTML and CSS.
- No server actions.
- Optional client interactions.

### SSR target

- Generate HTML per request.
- Serialize state where needed.
- Actions use server adapter.

### Streaming SSR target

- Support async regions.
- Flush early.
- Preserve fallback/error semantics.

### Resumable web target

- Include resume loader.
- Event handlers lazy by default.
- No full hydration baseline.

### Web Component library target

- Generate custom elements.
- Define component API manifest.
- Runtime includes custom-element upgrader where needed.
