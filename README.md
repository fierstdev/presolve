# EdgeZero Starter Repository

EdgeZero is a compiler-centered web authoring system. This repository starts with the smallest useful vertical slice:

```txt
source file
  -> source summary
  -> spans and basic declarations
  -> diagnostics
  -> explain output
  -> fixture
```

This is not the real TSX compiler yet. It is the first learning and infrastructure slice. The purpose is to establish repository layout, CLI shape, fixture discipline, documentation discipline, and source-location handling before committing to a parser backend.

## First commands

```sh
# after installing Rust and pnpm
cargo test --workspace
cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx
cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json
```

## Repository rules

- No major feature without an issue.
- No durable architecture decision without an ADR.
- No user-facing syntax without an RFC.
- No compiler behavior without a fixture.
- No compiler inference without explain output.
- No area switch until the current slice has exit criteria recorded.

## Current slice

See [`fixtures/0001-source-summary/README.md`](fixtures/0001-source-summary/README.md).
