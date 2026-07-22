# Phase N N1-A2 pure semantic package contract

## Scope

N1-A2 admits one executable semantic-package family: a direct call to an
integrity-checked imported `pure` export in an `@computed()` getter. It is not
permission to call arbitrary package functions. The compiler lowers the
declared operation into its own expression graph, IR, runtime-computed artifact,
and browser program; package JavaScript is neither loaded nor executed.

The initial operation set contains only `identity`: exactly one supported value
argument produces that value unchanged. Although intentionally small, this is a
complete vertical slice: it proves the import-to-contract-to-expression-to-IR
to-runtime path and preserves the package coordinate as artifact provenance.

## Source and contract form

```ts
import { identity } from "value-kit"

@component("x-counter")
class Counter extends Component {
  count = state(1)

  @computed()
  get visibleCount() {
    return identity(this.count)
  }
}
```

The caller-supplied `value-kit` contract must declare the imported export as
kind `pure`, its `pure_operation` as `identity`, one argument in its type
signature, a runtime module identity, and an explicit resume policy. The
compiler validates the package binding against N1-A1, then validates arity and
the compiler-known operation. A direct identifier imported from a declared
contract is the only admitted callee form; aliases are allowed because the
binding table resolves the local identifier. Member calls, callbacks, dynamic
imports, overload inference, and all unrecognised pure operations fail closed.

The explicit build handoff supplies each contract without package discovery:

```sh
presolve build src/Counter.tsx \
  --package-contract value-kit=contracts/value-kit.json \
  --out dist
```

The flag may repeat for independently named specifiers. Its path is an exact
caller-selected JSON input; the CLI does not search for contracts, inspect an
installed package, or consult a lockfile. A missing contract stops the build
with `PSBIND1009` before artifact publication.

## Semantic guarantees

`identity` inherits the dependency and serializability behavior of its
argument. It has no State write, capability operation, resource lifecycle,
DOM ownership, scheduling boundary, or package runtime activation. The emitted
runtime instruction retains package/version/integrity/export/operation
provenance even though its evaluation is compiler-owned.

An ordinary imported function, a declared non-`pure` package export, an absent
operation declaration, or unsupported arity is a canonical compiler error. It
is never reclassified as an arbitrary pure helper or emitted as a JavaScript
call.

## Compatibility

N1-A2 changes the runtime-computed artifact schema because it adds an explicit
`pure-package-call` instruction. Existing artifacts fail closed on schema
mismatch. The contract admits no package `capability`, `resource`, `codec`, or
`component` execution; those remain separate future vertical slices.
