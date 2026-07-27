# Changelog

This repository follows Keep a Changelog-style entries. Changes are recorded
under `Unreleased` and grouped by Added, Changed, Fixed, Deprecated, Removed,
and Security as applicable. A release entry names the released version and
date only after the corresponding artifacts have been verified; this file does
not publish or imply a release.

## Unreleased

## 0.2.0-beta.2 - 2026-07-27

### Fixed

- Decorator-free `Component` layouts can declare V2 slot fields with
  `children: SlotContent = slot()` through the published TypeScript authority
  bridge, without falling back to the legacy `@component()` diagnostic.
- `styles/` and `public/` are copied atomically into `dist/`, integrity-listed
  for deployment, and served by development and Node static hosts.

## 0.2.0-beta.1 - 2026-07-26

### Added

- Completion-grade V2 structural lifecycle, context, form, resource, route
  handoff, capability, and beta hardening evidence.
- The explicit closed beta Action surface, packed-scaffold verification, and
  tag-triggered beta publication workflow.

### Changed

- The compiler, framework, tooling, scaffold, native CLI packages, and VS Code
  extension now identify the `0.2.0-beta.1` compatibility train.

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
