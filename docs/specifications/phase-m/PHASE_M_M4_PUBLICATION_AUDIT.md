# Phase M M4 canonical artifact-publication decision

**Status:** M4 complete.

## Decision

The compiler's existing artifact publisher is the canonical Phase M build
boundary:

```text
presolve build <source> --out <directory> [--production]
```

It is the only current public path that generates `index.html`, the template
manifest, compiler runtime plans, the browser runtime, resume records, and
production products. The framework invokes that command through its inert
handoff package and does not parse source, write artifacts, or host a runtime.

The L9 `--config --source --format json` path remains useful for explicit
project/session compilation status, but it does not publish browser artifacts.
It is not a second framework build path. The previous “legacy compatibility”
classification is retired for Phase M rather than allowed to block the actual
compiler code generator.

## Counter proof contract

The exact Counter source is shared byte-for-byte by the TypeScript fixture and
the example. It declares compiler-recognized component, State, and Action
forms, then binds the Action directly to its button.

M4 must prove:

1. the TypeScript 7.0 declaration fixture resolves with no framework runtime;
2. `@presolve/framework` constructs exactly the canonical artifact-build
   invocation without opening either caller path;
3. the compiler emits static HTML, runtime, manifest, and resume artifacts;
4. a real browser clicks Counter and observes `Count: 1`; and
5. the proof introduces no hydration step, framework renderer, state store,
   scheduler, product decoder, or compiler semantic change.

## Publication ownership

The compiler owns every emitted file and all diagnostics. The framework sees
only caller-provided source/output strings and an opaque executor result. It
cannot derive paths from compiler IDs, caches, or output content, and it cannot
reconstruct any artifact.

## Next boundary

Counter's TypeScript, artifact, and real-browser proof passes. M5 may expose
the next existing declaration family only after its compiler fixtures and
browser evidence are selected. No router, server, dev process, scaffold,
package installation, or metaframework feature is authorized.
