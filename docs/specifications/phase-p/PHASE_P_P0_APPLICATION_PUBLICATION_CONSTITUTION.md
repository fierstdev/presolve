# Phase P P0 application publication constitution

**Status:** P0 implementation authority.

## Why Phase P exists

Phase O can project explicit multi-source `workspace` and `watch --once`
requests, but neither produces browser application artifacts. The existing
artifact publisher accepts only one source path. Combining those outputs in
JavaScript would invent entry, ownership, artifact, and runtime authority.

Phase P therefore amends the frozen compiler platform with one canonical
multi-source application-publication product.

## Request authority

The future `ApplicationPublicationRequestV1` must contain an exact compiler
`WorkspaceConfiguration`, a non-empty uniquely logical-path-keyed complete
source set, one explicit logical `entry_path` from that set, caller-provided
semantic-package contracts and runtime-module mappings, one output profile,
and a caller-owned output root.

No field may be inferred from source roots, filenames, exports, package files,
or current working directory. The compiler validates the entry as a unique
compiler-supported application root. Missing, non-member, unresolved,
ambiguous, or unsupported entries fail before artifact publication.

## Publication authority

The compiler owns all publication semantics. It lowers the complete workspace
through canonical binding, semantic, dependency, identity, runtime, resume,
Resource, opaque-terminal, and optimization products. The result is one
`ApplicationPublicationManifestV1` binding compiler contract, workspace
snapshot ID, entry semantic identity, profile, and exact artifact path/digest
inventory.

Publication uses a sibling staging directory. The compiler validates every
artifact and manifest before atomically replacing the caller output root. A
failure leaves the prior output untouched and removes only its own staging
directory. Neither CLI nor metaframework may hand-merge artifact files.

## CLI and framework boundary

The later CLI command is:

```sh
presolve application build --config presolve.json \
  --source src/App.tsx=src/App.tsx --source src/Card.tsx=src/Card.tsx \
  --entry src/App.tsx --out dist
```

It reuses explicit containment and canonical package inputs.
`@presolve/application` may only project this command after P3. Framework and
metaframework never parse source, select an entry, inspect package code, decode
manifests, or execute generated code.

## Required proof

P1–P5 prove source-order-independent output; multi-file relative binding;
invalid entry rejection; package mapping failure; manifest validation; atomic
failure preservation; production/resume; Resource/opaque inclusion; browser
execution; and absence of source discovery or JavaScript artifact merging.
