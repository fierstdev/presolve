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

## Canonical-model amendment

The [derived computed-candidate contract](computed-derived-candidate-contract.md)
defines the schema-v3 candidate and evidence required to represent an
analysis-proven, non-intrinsic getter. Reusing
`CanonicalIntrinsicKindV1::Computed` would falsely claim that a `computed()`
intrinsic exists; classifying getters by method name would be equally invalid.

The first implementation admits its finite call-free subset, including
analysis-proven transitive computed-to-computed State dependencies. Legacy
`@computed()` lowering remains alpha compatibility only and is never a
fallback for a V2 getter. Calls and unknown-call coverage remain an intentional
proof boundary.

## Initial acceptance evidence

- A reactive getter and ordinary getter in the same canonical component are
  distinguished without decorator recognition; only the direct-State getter is
  admitted.
- A transitive getter chain receives explicit direct-computed and reachable
  State evidence; a dependency cycle receives no candidate.
- A static getter, unknown member read, and call expression receive no derived
  candidate in the initial subset.
- A decorator-free route built through the installed authority bridge renders
  the derived getter in a real browser, invalidates it after a V2 action, and
  restores and invalidates it again on the resumed path.

Conditional dependency coverage and unknown-call proof remain required
evidence for a later broader candidate amendment.
