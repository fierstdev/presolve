# V2 computed source-recognition contract

V2 has no `computed()` intrinsic. A computed value is an instance getter of a
canonical V2 component only when compiler analysis proves that it reads
reactive State and is pure. An ordinary getter remains ordinary JavaScript.

## Source selection and proof

The parser selects non-static getter declarations from the general source AST.
The selected getter is not itself a framework intrinsic and therefore cannot
be classified by an import spelling or a resolved intrinsic registry entry.
It is admitted as computed only when all of the following inputs agree:

1. its owner is a component admitted by canonical inheritance lowering;
2. dependency analysis proves at least one direct or transitive read of a
   canonical State declaration for that component instance;
3. purity/effect analysis proves synchronous execution with no observable
   effect, unsupported call, or unknown call coverage; and
4. cycle analysis reports no dependency cycle.

The output keeps getter provenance, state dependency IDs, purity evidence, and
cycle status. Existing memoization, cache, dirty-flag, runtime, and resume
products may consume that output only after it is produced.

## Canonical-model amendment required

`CanonicalAuthoredSemanticModelV1` currently accepts framework intrinsics plus
TSX syntax candidates. It has no candidate form for an analysis-proven,
non-intrinsic getter. A computed source lowering must therefore introduce a
versioned derived-candidate variant and an explicit schema migration before
writing to the canonical model. Reusing `CanonicalIntrinsicKindV1::Computed`
would falsely claim that a `computed()` intrinsic exists; classifying getters
by method name would be equally invalid.

Until that amendment is implemented, legacy `@computed()` lowering remains
alpha compatibility only and the V2 beta path must not claim decorator-free
computed publication. This is an intentional proof boundary, not permission to
fall back to decorators.

## Required evidence

- a reactive getter and an ordinary getter in the same canonical component;
- a getter with an observable effect, async suspension, and an unknown call;
- direct and transitive State dependencies, conditional dependencies, and a
  dependency cycle;
- cold and resumed cache/dirty-flag evidence once the runtime adopts the
  derived candidate.
