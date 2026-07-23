# Phase P P1 application publication request

**Status:** P1 implementation authority and completion record.

`ApplicationPublicationRequestV1` is the compiler-owned pre-publication input.
It retains an exact `WorkspaceConfiguration`, parsed complete `CompilationUnit`,
explicit relative `entry_path`, semantic-package contract/runtime-module tables,
and caller output root. `validate_application_publication_request_v1` performs
no code generation or filesystem publication.

Validation rejects an empty or duplicate source set, invalid/non-member entry
path, missing rendered component root, and multiple rendered component roots in
the selected entry source. It builds the canonical application semantic model
using supplied package contracts and returns the exact entry `SemanticId`.
Source insertion order cannot affect that result.

P1 neither publishes artifacts nor validates output paths, writes staging
directories, adds a CLI command, infers an entry from exports, or changes the
single-entry build product. P2 owns workspace lowering and manifest projection.
