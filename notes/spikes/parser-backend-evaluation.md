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


## Oxc AST inspection

Oxc exposed the facts needed from the current counter fixture:

- class declaration: `Counter`
- decorators:
  - `route("/counter")`
  - `component("x-counter")`
- class property:
  - `count = state(0)`
- methods:
  - `increment`
  - `render`
- render return shape:
  - `ParenthesizedExpression`
  - nested `JSXElement`
- JSX element:
  - opening name `button`
  - attribute `onClick`
  - text child
  - expression child `{this.count}`
- spans:
  - byte-offset spans are present on relevant nodes

Notes:

- Oxc’s AST shape uses helpers such as `statement.as_declaration()`.
- JSX returned from a parenthesized `return (...)` appears under `ParenthesizedExpression`, so EdgeZero extraction must unwrap expression wrappers before detecting template roots.
- Decorators are represented as call expressions with identifier callees and literal arguments, which is suitable for extracting `@route(...)` and `@component(...)`.


## Oxc normalized probe

Created a spike-only `ParserProbe` that maps Oxc AST details into EdgeZero-style facts.

Command:

```sh
cargo run -p ezc_parser_spike -- fixtures/0001-source-summary/input/Counter.tsx
```

Result:

- Compiled: yes
- Parsed fixture: yes
- Errors: 0

Extracted facts:

- class declaration:
  - `Counter`
- decorators:
  - `@route("/counter")`
  - `@component("x-counter")`
- class property:
  - `count = state(...)`
- methods:
  - `increment`
  - `render`
- render JSX:
  - root element: `<button>`
  - attribute: `onClick={...}`
  - binding: `this.count`
- spans:
  - byte-offset spans are available for class, decorators, properties, methods, and JSX root

Oxc API notes:

- `Statement` uses Oxc’s inherited-variant/accessor pattern. Use `statement.as_declaration()`.
- `JSXExpression` also uses this accessor pattern. Use `jsx_expression.as_expression()`.
- JSX returned from `return (...)` appears under `ParenthesizedExpression`, so extraction must unwrap expression wrappers.
- Decorators are represented as call expressions with identifier callees and literal arguments.

Current limitations:

- only handles simple class declarations
- only handles simple decorator call expressions
- only summarizes a few expression types
- only detects JSX roots returned directly or through parentheses
- does not yet distinguish event handlers from reactive bindings
- does not recurse into nested JSX elements as separate roots
