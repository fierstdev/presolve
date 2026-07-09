# RFC 0001: Parser backend selection

## Status

Draft

## Question

Which parser should EdgeZero use for TSX/html-template source ingestion?

## Options to evaluate

1. Oxc parser
2. SWC parser
3. Tree-sitter TSX grammar
4. Biome parser infrastructure
5. TypeScript compiler API through a Node-side bridge

## Evaluation criteria

- TSX support
- span fidelity
- error recovery
- AST stability
- typed AST ergonomics in Rust
- transform/codegen compatibility
- source map compatibility
- maintenance activity
- license compatibility
- suitability for LSP/incremental parsing

## Required spike output

Each option must parse the same `Counter.tsx` fixture and report:

- class declarations
- decorators
- render method
- JSX element tree
- expression spans
- parser diagnostics

## Non-goal

Do not choose a backend based only on benchmark claims. Developer diagnostics and semantic graph construction matter more than raw parse speed for the first version.
