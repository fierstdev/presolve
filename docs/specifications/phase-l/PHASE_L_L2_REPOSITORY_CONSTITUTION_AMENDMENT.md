# Phase L Amendment L2-A — Authoritative Repository Classification and Migration Map

**Status:** Authoritative amendment  
**Applies to:** `PHASE_L_SLICES_L1_L10.md`, slice L2  
**Prerequisite:** L1 complete and committed  
**Known L1 commit:** `f269ba0` (`chore: transition public identity to presolve`)

## 1. Purpose

This amendment supplies the repository mapping omitted by the original L2 contract.

It replaces every L2 instruction that could be read as requiring a speculative migration to root-level `/compiler`, `/runtime`, `/cli`, or `/tests` directories.

L2 is a **repository classification, archival, and hygiene slice**. It does not repartition the compiler, parser, runtime, fixture system, package graph, or build graph. Product-boundary changes occur only in later slices that explicitly authorize them.

## 2. Governing rule

The current working repository, after L1, is authoritative for active implementation paths.

L2 shall:

1. preserve all active source and verification paths;
2. classify every tracked path;
3. archive historical material without rewriting its contents;
4. remove only proven disposable material;
5. add enforcement that prevents repository-layout regression; and
6. leave all build, test, browser, schema, fixture, and generated-output gates unchanged.

A path may be moved only where this amendment explicitly authorizes the move.

## 3. Canonical public repository layout after L2

The following root boundaries are authoritative:

```text
.github/                 GitHub configuration and workflows
adr/                     accepted architectural decision records
crates/                  active Rust crates, including compiler, parser, CLI, and supporting crates
docs/                    public documentation and archived engineering history
examples/                active canonical examples
fixtures/                active compiler/runtime/browser contract fixtures
packages/                active JavaScript/TypeScript packages, including runtime and tooling
rfcs/                     accepted or active technical RFCs
schemas/                  frozen and active versioned schemas
scripts/                  maintained repository automation only
tools/                    optional maintained developer/repository tools when introduced by an authorized slice
benches/                  Rust benchmark targets when present
benchmarks/               cross-platform or product benchmark assets when introduced by an authorized slice
tests/                    root integration tests only when already present or explicitly introduced by a later slice
```

The absence of `tools/`, `benches/`, `benchmarks/`, or root `tests/` is valid. L2 shall not create empty directories.

The following root directories are **not required and shall not be created in L2**:

```text
compiler/
runtime/
cli/
```

Their responsibilities remain represented by `crates/` and `packages/`.

## 4. Active implementation map

### 4.1 Rust compiler, parser, CLI, and support crates

All tracked paths under `crates/` that participate in any workspace, build, test, browser, fixture, schema, or release gate are active implementation.

Authoritative action:

- **KEEP IN PLACE.**
- Do not consolidate crates.
- Do not split crates.
- Do not rename crate directories in L2.
- Do not move parser modules or parser crates.
- Do not create a root `compiler/` directory.
- Do not create a root `cli/` directory.

Any remaining internal `ezc_*` crate directory or Rust crate name after L1 is governed by L1's completed compatibility and identity decisions. L2 shall not reopen that decision.

### 4.2 JavaScript/TypeScript packages

All tracked paths under `packages/` referenced by the package workspace, lockfile, build scripts, examples, browser tests, schemas, or runtime contracts are active implementation.

Authoritative action:

- **KEEP IN PLACE.**
- Do not move the runtime to root `runtime/`.
- Do not flatten packages.
- Do not create packages merely to match the future package list.
- Do not alter package boundaries in L2.

### 4.3 Examples

All tracked paths under `examples/` are active examples unless a file is proven unreachable, obsolete, and superseded by a tested canonical equivalent.

Authoritative action:

- **KEEP IN PLACE.**
- Do not rewrite authored examples except for broken path or metadata repairs caused by an authorized archival move.
- Do not delete an example merely because later Phase L slices require additional examples.

### 4.4 Fixtures and golden artifacts

All tracked paths under `fixtures/`, and any fixture or golden directories nested elsewhere, are frozen verification assets when referenced by tests, scripts, manifests, snapshots, or documentation gates.

Authoritative action:

