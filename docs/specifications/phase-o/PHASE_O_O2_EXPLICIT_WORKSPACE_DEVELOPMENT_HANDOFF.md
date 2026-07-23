# Phase O O2 explicit workspace development handoff

**Status:** O2 implementation authority.

`createApplicationWorkspaceInvocation(request)` and
`createApplicationWatchOnceInvocation(request)` accept a caller-owned
`configurationPath`, non-empty explicit `sources` array of `logical=relative`
specifications, and optional `format: "human" | "json"`. Workspace additionally
accepts `verifyCleanEquivalence: true`.

They project only to the existing CLI forms:

```sh
presolve workspace --config <path> --source <logical=relative> ...
presolve watch --once --config <path> --source <logical=relative> ...
```

O2 does not open a file watcher, retain a workspace, start an HTTP server,
provide HMR, discover sources, or decode workspace/watch results. Callers own
execution and receive the result unchanged.
