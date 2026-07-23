# Phase P P4 application-product adoption

**Status:** P4 implementation authority.

`@presolve/application` projects the P3 command through
`createApplicationPublicationInvocation` and `invokeApplicationPublication`.
The request is caller-owned: configuration path, complete exact source list,
logical entry path, output publication pointer, explicit package mappings, and
production profile. The projection is:

```text
presolve application build --config <path> --source <logical=relative> ...
  --entry <logical> --out <pointer> [package mappings] [--production]
```

The package preserves source order, sorts only independent package mapping
objects for deterministic arguments, invokes a caller-provided executor, and
returns the executor result unchanged. It does not read application files,
select an entry, inspect the manifest, merge artifacts, or serve generated
output. `APPLICATION_COMMAND_SCHEMA_VERSION = 1` also recognizes the explicit
`application-build` selector without changing the legacy single-entry `build`
selector.
