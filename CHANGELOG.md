# Changelog

This repository follows Keep a Changelog-style entries. Changes are recorded
under `Unreleased` and grouped by Added, Changed, Fixed, Deprecated, Removed,
and Security as applicable. A release entry names the released version and
date only after the corresponding artifacts have been verified; this file does
not publish or imply a release.

## Unreleased

## 0.2.0-beta.7 - 2026-07-27

### Fixed

- JSX text now decodes standard HTML character references before compiler
  publication escapes the resulting text. Literal code examples such as
  `&lt;button&gt;`, quoted attributes, braces, and other named or numeric entities
  render correctly in generated static HTML instead of appearing double-escaped.

## 0.2.0-beta.6 - 2026-07-27

### Fixed

- `presolve explain` now recognizes the canonical decorator-free
  `class … extends Component` form, reports it as a component, and no longer
  emits the legacy decorator-only `PS0100` warning.

## 0.2.0-beta.5 - 2026-07-27

### Fixed

- Unsupported-platform CLI diagnostics now report the installed Presolve beta
  version instead of the retired `0.1 alpha` label.
- The public capability matrix and guides now distinguish canonical
  decorator-free V2 source from retained alpha-compatibility decorators.

## 0.2.0-beta.4 - 2026-07-27

### Fixed

- Cloudflare Workers Static Assets deployments now consume compiler-internal
  canonicalization redirects for every authored route, including nested routes
  such as `/docs/`, without exposing `/routes/...` artifact paths.

## 0.2.0-beta.3 - 2026-07-27

### Added

- Canonical application files: `app/app.tsx`, automatic `app/app.css`, and a
  strict `app/index.html` document template with compiler-owned `head`, `app`,
  and `runtime` placeholders. The former `app/layout.tsx` and `styles/` paths
  remain beta compatibility inputs.

### Fixed

- Cloudflare Workers Static Assets deployments now consume internal asset
  canonicalization redirects inside the worker, so `/` remains `/` instead of
  exposing the internal `/routes/root/` artifact path.

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
