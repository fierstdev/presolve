# Architecture Overview

## System shape

EdgeZero has three major layers:

```txt
Authoring layer
  TSX
  html`` templates
  class-based components
  decorators/annotations
  state/resources/actions/forms
  route and layout declarations

Compiler layer
  parser and normalizer
  semantic graph builder
  graph analyzers
  optimizer
  target emitters
  diagnostics/explain engine

Runtime layer
  tiny scheduler
  signal engine
  event delegation
  DOM patching
  lazy import resolver
  resumability loader
  optional custom-element upgrader
```

The runtime is intentionally small because it executes decisions made by the compiler. It should not rediscover the application at runtime.

## High-level build flow

```txt
source files
  ↓
parse TS/TSX/html templates
  ↓
normalize to EdgeZero AST
  ↓
build semantic graphs
  ↓
analyze dependencies, ownership, serializability, a11y, style, resources
  ↓
produce compiler IR
  ↓
optimize and split
  ↓
emit target artifacts
  ↓
explain/debug metadata
```

## Core graphs

The compiler should own these graphs:

1. Template graph.
2. Reactive graph.
3. Event graph.
4. Serialization graph.
5. Resource/data graph.
6. Accessibility graph.
7. Style graph.
8. Component graph.
9. Streaming graph.
10. Debug graph.
11. Server/client split graph.
12. Lazy-loading graph.

These graphs are not marketing language. They are internal compiler assets that enable specific outputs and diagnostics.

## Runtime responsibilities

The runtime should do only what cannot be done statically:

- schedule reactive effects,
- patch bound DOM nodes,
- delegate and route events,
- lazily import event/resource chunks,
- resume serialized state,
- upgrade custom elements when requested,
- coordinate async region transitions,
- preserve form pending/error state,
- expose trace hooks in development.

The runtime should not:

- diff a virtual DOM as the default update model,
- re-run whole components for local state changes,
- hydrate the entire app just to attach event handlers,
- require user-managed memoization for common cases,
- infer server/client boundaries after code has shipped.

## Deployment targets

One source should support multiple output modes:

```bash
edgezero build --target static
edgezero build --target ssr
edgezero build --target streaming-ssr
edgezero build --target resumable-web
edgezero build --target wc-library
edgezero build --target island
edgezero build --target server-live
edgezero build --target client-only
```

Not every feature must be available for every target in v1, but the architecture should treat target emission as a core compiler function.

## Package-level architecture

```txt
@edgezero/compiler
  - parser adapters
  - semantic graph builder
  - IR
  - analyzers
  - emitters
  - diagnostics

@edgezero/runtime
  - signals
  - scheduler
  - event delegation
  - DOM operations
  - lazy import resolver
  - resume loader

@edgezero/server
  - SSR
  - streaming
  - server actions
  - resource serialization
  - adapter APIs

@edgezero/devtools
  - semantic inspector protocol
  - trace viewer
  - browser extension bridge

@edgezero/forms
  - optional form primitives
  - schema adapters
  - field/control registry

@edgezero/wc
  - custom-element emit helpers
  - form-associated element helpers
  - shadow/slot/part utilities
```

## Two modes of adoption

### Application mode

Developers use routing, resources, actions, forms, SSR, streaming, and deployment adapters.

### Library mode

Developers author components and export Web Components or framework adapters. Runtime should be extremely small and tree-shaken.

## Runtime budget

Initial target budgets:

| Artifact | Budget |
|---|---:|
| Base resumability/event loader | <= 1.5kb gzip |
| Signal engine + scheduler | <= 2.5kb gzip |
| DOM binding patcher | <= 1.5kb gzip |
| Optional custom-element upgrader | <= 2kb gzip |
| Dev trace hooks | dev-only |

Budgets are directional and should be verified with real implementation constraints. The important policy is that runtime growth must be justified by compiler capability or platform interop, not convenience.

## Compiler implementation language

Rust is a credible implementation choice for compiler performance and parallelism, but it should not be the public selling point. Developers adopt outcomes:

- fast builds,
- instant diagnostics,
- precise source maps,
- reliable incremental compilation,
- small output,
- explainable optimization.

Implementation language is a delivery mechanism, not the product category.
