# Phase O O3 application command envelope

**Status:** O3 implementation authority.

`createApplicationCommandInvocation(request)` accepts a versioned caller-owned
envelope:

```ts
{ schemaVersion: 1, command: "build" | "workspace" | "watch-once", input: object }
```

It selects only the O1/O2 immutable invocation projector and returns its
result. `invokeApplicationCommand(request, execute)` delegates the invocation
to the caller executor and returns the executor result unchanged. It does not
read command stdout/stderr, map diagnostics, parse JSON, inspect artifact paths,
or manufacture success/failure objects.

O3 is a request API, not a project configuration file or RPC protocol. Unknown
schema versions and commands fail before any executor is called.
