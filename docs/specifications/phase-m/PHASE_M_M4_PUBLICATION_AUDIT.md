# Phase M M4 public artifact-publication audit

**Status:** M4 audit complete; Counter vertical slice blocked.

## Question

Can the conformance-first framework obtain static HTML and runtime artifacts for
the exact frozen Counter source through a public, accepted compiler publication
boundary, without source translation, artifact decoding, or a new compiler
behavior?

## Evidence

The accepted L9 project commands are exactly:

```text
presolve build --config <file> --source <logical=relative-file> [--source ...]
presolve check --config <file> --source <logical=relative-file> [--source ...]
```

Their documented output is one `presolve.cli-result` JSON document. A live M4
build of the canonical Counter returned command status, workspace identity, and
snapshot identities only; it provided no HTML, runtime-artifact, output-root,
or publication location.

The older command form `presolve build <file> --out <directory>` does emit
`index.html`, `runtime.js`, and the frozen runtime artifacts. M4 verified that
behavior in an isolated temporary directory. It is nevertheless excluded:
Phase L classifies legacy compiler commands as frozen compatibility paths, not
new platform adapters. M0 therefore cannot wrap that command without widening
the accepted framework/compiler boundary.

The available artifact-graph tooling is also not a publication path. It
requires already-valid supplied product bytes and must not inspect a build
directory or invoke compilation. The runtime package consumes compiler-emitted
artifacts; it does not publish them.

## Decision

No existing accepted public publication boundary can satisfy the M4 Counter
artifact proof. The framework must not:

- invoke or wrap the legacy `--out` path;
- infer artifact locations from snapshot IDs, caches, source names, or output
  conventions;
- decode compiler products to reconstruct output; or
- add a writer, renderer, hydration layer, or compiler behavior.

Consequently the Counter browser/runtime proof is blocked before implementation.
M5 through M9 remain deferred because the roadmap requires a clean Counter
vertical slice first.

## Required owner decision

The only non-compiler-change route is a new explicit Phase M boundary decision
that authorizes a strictly scoped adapter over the already-frozen legacy
artifact-publication command, including its exact source/input grammar,
published file set, diagnostics, browser evidence, compatibility matrix, and
failure behavior. That would be a deliberate framework-boundary change, not a
silent implementation choice.

Absent that decision, Phase M stops at M4. A new compiler publication product
or any modification to frozen compiler behavior is outside the authorized work.
