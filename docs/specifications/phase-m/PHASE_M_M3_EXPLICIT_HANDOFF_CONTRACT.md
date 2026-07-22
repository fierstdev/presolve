# Phase M M3 explicit handoff contract

**Status:** M3 implementation authority and completion record.

## Scope

M3 adds one private `@presolve/framework` package that creates and dispatches
an exact, caller-supplied L9 explicit-project invocation. Its only supported
commands are `build` and `check`:

```text
presolve <build|check> --config <path> --source <logical=relative-file> [--source ...] --format json
```

The package does not read the configuration or source paths. It does not add a
target profile or output-root field: those are not independent L9 command
arguments. Configuration remains opaque and owns its existing fields. The
framework preserves source-entry order exactly; the CLI retains its own frozen
sorting and source-authority behavior.

## API boundary

`createExplicitProjectInvocation(request)` accepts only:

- `command`: exactly `build` or `check`;
- `configurationPath`: a non-empty caller-supplied string; and
- `sources`: a non-empty ordered list of non-empty `logicalPath` and
  `relativePath` strings.

It returns immutable `{ executable: "presolve", arguments }` data. It does
not stat, normalize, resolve, open, glob, sort, parse, compile, or retain the
input paths.

`invokeExplicitProject(request, execute)` passes that immutable invocation to a
caller-supplied one-shot executor and returns the executor's result unchanged.
The framework does not decode stdout, stderr, diagnostics, or CLI JSON, and it
does not map an exit code to a framework status. The executor seam keeps process
ownership outside this small framework package while giving the later product
one exact canonical command request.

## Product and diagnostic policy

The L9 JSON result currently publishes command status and snapshot identities;
it does not publish framework artifact locations. M3 must treat stdout and
stderr as opaque bytes and must not infer artifact paths from source names,
cache layout, snapshot IDs, or command output.

The framework cannot turn a CLI failure into a framework diagnostic. Canonical
diagnostic bytes and exit behavior pass through the executor boundary unchanged.

## Evidence

`./scripts/verify-m3-framework-handoff.sh` proves:

1. private package identity and dependency-free source boundary;
2. exact argument construction for build and check, including ordered repeated
   `--source` entries and fixed JSON format;
3. rejection only of malformed framework request shapes, with no source access;
4. one-shot executor invocation and unchanged opaque result identity; and
5. absence of filesystem, process, network, parser, compiler, decoder,
   source-transform, and runtime imports.

It inherits M2 and M0/M1 verification.

## Next boundary

M4 begins with a public artifact-publication capability audit. The accepted L9
build result does not itself provide static HTML or runtime-artifact locations.
No framework publication path, legacy-command adapter, artifact decoder, or
browser integration may be invented to bridge that gap. Only an already-frozen
public publication path may authorize the Counter vertical proof.
