# L8 Watch Mode Contract

L8 is an internal, transport-independent event and complete-candidate API. An
external observer reads any filesystem or editor state, constructs a complete
L7 workspace request, and submits it together with source-free change evidence.
Presolve does not watch, read, scan, poll, glob, resolve, or load any path.

`create_watch_session`, `submit_watch_change_batch`, `flush_watch_session`,
`poll_watch_events`, `inspect_watch_session`, and `stop_watch_session` are
internal compiler-service operations. There is no public `presolve watch`
command, dev server, browser refresh, HMR, editor integration, or streaming
transport.

Sessions are process-local. They bind one workspace ID, retain no authored
source text in snapshots or event journals, and disappear after restart. The
caller recreates the session and resubmits a complete candidate after restart.

Debounce is driven only by injected monotonic time and an explicit deterministic
scheduler turn. A pending window executes at the earlier quiet-period or
maximum-delay deadline. Before a turn, candidates coalesce to the highest
sequence complete replacement candidate while their source-free observations
are canonically unioned. Zero debounce still waits for the next scheduler turn.

At most one L7 whole-workspace operation is active for a session. A newer,
different candidate marks active work obsolete, retains one pending replacement,
and discards a late obsolete success from the watch publication. It never rolls
back valid L7 package or workspace publication. L5 reuse remains package-local
and ephemeral; L6 remains source-free, complete-result, and package-local.

Events are source-free, ordered by deterministic per-session sequence, and
polled non-destructively with bounded oldest-first eviction. A failed or
cancelled attempt retains the previous successful watch result. Watch reporting
cannot change L7 diagnostics, artifacts, scheduling, or publication.
