# Changelog

This repository follows Keep a Changelog-style entries. Changes are recorded
under `Unreleased` and grouped by Added, Changed, Fixed, Deprecated, Removed,
and Security as applicable. A release entry names the released version and
date only after the corresponding artifacts have been verified; this file does
not publish or imply a release.

## Unreleased

## 0.1.0-alpha.8 - 2026-07-24

### Changed

- Advanced the coherent public release train to `0.1.0-alpha.8` after alpha.7
  reached crates.io and npm but did not complete its Marketplace publication.
- Generated applications now declare the two pnpm lifecycle-script approvals
  required by the bundled Cloudflare tooling: `esbuild` and `workerd`.
- Scaffold conformance now performs a normal pnpm install so missing lifecycle
  approvals fail before publication.

### Fixed

- A fresh `pnpm create presolve` project now installs under pnpm 11 without an
  interactive `pnpm approve-builds` interruption.

## 0.1.0-alpha.7 - 2026-07-24

### Changed

- Renamed the public framework authoring package and compiler-recognized import
  from the npm-rejected unscoped `presolve` name to `@presolve/core`.
- Updated the scaffold, examples, framework fixtures, editor fixtures, and
  public documentation to use only `@presolve/core`; no unreleased legacy import
  alias remains.
- Advanced the release train to `0.1.0-alpha.7` after alpha.6 published
  its crates and scoped npm tooling before npm rejected the unscoped framework
  package name.

## 0.1.0-alpha.6 - 2026-07-24

### Changed

- Advanced the complete release train to `0.1.0-alpha.6` after npm rejected the
  alpha.5 scoped packages before their first publication.
- Tag releases now verify the npm publishing identity, Presolve organization,
  and scope access before publishing immutable crates or building native
  release artifacts.

## 0.1.0-alpha.5 - 2026-07-24

### Changed

- Advanced the complete release train to `0.1.0-alpha.5` after the alpha.4
  crates published without their downstream npm artifacts.
- Native CLI publication now validates the complete artifact set and dry-runs
  every tarball before the first registry write.

### Fixed

- Native CLI publishing now resolves every tarball to an absolute local path,
  preventing npm from interpreting release artifact paths as GitHub package
  specifications.
- Pre-tag release checks now exercise the same native publication helper used
  by the gated npm release job.

## 0.1.0-alpha.4 - 2026-07-24

### Changed

- Advanced the complete release train to `0.1.0-alpha.4` after the alpha.3
  Windows native package could not launch npm.
- Release dry runs now package and upload all four native CLI targets on their
  matching GitHub-hosted operating systems before any release tag is created.

### Fixed

- Native CLI packaging now launches Windows npm command shims through the
  Windows command processor and reports process-launch failures explicitly.

## 0.1.0-alpha.3 - 2026-07-24

### Changed

- Advanced the complete release train to `0.1.0-alpha.3` after the alpha.2
  crates published without their downstream npm artifacts.

### Fixed

- Native CLI tarballs now cross GitHub job boundaries through a visible
  `release-artifacts` directory accepted by `actions/upload-artifact`.

## 0.1.0-alpha.2 - 2026-07-24

### Changed

- Advanced the complete compiler, framework, tooling, and extension release
  train to `0.1.0-alpha.2` after the incomplete alpha.1 publication.

### Fixed

- Native CLI packaging now resolves package directories as validated local
  paths instead of npm GitHub package specifications.

## 0.1.0-alpha.1 - 2026-07-23

### Added

- Public `presolve`, `@presolve/cli`, and `create-presolve` release surfaces.
- A TypeScript 7 starter project and installable `presolve-vscode` extension
  package.
- A tag-gated npm/VS Code Marketplace release workflow with native CLI package
  staging.
- Public alpha documentation for framework authoring, metaframework workflow,
  tooling, Cloudflare deployment, and release operations.

### Changed

- The compiler, framework, application workflow, tooling, and release train
  identify `0.1.0-alpha.1`.
