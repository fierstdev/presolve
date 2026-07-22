# Phase M M5-A computed conformance

**Status:** M5-A complete.

## Scope

M5-A adds only the existing compiler Computed getter declaration:

```tsx
@computed()
get total() { return this.doubled + this.tripled; }
```

`@presolve/framework-types` supplies a standard TypeScript getter-decorator
declaration only. It does not evaluate a getter, derive dependencies, cache a
value, schedule invalidation, or expose manual invalidation.

## Evidence

`framework/tests/computed-types/src/ComputedDiamond.tsx` is byte-identical to
the existing production/resume Computed example. The M5 verifier proves:

1. TypeScript 7.0 resolves the existing Component, State, Action, and Computed
   spelling with no JavaScript framework package;
2. the unchanged explicit compiler check accepts the source; and
3. the existing real-browser Computed-diamond fixture, whose source is
   byte-identical to the example, recomputes compiler-generated values after an
   Action batch.

The browser fixture asserts initial values, a single compiler-planned update
run, clean caches, and no runtime diagnostics. It is compiler evidence, not a
framework scheduler test.

## Boundary

M5-A adds no Effect declaration or implementation. Its TypeScript, explicit
compiler-check, and real-browser Computed-diamond evidence passes. M5-B may
consider Effects only after selecting its exact compiler fixture and capability
evidence.
