# Phase N N3-C Action parameter State-type contract

N3-C makes the N3-B parameter-to-State replacement path type-safe at the
compiler boundary. A parameter assigned to a complete State field through an
`@action()` method must have the same compiler-known primitive kind as that
field. The field kind is its exact State annotation when present, otherwise it
is inferred from a primitive `state(...)` initializer.

```tsx
count = state(0);

@action() setCount(value: number) {
  this.count = value;
}
```

The following is rejected with `PSC1044` because `value` is `string` while
`count` is compiler-known `number`:

```tsx
@action() setCount(value: string) {
  this.count = value;
}
```

The accepted kinds are exactly `string`, `number`, `boolean`, and `null`. A
record/array State field, union, alias, generic, structural object type, or
unknown initializer has no primitive parameter compatibility proof and remains
outside this slice. N3-C does not add runtime behavior, a new artifact field,
or a TypeScript checker authority: it validates the already compiler-lowered
N3-B operation before artifacts are emitted.
