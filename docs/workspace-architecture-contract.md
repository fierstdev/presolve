# Workspace architecture (L7)

L7 coordinates complete caller-supplied package requests. The service neither
loads nor discovers `presolve.workspace.json`; that future filename is only a
documented schema convention. Each package remains an independent L4/L5/L6
compiler unit with unchanged diagnostics and artifacts.

`WorkspaceManifestV1` uses explicit package/session mappings and explicit
dependency edges. Edges impose deterministic serial topological scheduling
only: they do not create semantic binding, module resolution, artifact linking,
or cache-key changes. L7 v1 is whole-workspace only and fail-fast.

Package publication remains package-atomic. Durable source-free workspace state
publishes only after every package succeeds; a later package failure preserves
the prior workspace state while retaining any earlier valid package commits.
L5 baselines remain package-local and ephemeral; L6 may serve a complete
package result after restart. No watch mode, public workspace CLI, filesystem
discovery, parallelism, or remote execution is provided.
