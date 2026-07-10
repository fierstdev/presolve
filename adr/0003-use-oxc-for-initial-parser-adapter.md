# ADR 0003: Use Oxc for the initial parser adapter

## Status

Accepted for initial implementation.

This decision is reversible before public alpha.

## Context

EdgeZero needs a parser backend for TSX authoring. The parser must support class-based components, decorators, JSX, source spans, and syntax diagnostics.

The parser backend must not become the compiler model. EdgeZero should own its semantic model and use parser-specific ASTs only inside adapter code.

A parser spike evaluated Oxc and SWC against the current valid and broken fixtures.

Fixtures:

- `fixtures/0001-source-summary/input/Counter.tsx`
- `fixtures/0002-broken-tsx/input/BrokenCounter.tsx`

## Decision

Use Oxc for the first real EdgeZero parser adapter.

Keep SWC documented as a fallback candidate.

Defer Tree-sitter and Biome evaluation until editor recovery, incremental parsing, lossless formatting, or live-inspection requirements become concrete.

## Evidence

Oxc successfully parsed the valid TSX fixture with zero errors.

The Oxc spike extracted a normalized `ParserProbe` containing:

- class declaration: `Counter`
- decorators:
  - `@route("/counter")`
  - `@component("x-counter")`
- class property:
  - `count = state(...)`
- methods:
  - `increment`
  - `render`
- JSX root:
  - `<button>`
- JSX attribute:
  - `onClick={...}`
- JSX binding:
  - `this.count`
- byte-offset spans for relevant nodes

Oxc also returned structured diagnostics for the malformed TSX fixture:

- message: `Unexpected token`
- severity: `Error`
- label offset: `198`
- label length: `1`
- mapped location: `9:16`


SWC also successfully parsed the valid TSX fixture and reported a useful error for the malformed fixture:

- error: `Unexpected { got: "{", expected: "jsx identifier" }`
- span: `199..200`

SWC remains viable, but the spike currently has stronger EdgeZero integration evidence for Oxc because Oxc has already been mapped into normalized EdgeZero-style facts.

## Consequences

The first real parser adapter should use Oxc.

Oxc AST types must not leak into:

- component graph code
- template graph code
- reactive graph code
- public compiler APIs
- JSON explain output
- diagnostics output formats

The adapter should map Oxc AST structures into EdgeZero-owned data structures.

SWC remains a fallback if Oxc proves unsuitable.

Tree-sitter and Biome remain candidates for future editor/lossless/recovery use cases.

## Rules

- Parser-specific code must live behind an adapter boundary.
- EdgeZero semantic models must not expose Oxc types.
- Fixtures must prove parser behavior.
- Parser diagnostics must be converted into EdgeZero diagnostics.
- Parser spans must be converted into EdgeZero span/location types.
- This decision should be revisited before public alpha.

## Follow-up work

- Create a real parser adapter crate or module.
- Define EdgeZero-owned parsed component structures.
- Move useful spike logic out of `ezc_parser_spike`.
- Add parser fixtures for valid and invalid TSX.
- Add diagnostics tests.
- Decide whether the spike crate should remain in the workspace or be removed after adapter implementation.
