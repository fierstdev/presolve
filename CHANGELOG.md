# Changelog

This repository follows Keep a Changelog-style entries. Changes are recorded
under `Unreleased` and grouped by Added, Changed, Fixed, Deprecated, Removed,
and Security as applicable. A release entry names the released version and
date only after the corresponding artifacts have been verified; this file does
not publish or imply a release.

## Unreleased

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
