# CLI Cache and Clean

L9-E exposes the existing L6 persistent-cache operations without adding cache
state or interpreting cache entries:

```text
presolve cache [inspect|verify|clean] --config <file> [--format human|json]
presolve clean --config <file> [--format human|json]
```

The configuration path is explicit. Its parent is the project root and the
only cache location selected by this command is `<project>/.presolve/cache`.
`inspect` and `verify` project L6's canonical cache-inspection JSON unchanged.
`clean` delegates solely to L6's guarded cache-entry cleanup; it never removes
the project root, source files, output directories, or sibling files.

Cache-operation failures are written to stderr with exit code `5`; invalid or
unreadable explicit configuration uses exit code `2`.
