# MVP Roadmap

## MVP definition

The MVP must prove the compiler-centered thesis. A counter demo is insufficient.

A credible MVP demonstrates:

1. TSX class components.
2. Compiler-built template graph.
3. Fine-grained state-to-binding updates.
4. SSR HTML output.
5. Lazy event handler loading.
6. Native form fallback plus enhanced server action.
7. Basic resource primitive.
8. Accessibility diagnostics for forms/buttons/images/clickable divs.
9. `edgezero explain` output.
10. Web Component output for simple components.

## Phase 0: Design and spike

Duration: internal planning phase.

Deliverables:

- IR design draft.
- Parser strategy decision.
- Decorator/class-field transform spike.
- TSX template graph proof.
- Signal runtime prototype.
- DOM patch benchmark harness.
- SSR string/template emission prototype.

Exit criteria:

- Compile simple class component to HTML + patch code.
- State change updates exact text binding.
- Explain output lists state, binding, event.

## Phase 1: Core compiler/runtime prototype

Deliverables:

- Component parser/normalizer.
- Template graph.
- Reactive graph.
- Signal engine.
- DOM patcher.
- Event delegation.
- Lazy import manifest.
- `edgezero dev` minimal.
- `edgezero explain` minimal.

Example supported:

```tsx
@component("x-counter")
class Counter extends Component {
  @state count = 0;
  increment() { this.count++; }
  render() { return <button onClick={this.increment}>Count: {this.count}</button>; }
}
```

Exit criteria:

- SSR renders HTML.
- Initial JS is loader only.
- Click lazy-loads handler.
- Binding updates without component rerender.
- Explain output is accurate.

## Phase 2: Forms and actions MVP

Deliverables:

- Native form action compilation.
- Server action adapter.
- Enhanced submit runtime.
- Pending/error state.
- Field-level errors.
- Basic schema adapter.
- Form accessibility checks.

Example supported:

```tsx
<form action={this.save}>
  <label>Email <input name="email" type="email" required /></label>
  <button disabled={this.save.pending}>Save</button>
</form>
```

Exit criteria:

- Works without JS via native POST.
- Works with JS via enhanced lazy action.
- Errors patch into page.
- Accessibility diagnostics catch missing labels.
- Explain output shows fallback/enhancement.

## Phase 3: Resources and streaming MVP

Deliverables:

- `resource()` primitive.
- Server resource execution.
- Resource serialization.
- Basic invalidation from actions.
- `<Await>` primitive.
- Streaming HTML for async regions.

Exit criteria:

- Route renders async resource on server.
- Streaming fallback flushes before resource resolves.
- Action invalidates resource.
- Explain output shows resource graph.

## Phase 4: Web Component library target

Deliverables:

- Custom-element emitter.
- Attribute/property mapping.
- Slots and parts support.
- Type declarations.
- Component manifest.
- Lazy upgrade option.

Exit criteria:

- A component authored in EdgeZero exports as a standards-native custom element.
- Consumed in a plain HTML page with minimal runtime.
- Manifest documents props/events/slots/parts.

## Phase 5: Semantic inspector alpha

Deliverables:

- Dev metadata format.
- Browser extension or overlay prototype.
- DOM node to source mapping.
- Binding dependency view.
- Last update cause trace.
- Chunk and ownership display.
- Accessibility panel.

Exit criteria:

- Click DOM node and see source expression.
- See why a binding last updated.
- See which chunk an event loaded.

## Phase 6: Production hardening

Deliverables:

- Incremental compiler cache.
- Full source maps.
- CSP support.
- Security review for actions/state serialization.
- Test matrix.
- Adapter hardening.
- Documentation site.
- Migration policy.

Exit criteria:

- Real sample app can deploy.
- CI can run `check`, `a11y`, and `size`.
- Performance budget is tracked.

## MVP sample app

Build a sample app that exercises real constraints:

```txt
/examples/acme-admin
  - authenticated layout mock
  - users route with resource
  - editable user profile form
  - optimistic role update
  - streamed activity feed
  - reusable design-system fields exported as WC
  - accessibility diagnostics intentionally demonstrated
  - explain/size reports checked into docs
```

## Do not build first

Avoid spending early cycles on:

- visual builder,
- large component library,
- every deployment adapter,
- full React compatibility,
- animation system,
- complex router edge cases,
- distributed cache layer,
- AI code generation,
- proprietary CSS system.

These can follow after the compiler thesis is proven.
