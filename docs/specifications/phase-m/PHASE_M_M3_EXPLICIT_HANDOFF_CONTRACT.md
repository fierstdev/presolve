# Phase M M3 canonical artifact handoff contract

**Status:** Superseded and narrowed by the M4 publication decision.

## Current scope

The original L9 project-status handoff was an honest command adapter, but it
does not reach the compiler's HTML/runtime/resume publisher. M4 therefore
selects the existing full compiler build command as the framework's only
current publication boundary:

```text
presolve build <source> --out <directory> [--production]
```

`@presolve/framework` is private and exposes only
`createArtifactBuildInvocation(request)` and
`invokeArtifactBuild(request, execute)`. This is not a compatibility shim: it
is the current compiler path that produces the artifacts the framework must
prove.

## API boundary

`createArtifactBuildInvocation(request)` accepts only caller-supplied:

- `sourcePath`: one non-empty source path;
- `outputDirectory`: one non-empty output directory; and
- optional `production`: a boolean.

It returns immutable `{ executable: "presolve", arguments }` data. It does
not stat, normalize, resolve, open, glob, parse, compile, or retain either
path. `invokeArtifactBuild(request, execute)` calls a caller-owned executor
once and returns its result unchanged.

The framework does not decode stdout, stderr, diagnostics, or generated files;
the compiler owns publication and the caller owns process execution. A failed
compiler invocation remains a failed compiler invocation.

## Evidence

`./scripts/verify-m3-framework-handoff.sh` proves private package identity,
exact artifact-build argument construction, malformed request rejection,
opaque executor-result identity, and absence of filesystem, process, network,
parser, compiler, decoder, source-transform, and runtime imports. It inherits
the M2 type conformance proof.

## Historical disposition

The old `--config --source --format json` request builder is removed from the
private package. It remains a compiler service/status command, not a framework
artifact publication API. Phase M does not provide two framework build models.
