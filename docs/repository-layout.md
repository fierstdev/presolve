# Repository layout

This is the current repository map for contributors. It describes present
ownership only; it does not define future package, service, or release
architecture.

## Active boundaries

| Root path | Ownership |
| --- | --- |
| `.github/` | GitHub workflows and collaboration templates. |
| `adr/` | Accepted architectural-decision records. |
| `crates/` | Active Rust compiler, parser, CLI, and supporting crates. |
| `docs/` | Public documentation, current specifications, frozen contracts, and historical archive. |
| `e2e/` | Active browser-harness documentation. |
| `examples/` | Active canonical examples. |
| `fixtures/` | Frozen compiler, runtime, browser, and golden verification fixtures. |
| `notes/` | The live continuation log and current handoff remain under `notes/progress/`. |
| `packages/` | Active JavaScript and TypeScript packages. |
| `rfcs/` | Accepted or active technical RFCs. |
| `schemas/` | Active and frozen versioned schemas. |
| `scripts/` | Maintained repository automation. |

`LICENSE`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`,
`Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`,
`pnpm-workspace.yaml`, `justfile`, `rust-toolchain.toml`, `.gitattributes`,
`.gitignore`, and `README.md` remain root control files. Optional `tools/`, `benches/`,
`benchmarks/`, and root `tests/` directories are absent until an authorized
slice introduces them.

The repository deliberately has no root `compiler/`, `runtime/`, or `cli/`
directory. Active responsibilities remain in `crates/` and `packages/`.

## Historical archive

`docs/archive/engineering/` preserves non-normative engineering history without
rewriting it. Planning documents live beneath `planning/`; accepted parser
spike evidence lives beneath `spikes/accepted/`; inactive resource notes live
beneath `resources/`. The live progress and handoff records remain under
`notes/progress/` so the established continuation workflow and weekly-log
automation keep their current paths.

## Phase L authority

The authoritative Phase L specifications are tracked in
[`specifications/phase-l/`](specifications/phase-l/README.md). They are active
constitutional documents, not archived planning material.

## Enforcement

Run `./scripts/verify-repository-layout.sh` (or `just repository-layout`) to
validate this map, the archive boundaries, and the Phase L specification index.
