# V2 synchronous Action-local-literal contract

This contract admits one further deterministic V2 Action body form: a
serializable local literal assigned directly to canonical State. It extends the
typed Action-parameter contract without evaluating source at runtime.

## Admitted source form

A canonical, authority-proven synchronous block handler may declare one or
more local variables only when each declaration has a parser-retained,
serializable primitive literal (`string`, `number`, `boolean`, or `null`). Each
local must occur before, and exactly once in, a direct assignment:

```ts
setLocal = action(() => {
  const next = 23;
  this.count = next;
});
```

The receiving State must have the same primitive type, established by its
annotation or serializable initial value. Every retained local must be used;
every identifier assignment must resolve exactly once to either such a local or
to a parameter admitted by the typed Action-parameter contract.

## Projection

The parser retains the declaration's serializable value and source span. V2
lowering proves ordering, ownership, and type compatibility, then projects a
normal literal `assign` action operation. The local identifier, declaration,
and handler body never enter the runtime artifact. No runtime source evaluator
or synthetic method is introduced.

## Exclusions and proof

Computed local initializers, objects, arrays, untyped or unused locals,
reassignment, local-dependent arithmetic, calls, branches, loops, captures,
and async handling remain outside this contract. The decorator-free browser
fixture proves a canonical Action updates State from a local literal after the
ordinary and typed-parameter paths, without runtime diagnostics.
