# Fixture 0001: Source Summary

## Purpose

This fixture proves the first observable compiler behavior:

```txt
Counter.tsx -> source summary -> explain output
```

This is not yet TSX parsing. It proves source ingestion, basic source spans, declaration discovery, diagnostics, text explain output, and JSON explain output.

## Exit criteria

- `presolve explain input/Counter.tsx` prints a stable text explanation.
- `presolve explain input/Counter.tsx --format json` prints schema-shaped JSON.
- The output includes component decorator, route decorator, class declaration, render method, and diagnostics.
- The summary code has unit tests.

## Why this exists before a real parser

A compiler project must preserve source locations and explain its own inferences from the first week. Starting here prevents the project from becoming a black box later.
