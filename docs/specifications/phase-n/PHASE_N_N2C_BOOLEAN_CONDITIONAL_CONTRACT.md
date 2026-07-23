# Phase N N2-C boolean computed conditional contract

## Admitted source form

```ts
@computed()
get label() { return this.enabled ? "enabled" : "disabled" }
```

The condition must be boolean or a boolean union. Both branches must belong to
the existing Computed expression subset. JavaScript truthiness, callbacks,
statement-level control flow, and conditional expressions in Actions or Effects
remain outside this admission.

## Compiler ownership and compatibility

The compiler retains the condition and branches, derives all dependencies,
infers the normalized branch-union result, and emits canonical `Select` IR. A
non-boolean condition reports `PSC1029`. The generated runtime requires
artifact schema `7`; N2-D later advances the artifact to schema `8` while
retaining `select`. Exact `true` selects the first branch, and no authored
JavaScript source is evaluated.
