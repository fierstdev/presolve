# Phase M M5-B effect conformance

**Status:** M5-B complete.

## Scope

M5-B adds only the existing compiler Effect method declaration:

```tsx
@effect()
report() { document.title = this.title; }
```

The type package supplies a method-decorator declaration only. It does not run
an initial callback, derive dependencies, schedule batches, perform capability
access, offer cleanup, or create a hook runtime.

## Evidence

The framework fixture is byte-identical to the compiler's
`InitialEffectRuntime` fixture. Its verifier proves TypeScript 7.0 resolution,
unchanged compiler-check success, and the existing real-browser Effect proof.
That proof verifies a single initial run, exact compiler capability-dispatch
order, computed input visibility, document/local-storage updates, and no
runtime diagnostics.

## Boundary

M5-B does not add cleanup returns, arbitrary capability calls, state mutation,
manual dependencies, action/effect calls, or a generic `useEffect` analogue.
Its TypeScript, explicit compiler-check, and real-browser evidence passes. M5
is complete; later composition work must still select existing compiler forms
one family at a time.
