# Phase L midpoint tooling capability inventory

This is the L10-B midpoint inventory. It records the exact Phase L product
surface accepted through L10-A; it does not create a tool, public reader, or
new compiler product. The L10 compatibility corpus freezes the referenced
L3--L8 fixture bytes.

| Registered schema | Owner and canonical reader | Current public projection | Persistence class | Capability deliberately absent |
| --- | --- | --- | --- | --- |
| `presolve.workspace-configuration` v1 | L3 `WorkspaceConfiguration`; validation and `canonical_workspace_configuration_json_v1` | L9's distinct CLI codec constructs it but does not expose an L3 decoder | request-local; no public durable decoder | project discovery, migration, cross-codec decoding |
| `presolve.workspace-snapshot` v1 | L3 `WorkspaceSnapshot::to_canonical_json` / `decode_workspace_snapshot_json_v1` | none | canonical result; L4/L6 may retain only under their existing contracts | semantic-query, artifact, or source-text inspection |
| `presolve.workspace-graph` v1 | L3 `WorkspaceGraph::to_canonical_json` / `decode_workspace_graph_json_v1` | none | canonical result; L4/L6 may retain only under their existing contracts | graph reconstruction or inferred dependency facts |
| `presolve.compiler-service-protocol` v1 | L4 `encode_frame` / `decode_frame` | none | local durable service protocol; no network transport | daemon, RPC, or remote service |
| `presolve.persistent-artifact-cache` v1 | L6 `PersistentArtifactCacheV1` and its private payload validator | `presolve cache` operations only | source-free complete-result cache | public cache-payload decoder, remote cache, parser-product persistence |
| `presolve.cache-inspection-report.v1` v1 | L6 `CacheInspectionReportV1::to_canonical_json` | `presolve cache inspect` and `verify` project it unchanged | transient inspection over L6 state | cache authority or source inspection |
| `presolve.workspace-manifest` v1 | L7 `WorkspaceManifestV1`, `graph`, and `plan` | explicit single-project `presolve workspace` projection | caller-owned request; no discovery | manifest discovery, inferred packages, cross-package semantics |
| `presolve.watch-session-configuration` v1 | L8 `WatchSessionV1::new` | L9 `watch --once` adapter | process-local; never restored | filesystem watcher, dev server, HMR, stream transport |
| `presolve.watch-change-batch` v1 | L8 `submit_change_batch` | L9 `watch --once` adapter | transient caller-supplied input | source-bearing journal or autonomous observer |
| `presolve.watch-execution-plan` v1 | L8 `WatchExecutionPlanV1` | none | process-local execution result | a build trace or wall-clock timeline |
| `presolve.watch-event` v1 | L8 `WatchEventV1` | none | bounded process-local source-free journal | durable event history or streaming transport |
| `presolve.watch-session-snapshot` v1 | L8 `WatchSessionSnapshotV1` | none | process-local session inspection | restart restoration or source retention |
| `presolve.watch-execution-report` v1 | L8 `WatchExecutionReportV1` | L9 `watch --once` result envelope | transient process-local execution result | compiler trace, profiling, or artifact facts |

The registry intentionally leaves these names `reserved`: `presolve.build-trace`,
`presolve.compile-cost-report`, and `presolve.artifact-graph`. L11 must first
add an accepted producer contract, canonical serializer, strict validation,
fixture, provenance/identity proof, and compatibility proof before one becomes
available.

The current products also do not establish editor-query facts for hover,
completion, rename, references, signature help, semantic tokens, or
source-mapping. L12 begins with a capability audit; it may not replace a
missing product by parsing, binding, or analyzing source outside the compiler.
