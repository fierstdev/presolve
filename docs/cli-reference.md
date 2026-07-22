# Presolve CLI reference

This is a **Reference** for the accepted L9 and L11 command adapters. Every
marked command below is executed by `verify-l13b-public-cli-docs.sh` against
the listed explicit input. `presolve` denotes the installed public executable;
the repository verifier runs the identical arguments through the local Cargo
package.

## L9 explicit-project commands

All project operations require an explicit `--config` and explicit
`--source <logical=relative-file>` membership where shown. They do not discover
projects, manifests, dependencies, or sources.

<!-- presolve-snippet: id=l13b-version-json; kind=command -->
```sh
presolve version --format json
```

<!-- presolve-snippet: id=l13b-help; kind=command -->
```sh
presolve help
```

<!-- presolve-snippet: id=l13b-check-counter; kind=command -->
```sh
presolve check --config examples/counter/presolve.json --source counter.tsx=src/Counter.tsx --format json
```

<!-- presolve-snippet: id=l13b-build-forms; kind=command -->
```sh
presolve build --config examples/forms/presolve.json --source Forms.tsx=src/Forms.tsx --format json
```

<!-- presolve-snippet: id=l13b-workspace; kind=command -->
```sh
presolve workspace --config examples/explicit-workspace/presolve.json --source src/main.ts=src/main.ts --format json
```

<!-- presolve-snippet: id=l13b-watch-once; kind=command -->
```sh
presolve watch --once --config examples/components-context-slots/presolve.json --source Composition.tsx=src/Composition.tsx --format json
```

<!-- presolve-snippet: id=l13b-cache; kind=command -->
```sh
presolve cache inspect --config examples/counter/presolve.json --format json
presolve cache verify --config examples/counter/presolve.json --format json
presolve clean --config examples/counter/presolve.json --format json
```

`cache clean` and `clean` are limited to the selected project's cache entries.
They do not remove the project root, source, build output, or sibling paths.
`watch` accepts only one complete `--once` replacement candidate; it does not
add a daemon, event discovery, debounce, or cancellation behavior.

## L11 validated-product commands

L11 commands take exactly one named, already-valid product. They never compile,
discover a project, read a build directory, or reconstruct a product. The
established command grammars are:

| Command | Required product schema | Formats |
| --- | --- | --- |
| `inspect workspace-snapshot` | `presolve.workspace-snapshot` | human, json |
| `inspect workspace-graph` | `presolve.workspace-graph` | human, json |
| `graph workspace` | `presolve.workspace-graph` | human, json, dot |
| `graph artifact` | `presolve.artifact-graph` | human, json, dot |
| `trace` | `presolve.build-trace` | human, json |
| `profile` | `presolve.compile-cost-report` | human, json |

Each uses `--schema <listed-schema> --product <file>`. The verifier executes
the real L11 fixture tests, which construct and strictly validate the required
product bytes before invoking each command; this reference intentionally does
not offer a placeholder filename as a runnable example.

## Reserved and excluded commands

`create`, `dev`, `benchmark`, and `doctor` are recognized reserved commands and
exit `6`; they have no canonical adapter. No public documentation here grants
source discovery, scaffolding, server, benchmark, telemetry, deployment, or
editor-write authority. Legacy compiler inspection commands are documented by
their frozen contracts, not redefined by this L9/L11 reference.
