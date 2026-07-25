# Architecture

Presolve has one semantic authority: the compiler. It reads TypeScript and TSX
source, validates the language forms it supports, and publishes the HTML,
browser artifacts, route inventory, resumability records, and deployment plan.

The `@presolve/core` package supplies types and compiler intrinsics. Its decorators
do not create stores, renderers, dependency trackers, or registries at runtime.
The CLI provides the application conventions: file routes, layouts, development
workflow, production builds, and deployment preparation.

```text
application source
        ↓
Presolve compiler
        ├── static HTML and browser artifacts
        ├── file-route inventory
        ├── resumability records
        └── deployment inventory
```

This boundary is practical rather than ceremonial: a route, state update, or
deployment artifact has one source of truth. Runtime code and editor tooling
consume compiler products; they do not reverse-engineer application semantics
from generated JavaScript.
