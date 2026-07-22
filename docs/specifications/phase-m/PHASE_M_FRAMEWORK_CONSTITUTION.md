# Phase M framework constitution

**Status:** M0 owner-accepted authority.

## Purpose

Phase M establishes a private, conformance-first Presolve Framework. It makes
the frozen compiler's authoring language usable from TypeScript projects; it
does not redesign that language or add a framework execution model.

The compiler remains the only authority for source interpretation, semantic
identity, dependency topology, storage, scheduling, diagnostics, generated
artifacts, browser execution, resumability, and optimization. The framework
may supply ambient TypeScript declarations, explicit configuration helpers,
opaque canonical-command invocation, documentation, examples, and diagnostic
presentation. It must never translate source, reimplement compiler analysis,
decode or rewrite compiler products, or change compiler behavior.

## Product boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| Presolve Compiler | Frozen authoring grammar, semantic facts, diagnostics, products, runtime plans, artifacts, and schemas | Framework package resolution, guides, or compatibility presentation |
| Presolve Framework | TypeScript declaration delivery, explicit project handoff, opaque command status, examples, and a framework compatibility table | A parser, source transform, semantic analyzer, state store, renderer, Context lookup, product decoder, artifact writer, or runtime scheduler |
| Future Presolve Metaframework | Routing, loading, server rendering, dev/build orchestration, deployment, installation, project discovery, and `presolve create` | Any Phase M responsibility before a separately accepted roadmap |

`create`, `dev`, `benchmark`, and `doctor` remain reserved exit-6 command
families. Phase M does not change their disposition.

## Non-negotiable conformance rules

1. A framework-authored source file is passed unchanged to the canonical
   compiler path. The framework cannot substitute, normalize, generate, or
   lower a different source representation.
2. Every supported framework form must have one cited frozen compiler form and
   one conformance fixture. A form without both is unavailable.
3. The framework reports compiler diagnostics with their original code,
   severity, spans, labels, and canonical identities intact. Optional guidance
   is separate and cannot replace, suppress, reorder, or manufacture a
   diagnostic.
4. The framework invokes only caller-supplied explicit project configuration
   and source membership through the accepted `presolve` command boundary. It
   performs no source, project, package, or artifact discovery.
5. Compiler bytes and schema meanings are immutable compatibility inputs. A
   framework version either declares support for an exact compiler/product
   tuple or fails closed before interpreting the result.
6. TypeScript declarations exist for authoring ergonomics only. They do not
   make a field reactive, register a component, perform Context lookup, or
   establish any runtime authority.

## Initial package decision

M2 creates one private declaration-only package at
`framework/packages/framework-types` with package identity
`@presolve/framework-types`. It is selected through a project `tsconfig`
`types` entry rather than importing or aliasing compiler language primitives in
application source. This keeps the compiler-visible forms lexical and
unchanged:

```json
{
  "compilerOptions": {
    "types": ["@presolve/framework-types"]
  }
}
```

The package may provide ambient declarations for existing compiler built-ins
such as `Component`, `SlotContent`, `Form`, `state`, and the existing decorator
names. It emits no JavaScript and has no runtime registration or transform.
This decision is required because frozen `Form` and `SlotContent` forms are
compiler built-ins: imported, aliased, or locally redeclared `Form` is not the
frozen Form authority.

The initial framework package is private and does not make an npm publication,
registry-installation, or package-manager promise. The short package name
`presolve` is not reserved or exported by Phase M.

## TypeScript toolchain policy

Framework declaration conformance uses the repository-pinned TypeScript 7.0
native command-line compiler only. Phase M invokes `tsc` as a CLI and never
uses a TypeScript compiler API, parser, transform, or language-service API.
This matches TypeScript 7.0's command-line boundary while its new programmatic
API is deferred to 7.1.

TypeScript 7.1 is a future compatibility input, not an implicit upgrade. The
framework compatibility matrix may add a 7.1 row only after the pinned compiler
is installed, every declaration fixture is rerun, and any changed diagnostic or
configuration behavior is recorded without changing compiler-facing source.

## Compatibility and evidence

The framework compatibility matrix records, for every supported release:

- framework declaration package version;
- accepted `presolve` CLI grammar and explicit project-envelope version;
- compiler and runtime artifact schema versions consumed only opaquely;
- exact supported authoring-form rows; and
- fixture and browser evidence.

The matrix records TypeScript 7.0 and later TypeScript 7.1 support separately.

An unsupported compiler version, missing required artifact, unexpected command
result, or unsupported form is a fail-closed framework result. No fallback
parser, decoder, runtime, or source conversion is permitted.

## M0 completion and next boundary

M0 accepts the conformance-first direction and changes no frozen compiler or
platform contract. M1 is the authoritative authoring conformance map. M2 may
create only the isolated declaration package and its focused type-resolution
fixtures.
