# ADR 0002: Start with source summary before real TSX parsing

## Status

Accepted

## Context

The project needs parser research, but it also needs immediate repo discipline, CLI shape, fixtures, and explain output.

## Decision

The first compiler slice will summarize source files without claiming to parse TSX. Real parser integration will be decided by RFC after comparing Oxc, SWC, Tree-sitter, and other options.

## Consequences

- The first week produces working infrastructure without blocking on parser choice.
- The source summary code must be deleted or demoted once a real parser exists.
- No semantic compiler claims may be made from this heuristic scanner.
