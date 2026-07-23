# Phase M M9 framework freeze

**Status:** M9 freeze authority.

## Frozen framework surface

Phase M freezes the private `@presolve/framework-types` declaration surface,
private artifact-build handoff, compiler-backed DX guidance, and these source
forms:

- explicit `@component("tag")`, `state(initializer)`, `@action()`,
  `@computed()`, and `@effect()`;
- static `@context()` with `@provide("Owner.member")` and
  `@consume("Owner.member")`;
- `@slot()` and compiler Component/Slot JSX forms;
- `@form()`, `@field("form")`, optional static `@field("form", "a.b")`
  serialization paths, validation/serialization, `@submit("form")`,
  and `<form form={this.form}>`.

Decorators remain TypeScript declarations only. Compiler compilation, identity,
diagnostics, artifacts, scheduling, runtime execution, resume, and optimization
remain the sole semantic authority.

## Compatibility and migration

The freeze supports the repository-pinned TypeScript 7.0 native CLI and the
current compiler products proven by M2–M8. TypeScript 7.1 is unsupported until
its pinned toolchain reruns the complete declaration matrix and this contract
is amended. There is no compatibility promise for obsolete framework source:
instance Context declarations, `Owner.instanceField` Context references, and
`@field(this.form)` / `@submit(this.form)` are not framework forms. The
compiler may retain internal compatibility, but the framework neither types nor
translates it.

## Permanently unavailable in Phase M

No router, SSR, server actions, loaders, dev server, HMR, scaffolder, project
discovery, deployment, package publication, CSS system, framework renderer,
hydration layer, reactive runtime, Context lookup, Form controller, artifact
decoder, parser, transform, source scanner, or alternate compiler path exists.

## Evidence and metaframework handoff

`scripts/verify-m9-framework-freeze.sh` executes the Phase M proposal audit
and every focused M2–M8 verifier. It is the frozen evidence matrix. A future
metaframework must supply separate authority for routing, data loading, server
rendering, orchestration, installation, and deployment; it may not reinterpret
this contract or create a compatibility shim around compiler artifacts.
