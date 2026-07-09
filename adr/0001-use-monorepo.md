# ADR 0001: Use a monorepo

## Status

Accepted

## Context

The compiler, runtime, CLI, fixtures, schemas, examples, and documentation are tightly coupled during early development. Splitting by language would make atomic compiler/runtime changes harder.

## Decision

Use one monorepo with Rust crates under `crates/`, TypeScript packages under `packages/`, versioned schemas under `schemas/`, and behavior fixtures under `fixtures/`.

## Consequences

- Cross-language changes can be reviewed atomically.
- CI must be path-aware as the repository grows.
- Release lanes can split later when contracts stabilize.
