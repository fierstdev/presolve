# Parser Backend Evaluation Spike

## Purpose

Evaluate parser backends for EdgeZero TSX source analysis.

This spike does not select a parser by opinion. It compares candidates against EdgeZero’s compiler needs.

## Candidates

- Oxc
- SWC
- Tree-sitter TypeScript/TSX
- Biome parser

## Required capabilities

A parser candidate must be evaluated against this fixture:

`fixtures/0001-source-summary/input/Counter.tsx`

The spike should determine whether the parser can expose:

- route decorator
- component decorator
- class declaration
- render method
- JSX root node
- JSX element names
- expression spans
- syntax diagnostics
- stable byte spans
- useful error recovery behavior

## Evaluation matrix

| Criterion | Oxc | SWC | Tree-sitter | Biome |
|---|---:|---:|---:|---:|
| Parses TSX | Unknown | Unknown | Unknown | Unknown |
| Decorator support | Unknown | Unknown | Unknown | Unknown |
| JSX node access | Unknown | Unknown | Unknown | Unknown |
| Span quality | Unknown | Unknown | Unknown | Unknown |
| Error recovery | Unknown | Unknown | Unknown | Unknown |
| API simplicity | Unknown | Unknown | Unknown | Unknown |
| Dependency weight | Unknown | Unknown | Unknown | Unknown |
| Maintainer confidence | Unknown | Unknown | Unknown | Unknown |
| Fit for compiler pipeline | Unknown | Unknown | Unknown | Unknown |
| Fit for LSP/editor tooling | Unknown | Unknown | Unknown | Unknown |

## Rules

- Do not replace the existing source-summary implementation during this spike.
- Do not remove existing fixtures.
- Do not make a permanent parser decision without an ADR.
- Prefer small isolated spike crates or examples over invasive changes.
- Record failed attempts.
- Record confusing APIs.
- Record compile errors and fixes.
- Record whether spans are byte offsets, line/column positions, or both.

## Expected output

At the end of the spike, produce:

- a short parser comparison
- one recommended parser for the next implementation slice
- one fallback parser
- risks
- follow-up ADR draft


## Oxc first run

Command:

```sh
cargo run -p ezc_parser_spike -- fixtures/0001-source-summary/input/Counter.tsx
```

Result:

- Compiled: yes
- Parsed fixture: yes
- Errors: 0

Notes:

- Oxc dependency tree compiled successfully in the workspace.
- The parser accepted the current TSX fixture with TypeScript and JSX enabled.
- This only proves parse acceptance. It does not yet prove that decorators, class declarations, render methods, JSX nodes, or spans are accessible in the shape EdgeZero needs.
