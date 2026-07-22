# Phase N N2-A template interpolation contract

## Admitted source form

N2-A admits untagged template literals in a supported `@computed()` getter:

```ts
@computed()
get label() {
  return `Count: ${this.count}!`
}
```

Every interpolation expression must already belong to the compiler-supported
Computed expression subset. Tagged templates, invalid cooked escapes, dynamic
template factories, template literals in Actions or Effects, and arbitrary calls
remain rejected or outside this admission.

## Compiler ownership

The parser retains cooked literal segments and interpolation expressions. The
compiler constructs one `Template` expression node, derives dependencies from
each interpolation, assigns the result `string` type, lowers a canonical
`Template` IR instruction, and emits a `template` runtime-computed instruction.
The generated runtime validates segment arity and performs the interpolation
from compiler-produced values; it never evaluates authored JavaScript source.

The result is serializable as a string and inherits the reactive dependencies
of its interpolation expressions. It has no independent State, capability,
activation, package, or resume record.

## Artifact compatibility

N2-A increments the runtime-computed artifact schema to version `5`. Older
artifacts fail closed through their existing schema validation. The browser
proof verifies that `` `Count: ${this.count}!` `` evaluates to `Count: 2!` from
the generated program with no runtime diagnostic.