- **KEEP IN PLACE AND BYTE-PRESERVE CONTENT.**
- Do not rename fixture identities.
- Do not renumber fixtures.
- Do not regroup fixtures into new category directories.
- Do not regenerate expected outputs solely for repository cleanup.
- Do not delete apparently duplicate fixtures without executable proof that no test, script, manifest, or documentation reference depends on them.

L2 may remove only ignored or untracked generated fixture output after confirming it is reproducible and not part of a golden contract.

### 4.5 Schemas

All tracked paths under `schemas/` are active or frozen contracts.

Authoritative action:

- **KEEP IN PLACE AND BYTE-PRESERVE SCHEMA CONTENT.**
- Do not rename schema files.
- Do not change `$id`, title, version, field order, wording, or formatting.
- Do not move schemas into package-local directories.

### 4.6 ADRs and RFCs

Tracked `adr/` and `rfcs/` documents are permanent decision history.

Authoritative action:

- **KEEP IN PLACE.**
- Do not rewrite historical names, repository URLs, decisions, statuses, or examples merely to make them current.
- A brief top-level historical-context notice may be added only if required to prevent a reader from mistaking an old document for current public guidance.
- New superseding decisions require a new ADR or RFC; old records remain immutable.

## 5. Historical and planning material map

### 5.1 Historical archive root

Create the following directory only when at least one authorized move is required:

```text
docs/archive/engineering/
```

Create an archive index:

```text
docs/archive/engineering/README.md
```

The index shall state that archived material records EdgeZero-era and pre-public Presolve engineering history, may contain obsolete names or proposals, and is non-normative unless another authoritative document explicitly incorporates it.

### 5.2 Planning documents

The following material is historical unless currently linked as public product documentation:

```text
docs/planning/**
```

Authoritative destination:

```text
docs/archive/engineering/planning/**
```

Rules:

- Move with history-preserving filesystem operations.
- Preserve file contents byte-for-byte.
- Preserve relative structure beneath `docs/planning/`.
- Update only active navigation, scripts, or links that must locate the moved files.
- Do not update links inside the archived documents themselves unless a verification gate requires link integrity within the archive; prefer archive-index context over historical rewriting.

If `docs/planning/` contains a document explicitly identified by the current repository as a live Phase L authority, keep that document in its current authoritative location or move it to `docs/specifications/phase-l/` only when the supplied Phase L document packaging contract requires it. Do not archive active constitutional specifications.

### 5.3 Progress logs and handoffs

Historical progress logs, weekly logs, and completed handoffs are engineering records.

Source patterns may include:

```text
notes/progress/**
**/AGENT_HANDOFF*.md
```

Authoritative destination for completed historical records:

```text
docs/archive/engineering/progress/**
docs/archive/engineering/handoffs/**
```

Exceptions:

- The single current progress log used by the established continuation workflow shall remain at its established live path during L2.
- The single current `AGENT_HANDOFF.md` shall remain at its established live path.
- The weekly-log script shall continue to target the live progress location.
- A completed log or handoff may be archived only after a new live successor exists and all continuation tooling references the successor.

L2 shall not redesign the continuation workflow.

### 5.4 Learning notes and resource notes

Source patterns may include:

```text
notes/learning/**
notes/resources/**
```

Classification procedure:

- Material referenced by active public docs or contributor workflow: move to an appropriate live `docs/` location with link updates.
- Internal historical research or learning records: move to `docs/archive/engineering/learning/` or `docs/archive/engineering/resources/`.
- Empty directories: remove.

Contents shall not be substantively rewritten in L2.

### 5.5 Spikes

Source patterns may include:

```text
notes/spikes/**
spikes/**
**/spike/**
```

Every spike shall be classified by executable evidence:

1. **Accepted and incorporated:** archive source notes under `docs/archive/engineering/spikes/accepted/`; active implementation remains in its existing source path.
2. **Rejected or superseded:** archive under `docs/archive/engineering/spikes/rejected/` with no content rewrite.
3. **Still executed by a current gate:** it is not historical; keep it in place until a later explicit slice replaces or retires it.
4. **Generated scratch output:** delete only when untracked or ignored and reproducible.

No spike implementation, parser experiment, benchmark, or fixture may be deleted based only on its name.

## 6. Phase L specification placement

The authoritative Phase L Markdown documents supplied to Codex shall be tracked under:

```text
docs/specifications/phase-l/
```

Use their supplied filenames unchanged.

