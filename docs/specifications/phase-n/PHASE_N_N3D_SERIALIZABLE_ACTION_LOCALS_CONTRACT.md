# Phase N N3-D serializable Action locals contract

N3-D admits one bounded Action-local form: a serializable literal declared in
an `@action()` method and assigned to a complete component State field.

```tsx
@action() lock() {
  const next = "Locked";
  this.label = next;
}
```

The compiler retains the local declaration, resolves its exact value during
Action lowering, and emits the existing literal `assign` State operation. No
JavaScript local is executed in the browser. The local and target State field
must have the same compiler-known primitive kind; `PSC1045` rejects an
undecorated method, a non-primitive local/State boundary, or a mismatch.

N3-D excludes computed local initializers, State reads, function calls,
closures, reassignment, destructuring, arrays/objects, aliasing, local values
in templates, and any Action control flow. Each needs an independent semantic,
dependency, type, and runtime contract.
