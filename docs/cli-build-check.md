# CLI Build and Check

L9-D adds the service-backed public path:

```text
presolve build --config <file> --source <logical=relative-file> [--source ...]
presolve check --config <file> --source <logical=relative-file> [--source ...]
```

Every source is explicitly named. Relative host paths resolve below the configuration directory, must remain contained after physical resolution, and are read exactly once. The CLI sorts logical paths and delegates the complete candidate to L4 through L9-C. It never searches source roots, expands globs, infers membership, parses source, or generates output itself.

`--format json` emits one `presolve.cli-result` document. Existing pre-L9 build/check invocation forms remain available for frozen artifact and diagnostic compatibility.

The L9 path emits configuration and source-authority errors on stderr with exit
code `2`; successful JSON output is written to stdout. `--config` has no
implicit default and each build/check invocation requires at least one explicit
`--source`.
