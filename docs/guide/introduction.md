# Introduction

Presolve makes the compiler the semantic authority for a web application.
Components, reactive updates, DOM bindings, route discovery, production
artifacts, and deployment inventories are compiler products rather than
conventions reconstructed by a general-purpose browser framework.

That does not change the everyday authoring model: write TypeScript classes
and TSX, put routes in `app/routes`, and run `presolve dev`, `presolve check`,
or `presolve build`.

```text
TypeScript and TSX
       ↓
Presolve compiler
       ├── static HTML
       ├── narrowly scoped browser artifacts
       ├── route inventory
       └── deployment inventory
```

The `@presolve/core` package is deliberately small. Its decorators communicate
meaning to the compiler; they do not register components, create a reactive
store, or install a renderer when JavaScript evaluates them. Compilation is
what gives those declarations their behavior.

## What Presolve is for

The alpha is appropriate for static and interactive sites whose source uses
compiler-admitted TypeScript and TSX. It includes components, state, actions,
computed values, effects, composition, Context, forms, resources, file routes,
production artifacts, and the Cloudflare static deployment path.

Presolve is not yet a general server platform. It does not provide SSR,
streaming, arbitrary server execution, database or authentication abstractions,
or executable server loaders/actions. Read the [support reference](../reference/support.md)
before choosing it for an application.

## Next

Create an application with [the installation guide](installation.md), then
read [components](components.md) and [reactivity](reactivity.md).
