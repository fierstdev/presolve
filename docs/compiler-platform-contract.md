# Compiler platform contract (L3)

L3 defines the `presolve_compiler::platform` boundary. Its v1 products are
`WorkspaceSnapshot`, `WorkspaceGraph`, `IncrementalPlan`,
`CompilerSessionState`, and the memory-only `ProductCache` inspection.

All deterministic documents are compact UTF-8 JSON ending in one newline.
They use SHA-256 typed identities, normalized relative workspace paths, exact
schema-version rejection, and canonical path/ID ordering. The platform owns
orchestration only: parser, semantic, lowering, runtime, resume, manifest, and
diagnostic authorities remain the completed Phase A-K compiler products.

A session exposes only one committed graph and snapshot. Candidate products are
attempt-local until validation completes. Cancellation, rejection, and platform
failure retain the preceding commit. Cache entries are process-local, keyed by
exact input revision/contract/configuration, and are never serialized as
compiler-internal persistence.

L3 introduces no daemon, watcher, IPC/socket protocol, alternate parser,
semantic ownership model, persistent cache, or parallel scheduler. Subsequent
Phase L work must add a schema version and explicit migration or rejection rule
before changing these v1 meanings.
