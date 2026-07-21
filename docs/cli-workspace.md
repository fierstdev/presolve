# CLI Workspace

L9-F provides an explicit single-project L7 workspace operation:

```text
presolve workspace --config <file> --source <logical=relative-file> [--source ...] [--format human|json]
```

This command does not discover packages, manifests, dependencies, or sources.
It forms one complete caller-owned package candidate and invokes L7's serial
workspace compilation API through the existing service host. The JSON result
contains the resulting workspace, manifest, graph, plan, and package snapshot
identities.
