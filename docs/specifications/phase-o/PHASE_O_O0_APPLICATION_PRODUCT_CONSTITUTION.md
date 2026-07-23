# Phase O O0 application product constitution

**Status:** O0 implementation authority.

## Product boundary

The private `metaframework/` workspace owns an application-product facade over
the public `presolve` command. Its public JavaScript functions may validate
caller-provided request objects and create immutable invocation objects. They
must not read application source, discover files, parse TypeScript/TSX, inspect
package implementations, derive dependency topology, decode compiler artifacts,
or execute generated application code.

## Compiler authority

The compiler remains the only authority for source admission, package-contract
validation, semantic identity, diagnostics, generated HTML/runtime artifacts,
Resource/opaque runtime behavior, resume, and optimization. Phase O supplies
only exact public CLI arguments and preserves executor output unchanged.

## Product names

The first private package is `@presolve/application`. It is a metaframework
orchestration package, not a replacement for `@presolve/framework-types`.
It has no TypeScript decorators, JSX runtime, renderer, state store, router,
or artifact decoder.

## Explicit exclusions

O0 does not authorize project discovery, automatic source collection, package
installation or lockfile resolution, dev server, HMR, routing, SSR, server
actions, loaders, middleware, CSS/assets, deployment, authentication,
environment variables, or a create command.

## Evidence

`scripts/verify-o0-application-productization.sh` proves the O0 documents
reference only existing public CLI command forms and preserve the frozen M10/N10
boundaries. It does not compile an application or mutate compiler products.
