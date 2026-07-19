# Risk Register

## Risk 1: Scope explosion

### Description

The thesis touches compiler design, runtime, SSR, streaming, resumability, accessibility, forms, resources, tooling, DevTools, Web Components, and deployment adapters. This is too broad for an initial build if all are attempted at once.

### Mitigation

Use a narrow MVP:

1. class TSX components,
2. state-to-binding updates,
3. SSR,
4. lazy event handler,
5. form/action primitive,
6. a11y diagnostics,
7. explain output.

Defer full ecosystem breadth.

## Risk 2: Resumability complexity leaks into authoring

### Description

If users must think about serialization constantly, the system becomes Qwik-like in ceremony without Qwik’s maturity.

### Mitigation

Make resumability an inferred default. Require annotations only when crossing ambiguous boundaries. Diagnostics must explain captured values and fixes.

## Risk 3: Class-based authoring feels retro or conflicts with JS semantics

### Description

Decorators and class fields have nuanced semantics. Some developers associate classes with older frameworks.

### Mitigation

Frame classes as semantic authoring units, not inheritance-heavy OOP. Keep composition through components/functions/resources. Consider function component support later, but do not compromise compiler clarity early.

## Risk 4: Web Component output underdelivers

### Description

Web Components solve distribution but not automatically state, forms, styling, SSR, or ergonomics. Shadow DOM and styling can create adoption friction.

### Mitigation

Treat WC as one target. Support light DOM and shadow DOM modes. Generate manifests, slots, parts, and typed props. Do not force app-mode rendering through custom elements.

## Risk 5: Accessibility compiler overpromises

### Description

No compiler can fully guarantee accessible UX.

### Mitigation

Use precise language: compiler-enforced checks and semantic diagnostics. Clearly mark opaque regions and dynamic cases. Provide severity levels and rule configuration.

## Risk 6: Performance claims become benchmark theater

### Description

Frameworks often over-index on contrived benchmarks.

### Mitigation

Show interaction-level size and behavior on realistic apps. Make `edgezero size --by-interaction` central. Track no-JS fallback, slow-JS behavior, and streaming behavior, not only microbenchmarks.

## Risk 7: Compiler magic becomes hard to debug

### Description

The more the compiler infers, the more developers fear invisible behavior.

### Mitigation

Every optimization must have an explanation path. Ship explain/why/trace early, not after v1.

## Risk 8: Server/client splitting creates security bugs

### Description

Accidental serialization of secrets or bundling of server-only code would be severe.

### Mitigation

Fail closed. Maintain a server-only module registry. Require explicit public serialization. Add security tests for action invocation, state payloads, and error leakage.

## Risk 9: Ecosystem adoption barrier

### Description

A new framework must compete with large ecosystems.

### Mitigation

Interop first:

- consume Web Components,
- export Web Components,
- allow React islands,
- use normal npm packages,
- support existing APIs,
- avoid proprietary language lock-in.

## Risk 10: Name conflict

### Description

EdgeZero has existing external usage. Domain ownership does not clear trademark risk.

### Mitigation

Run trademark/package/legal diligence before public launch. Keep Blokd as fallback or sub-brand.

## Risk 11: Rust compiler slows contributor velocity

### Description

Rust can improve compiler performance, but may reduce contributor pool and slow early iteration.

### Mitigation

Prototype IR and semantics quickly. Use Rust where performance matters after architecture stabilizes, or use a hybrid model. Sell outcomes, not Rust.

## Risk 12: Too many output targets weaken quality

### Description

Supporting static, SSR, streaming, resumable, WC, islands, live server, and client-only too early can dilute quality.

### Mitigation

Prioritize:

1. SSR/resumable-web,
2. static subset,
3. WC-library subset,
4. streaming,
5. other targets later.

## Risk 13: Forms become a framework within a framework

### Description

Form systems can become large and opinionated.

### Mitigation

Keep native form semantics at the center. Provide structured form API as an enhancement. Support schema adapters, not one proprietary schema format.

## Risk 14: Fine-grained reactivity with class fields has edge cases

### Description

Deep mutation, destructuring, aliasing, async closures, and getter purity can break clear dependency tracking.

### Mitigation

Define strict rules early. Prefer explicit state wrappers for deep mutable objects. Warn on destructuring that loses reactivity if applicable. Keep dependency tracing visible.

## Risk 15: DevTools is expensive

### Description

A semantic inspector can consume significant engineering time.

### Mitigation

Build explain metadata first. CLI explain/trace should use the same data. Browser DevTools can be layered later.
