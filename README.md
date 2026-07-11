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
cargo run -p ezc_cli -- check fixtures/0001-source-summary/input/Counter.tsx
```

## Check policy

Use `ezc_cli check` to compile one or more source files, assemble the canonical
Application Semantic Model (ASM), and report parser, compiler, and ASM
validation diagnostics.

```sh
ezc_cli check <file> [file...] \
  [--format text|json] \
  [--category parser|compiler|validation] \
  [--fail-on error|warning|info]
```

The project default is `--fail-on error`: parser errors fail the command, while
parser warnings and informational diagnostics remain visible without failing it.
`--fail-on warning` also fails on warnings, and `--fail-on info` fails on every
parser diagnostic. Compiler diagnostics and ASM validation diagnostics always
fail `check`, regardless of the selected parser threshold.

Repeat `--category` to limit diagnostic detail in text or JSON output. Category
filters never change summary counts or the command's exit status. JSON output
includes the selected parser threshold in `fail_on` so automation can record
the effective policy. Check policy is currently selected per command; no
project configuration file is interpreted yet.

## Repository rules

- No major feature without an issue.
- No durable architecture decision without an ADR.
- No user-facing syntax without an RFC.
- No compiler behavior without a fixture.
- No compiler inference without explain output.
- No area switch until the current slice has exit criteria recorded.

## Current slice

See [`fixtures/0001-source-summary/README.md`](fixtures/0001-source-summary/README.md).
