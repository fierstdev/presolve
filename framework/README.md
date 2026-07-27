# Presolve framework

The public framework package is [`presolve`](packages/presolve/). It exposes a
small TypeScript authoring vocabulary over compiler-owned semantics and has no
application renderer, scheduler, dependency tracker, parser, or reactive
runtime of its own.

The compiler decides component and instance identity, state storage,
initialization order, dependency topology, DOM operations, effect scheduling,
Context resolution, forms, serialization, resumability, and code generation.
The framework declaration package makes those admitted forms pleasant to write
and useful to TypeScript; it does not recreate them in JavaScript.

Internal compatibility packages and conformance fixtures remain in this
workspace to keep the public package small. They are not application imports.
