# Repository layout

This is the current repository map for Presolve contributors. It describes
present ownership only; the public product boundary is described in the
[documentation index](README.md).

## Active boundaries

| Root path | Ownership |
| --- | --- |
| `.github/` | GitHub workflows and collaboration templates. |
| `adr/` | Accepted architectural-decision records. |
| `crates/` | Active Rust compiler, parser, CLI, and supporting crates. |
| `docs/` | Public product documentation plus historical engineering records. |
| `e2e/` | Active browser-harness documentation. |
| `examples/` | Active canonical examples. |
| `fixtures/` | Frozen compiler, runtime, browser, and golden verification fixtures. |
| `framework/` | The `presolve` authoring package and its compiler-conformance fixtures. |
| `metaframework/` | Application-facing metaframework package and integration fixtures. |
| `notes/` | Historical engineering progress records. |
| `packages/` | Active JavaScript and TypeScript packages. |
| `rfcs/` | Accepted or active technical RFCs. |
| `schemas/` | Active and frozen versioned schemas. |
| `scripts/` | Maintained repository automation. |
| `site/` | Historical repository-local launch-content prototype; not the official website source. |

`LICENSE`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`,
`SUPPORT.md`,
`Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`,
`pnpm-workspace.yaml`, `justfile`, `rust-toolchain.toml`, `.gitattributes`,
`.gitignore`, and `README.md` remain root control files. Optional `tools/`, `benches/`,
`benchmarks/`, and root `tests/` directories are absent until an authorized
slice introduces them.

The repository deliberately has no root `compiler/`, `runtime/`, or `cli/`
directory. Active responsibilities remain in `crates/` and `packages/`.

## Historical records

`docs/archive/engineering/` preserves non-normative engineering history without
rewriting it. `docs/specifications/` and `notes/progress/` also retain frozen
pre-public engineering records at their original paths so evidence links and
fixtures remain reproducible. They do not define the public product API.

## Enforcement

Run `./scripts/verify-repository-layout.sh` (or `just repository-layout`) to
validate this map and the historical-record boundaries.
