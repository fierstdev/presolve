# Deterministic incremental compilation (L5)

L5 is a session-local orchestration optimization over the canonical L3 clean
compiler. Every compiler-service compile request still owns the full normalized
configuration and complete source universe. The service discovers no files and
does not persist authored source text.

`IncrementalCompilationPlanV1` has discriminator
`presolve.incremental-compilation-plan.v1`; `IncrementalExecutionReportV1` has
discriminator `presolve.incremental-execution-report.v1`. Both use SHA-256
canonical fingerprints and ordered arrays. They contain identities,
fingerprints, and stable reason codes only, never source text, host paths,
timestamps, process identifiers, or memory addresses.

The live service retains one non-durable baseline only after L4 atomic
publication succeeds. It consists of the normalized configuration fingerprint,
source revision fingerprints, canonical L3 snapshot/graph, and L3-authorized
immutable parse products. Closing or restarting a service removes that state;
the next request is cold. Durable L4 files remain configuration plus canonical
snapshot/graph products only.

For a source-content edit, invalidation starts with the changed source unit and
walks only canonical L3 `WorkspaceGraph` compile-dependency edges in canonical
order. Unaffected parse products are reused only after L3 validates their
identity, source revision, product key, and normalized path. The full parser,
binder, semantic model, diagnostics, and graph validation remain L3-owned.

L3 v1 has no product-granular source-universe-membership dependency authority.
Consequently source additions, deletions, and rename-as-delete-plus-add use
`clean_fallback` with `L5F009_SOURCE_UNIVERSE_MEMBERSHIP_UNMODELED`.
Configuration changes use `clean_fallback` with
`L5F002_CONFIGURATION_CHANGED`. Malformed retained baseline products use
`L5F006_MALFORMED_BASELINE_GRAPH`. Fallback is a successful canonical compile,
not a diagnostic or changed exit status.

Execution modes are `cold`, `no_change`, `incremental`, and `clean_fallback`.
A no-change request returns the validated published L3 result and does not
rewrite the durable publication. Optional service reports are selected with
`none`, `summary`, or `full`; `none` preserves the L4 compile surface.

The explicit test-only verification mode also runs an isolated clean L3 build
of the candidate and compares canonical snapshot and graph bytes before
publication. It never reads or changes the live baseline, and clean comparison
products are never published.

L5 does not provide watch mode, partial updates, rename detection, persistent
cache, workspace scheduling, remote execution, parallel scheduling, or
performance guarantees. Persistent-cache work begins no earlier than L6.