The directory shall include an `README.md` that lists the documents in authority order and states that they govern Phase L.

Do not place these documents in the historical archive.

## 7. Deletion contract

L2 may delete a tracked file only when all of the following are proven:

1. it is not referenced by Cargo, pnpm, `just`, CI, scripts, schemas, fixtures, examples, documentation navigation, release metadata, or tests;
2. it does not contain an accepted ADR, RFC, frozen schema, golden output, historical progress record, or authoritative specification;
3. it is generated, superseded, or empty;
4. its removal is covered by a focused repository-hygiene test or audit;
5. all complete gates pass after deletion; and
6. the progress log records the exact deletion and evidence.

When those conditions are not met, keep or archive the file.

Directories that are empty after authorized moves shall be removed.

## 8. Repository-control files

The following remain at repository root when present:

```text
Cargo.toml
Cargo.lock
package.json
pnpm-lock.yaml
pnpm-workspace.yaml
justfile
rust-toolchain.toml
.gitignore
README.md
LICENSE*
CHANGELOG.md
CONTRIBUTING.md
SECURITY.md
CODE_OF_CONDUCT.md
```

L2 may update path references caused by authorized moves. It shall not redesign commands, package boundaries, release policy, or public documentation content assigned to later slices.

## 9. Required L2 deliverables

L2 shall produce:

1. an inventory of every tracked root path and its classification;
2. the archive root and archive index if historical moves occur;
3. the Phase L specification directory and authority index;
4. only the moves and deletions authorized above;
5. repaired active links and automation paths;
6. a machine-executable repository-layout audit;
7. a machine-executable public-identity audit retained from L1 or extended without changing L1 semantics;
8. updated contributor-facing repository map limited to current paths;
9. progress and handoff updates; and
10. one L2 commit.

The inventory shall be stored at:

```text
docs/repository-layout.md
```

It shall describe current active boundaries, historical archive policy, and ownership of root directories. It shall not duplicate future package or service architecture.

## 10. Repository-layout audit

Add a deterministic audit invoked by the repository's normal validation surface.

The audit shall fail when:

- forbidden speculative root directories `compiler/`, `runtime/`, or `cli/` are introduced;
- authoritative Phase L specifications are missing from `docs/specifications/phase-l/`;
- historical planning material remains under `docs/planning/` after it was classified for archival;
- archived material appears in public documentation navigation;
- active workspace members point into `docs/archive/`;
- schemas or fixtures are moved into the archive;
- a tracked cache, build output, secret, or machine-local file is present;
- required active root control files are missing; or
- the documented repository map disagrees with actual root ownership.

The audit shall not encode a list of future crates or packages.

## 11. Verification gate

L2 is complete only after all of the following pass from a clean worktree candidate:

1. focused repository-layout audit;
2. public-identity audit;
3. Phase L specification authority-index check;
4. archive-link/navigation check;
5. full Rust workspace tests;
6. full JavaScript/TypeScript workspace tests;
7. browser tests;
8. fixture/golden verification;
9. schema validation;
10. documentation link checks currently available;
11. `just check` or the repository's canonical aggregate equivalent;
12. generated-output comparison proving no semantic artifact change; and
13. clean-worktree verification after the L2 commit.

Where a listed gate does not yet exist, L2 shall invoke the closest established repository gate and record the absence. L2 shall not invent a new test framework solely for this slice.

## 12. Non-goals

L2 does not authorize:

- compiler or parser repartitioning;
- crate renaming or package renaming beyond completed L1 work;
- introduction of the compiler service;
- introduction of incremental compilation;
- cache architecture;
- workspace semantic changes;
- CLI command expansion;
- public documentation rewrite;
- example expansion;
- CI/CD redesign;
- release publishing;
- GitHub repository creation or visibility changes;
- schema changes;
- fixture regeneration; or
- deletion of historical engineering evidence.

## 13. Completion state

At L2 completion:

- active compiler, parser, runtime, package, schema, example, and fixture paths remain stable;
- historical planning and inactive research material are clearly archived;
- live continuation records remain usable;
- Phase L specifications have a canonical tracked location;
- no speculative product-boundary migration has occurred;
- repository ownership is documented and executable audits enforce it;
- all inherited behavior and verification gates pass; and
- the repository is clean at the committed L2 boundary.

Only then may L3 begin.
