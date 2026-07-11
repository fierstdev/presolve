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
the effective policy. Each parser diagnostic includes `labels` with source
`line`, `column`, `start`, and `end` coordinates; text output prints the same
file-qualified label locations below the diagnostic. Compiler diagnostics with
reliable source locations include optional JSON `provenance` using `path`,
`line`, `column`, `start`, and `end`; diagnostics without a reliable source
location omit that field. Check policy is currently selected per command; no
project configuration file is interpreted yet.

## ASM entity inspection

Inspect one canonical semantic entity with its ownership, provenance,
containment, relations, and overlap-based compiler diagnostics:

```sh
ezc_cli asm <file> --entity <semantic-id> [--format text|json]
ezc_cli asm <file> --source <path> --offset <byte> [--format text|json]
```

Use `ezc_cli asm <file> --format json` to discover the available semantic IDs.
The selected-entity document includes the entity itself, direct child IDs,
descendant count, incoming and outgoing references, and related compiler
diagnostics. An unknown semantic ID fails explicitly.

Source selection chooses the uniquely narrowest semantic span covering the
given byte offset. No match or tied narrowest spans fail explicitly; `--entity`
cannot be combined with `--source` or `--offset`.

## Repository rules

- No major feature without an issue.
- No durable architecture decision without an ADR.
- No user-facing syntax without an RFC.
- No compiler behavior without a fixture.
- No compiler inference without explain output.
- No area switch until the current slice has exit criteria recorded.

## Current slice

See [`fixtures/0001-source-summary/README.md`](fixtures/0001-source-summary/README.md).
