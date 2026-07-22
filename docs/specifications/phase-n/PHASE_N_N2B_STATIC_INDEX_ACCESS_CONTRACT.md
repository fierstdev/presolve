# Phase N N2-B static index access contract

## Admitted source form

N2-B admits a bracket read only in a supported `@computed()` getter when the
index is a string literal or a non-negative integer literal:

```ts
@computed()
get selected() {
  return this.labels[1]
}
```

The target must be a compiler-supported tuple, array, or object value. Dynamic
keys, variables, negative or fractional numeric keys, optional indexing,
prototype lookup, bracket writes, Actions, and Effects remain outside this
admission.

## Compiler ownership

The parser retains the object and literal index as an `IndexAccess` expression.
The expression graph derives the reactive dependency from the object and the
IR emits `GetIndex`; the compiler never delegates dependency discovery to
JavaScript. Tuple access carries the selected element type when the literal is
in bounds, arrays carry their element type, and object reads carry the
corresponding literal property type. Unknown or unsupported targets remain
unknown rather than being interpreted dynamically.

The generated runtime consumes the compiler artifact only. It accepts a string
or non-negative integer key, performs an own-property read, and otherwise
produces `undefined`; it does not execute authored source or traverse a
prototype chain.

## Artifact compatibility

N2-B increments `computed.runtime.json` to schema version `6` and adds the
`get-index` instruction. The generated runtime requires exactly schema `6` and
fails closed for other versions. The browser proof verifies that a tuple State
value indexed by `1` produces `"one"` without runtime diagnostics.
