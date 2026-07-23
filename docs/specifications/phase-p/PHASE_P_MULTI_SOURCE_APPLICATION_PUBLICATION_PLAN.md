# Phase P: Multi-source application publication

**Status:** P4 complete; P5 requires the P0-P4 contracts.

## Objective

Unblock Phase O O4 with one compiler-owned, explicit multi-source application
publication product. It turns a caller-provided workspace, entry identity, and
package coordinates into one validated artifact set without a metaframework
merge, renderer, source parser, or artifact decoder.

## Governing boundary

```text
explicit workspace configuration + exact sources + explicit entry
        ↓
canonical compiler application-publication request
        ↓
validated application semantic model and compiler lowering
        ↓
atomic compiler-published artifact set and manifest
```

The existing single-entry `presolve build <source>` command remains unchanged.
Phase P adds a separately versioned multi-source product; it does not make
source discovery, implicit entry selection, routing, SSR, a dev server, HMR,
or a framework runtime available.

## Slice sequence

### P0 — publication constitution

Freeze request ownership, explicit entry selection, artifact atomicity,
identity/versioning, compatibility, and prohibited shortcuts.

### P1 — compiler request and validated entry selection

Add a public typed compiler request accepting exact `WorkspaceConfiguration`,
complete sources, an explicit logical entry path, package contracts, package
runtime mappings, output profile, and output root. Validate that the entry
resolves exactly one compiler-supported application root before lowering.

### P2 — canonical workspace lowering and artifact manifest

Generalize the existing publication path so it lowers the validated complete
workspace through existing compiler products. Publish a schema-v1 application
manifest binding compiler contract, workspace snapshot, entry identity, and
every emitted artifact digest/path.

### P3 — atomic CLI publication command

Add `presolve application build --config <path> --source <logical=relative>
--entry <logical> --out <directory>` with repeatable exact package-contract and
package-runtime mappings. Stage, validate, and atomically publish the complete
artifact set; preserve canonical diagnostics.

### P4 — application-product adoption

Extend `@presolve/application` only to project this new command. It must not
read sources, derive an entry, parse compiler results, or merge artifacts.

### P5 — proof and freeze

Prove source-order determinism, entry validation, relative-module composition,
Context/Slots/Forms, Resources, opaque terminals, production, resume,
malformed publication failure, atomic replacement, browser execution, and
incremental invalidation. Freeze the request/manifest/CLI compatibility matrix.

## Compatibility policy

The single-entry build command and its artifacts are frozen. The new command
and manifest are v1-only and fail closed on incompatible schema/compiler
contracts. There is no compatibility adapter that turns workspace inspection
results, independently built pages, or JavaScript-side artifact merging into an
application build.
