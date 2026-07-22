# Phase M conformance authoring contract

**Status:** M1 owner-accepted authority.

## Rule of interpretation

This document exposes only forms already accepted by the frozen compiler. The
examples are not proposed alternatives, aliases, or syntax sugar. The
framework must pass them to the compiler unchanged. If a desired spelling is
absent below, it is unsupported in Phase M.

Compiler diagnostics, not TypeScript declarations, decide semantic validity.
Declaration types may teach ordinary TypeScript shapes, but they do not infer
compiler-only facts such as serializability, capability validity, Context
selection, action batching, or compiler identity.

## Frozen authoring vocabulary

| Family | Exact framework source form | Frozen authority and Phase M boundary |
| --- | --- | --- |
| Component | `@component("x-name") class Name extends Component { render() { return <div />; } }` | The string remains the compiler-recognized component declaration argument. `@component()`, component-option objects, generated tag names, and framework component registration are unavailable. A source may use only compiler-accepted export and heritage forms; inheritance beyond the compiler's allowed `Component` base remains unsupported. |
| State | `count = state(0)` | `state(initializer)` is the frozen State declaration. `@state() count = 0`, signals, `.value`, proxies, setters, and framework state storage are unavailable. Compiler action rules decide every reactive write. |
| Action | `@action() increment() { this.count += 1; }` | The compiler owns action validation, transactional batching, direct event references, and compiler-supported closures. The framework defines no separate nested-action or async policy and no event wrapper. |
| Computed | `@computed() get doubled() { return this.count * 2; }` | Getter-only compiler semantics, dependency topology, caching, and invalidation remain unchanged. Authored dependency arrays and manual invalidation are unavailable. |
| Effect | `@effect() syncTitle() { document.title = this.title; }` | Effects remain compiler-planned terminal capability programs. Cleanup returns, state mutation, action/effect calls, and unrecognized capabilities remain compiler-rejected or unsupported; the framework adds no hook API. |
| Context declaration | `@context() theme: string = "light";` | A Context is declared on a compiler-recognized component field. `context<T>()`, global Context handles, and string keys are unavailable. |
| Context provider | `@provide(Theme.theme) providedTheme: string = this.selectedTheme;` | The designator stays the existing static member form. Provider getters/methods, runtime lookup, and framework-owned Context values are unavailable. |
| Context consumer | `@consume(Theme.theme) theme!: string;` | Compiler-owned binding and instance-qualified selection remain authoritative. Consumer initializers and fallbacks are unavailable. |
| Component use | `<Card />` | PascalCase JSX invocation, compiler component resolution, and compiler-owned instance identity remain unchanged. Props, spreads, callbacks as component arguments, and dynamic component targets are unavailable. |
| Slots | `@slot() children!: SlotContent;` and `<Card><template slot="header"><h1 /></template></Card>` | Default/named Slot declarations, direct-child template wrappers, and outlets use the frozen component contract. Nested JSX slot sugar, forwarding, fallback, runtime name matching, and dynamic names are unavailable. |
| Forms | `@form() profile!: Form;`, `@field(this.profile) name = "";`, `@action() @submit(this.profile) save() {}`, and `<form form={this.profile}>` | Forms retain their exact compiler declaration, field, submission, and host forms. Framework aliases, generated hosts, async/network submission, and DOM-derived ownership are unavailable. |

The Counter conformance source is therefore:

```tsx
@component("x-counter")
class Counter extends Component {
  count = state(0);

  @action()
  increment() {
    this.count += 1;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
```

## TypeScript declaration policy

`@presolve/framework-types` supplies only ambient names required for ordinary
TypeScript parsing and checking of frozen forms. Its `state<T>(value: T): T`
declaration describes the source initializer shape; it does not implement
State. Decorator declarations are compile-time declarations with no emitted
registration, wrapper, or metadata behavior. `Component`, `SlotContent`, and
`Form` remain ambient compiler-language names rather than imported application
symbols.

The frozen Context designator form, for example `Theme.theme`, is a
compiler-level static designator over an instance declaration. TypeScript
cannot faithfully derive that relation from an inert property decorator alone.
Phase M therefore must not add a global index-signature workaround, proxy, or
source rewrite merely to silence TypeScript. Until a source-preserving,
compiler-conformant declaration strategy is proven, Context conformance is
verified by the canonical compiler check and its diagnostic fixtures.

The compiler consumes preserved TSX. The declaration package must not install a
JSX factory, `jsx-runtime`, DOM renderer, or TypeScript decorator transform.

## Required conformance evidence

Each family is added only after all applicable proof below passes:

1. the source uses exactly the documented frozen form;
2. ambient types resolve only where the declaration policy promises they do;
3. the explicit canonical `presolve check` or `build` path accepts the source
   or emits the existing canonical diagnostics unchanged;
4. expected compiler products are compared without framework decoding or
   rewriting; and
5. where execution exists, the frozen browser/runtime fixture proves the
   expected behavior with no hydration or framework reactive runtime.

## Explicitly deferred authoring requests

Phase M does not add undecorated input fields, `@state()`, optional component
identities, external Context factories, object-returning provider getters,
React-style children, cleanup effects, async actions, framework lifecycle
hooks, routers, server rendering, loaders, dev servers, project discovery,
scaffolding, deployment, or package installation. These are unavailable rather
than framework shims over the compiler.

## M1 completion and next boundary

M1 resolves Phase M's source-language question without changing frozen compiler
syntax or semantics. M2 may implement only the private ambient declaration
package and prove the Counter type-resolution path; it may not add a source
transform, runtime package, compiler adapter, or new authoring form.
