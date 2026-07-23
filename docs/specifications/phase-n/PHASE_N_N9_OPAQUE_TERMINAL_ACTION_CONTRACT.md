# Phase N N9 opaque terminal Action contract

**Status:** implementation authority.

N9 admits arbitrary third-party JavaScript only at an explicit compiler-recorded
terminal boundary. It does not make arbitrary TypeScript compiler-native.

## Source form

```tsx
import { trackPurchase } from "@acme/analytics";

@component("checkout")
class Checkout {
  @action()
  @opaque("@acme/analytics", "trackPurchase")
  track(): void {}

  render() { return <button onClick={this.track}>Buy</button>; }
}
```

`@opaque(packageSpecifier, exportName)` is legal only on an otherwise valid,
zero-parameter `@action()` method with an empty body. It replaces the method
body with one compiler-recorded terminal Action activation. The method remains
the ordinary compiler-owned event target and Action batch boundary; the opaque
export is never parsed, analyzed, inlined, or treated as a component callback.

The selected package must be supplied through a new exact opaque-runtime table:
package specifier, resolved version, SHA-256 integrity, runtime module location,
and exported name. The compiler treats those values as an explicit host input,
not package-manager discovery or a semantic package contract.

## Initial boundary

The first opaque capability is client-only and has no application inputs or
outputs. Generated runtime dynamically imports only the exact host-bound
module, obtains only the declared export, checks that it is callable, and calls
it with no arguments after the compiler-owned Action batch starts. Failure is
recorded as an opaque runtime diagnostic and leaves compiler-owned State, Form,
Context, Component, and Resource storage untouched.

It may not appear on State, Computed, Effect, Form, Resource, Context, Slot, or
render members. It may not have parameters, a body, return a value, read event
payloads, contribute render dependencies, access compiler-owned state, or claim
resume behavior. Resume deterministically falls back cold when an opaque
activation is present; an opaque-resume codec is a later contract.

## Artifacts and inspection

The compiler issues an `OpaqueActivationId` from the component Action identity,
then emits an opaque runtime artifact containing only that ID, Action batch,
package/version/integrity/export/runtime location, client boundary, no-input
signature, and `resume: "cold-fallback"`. Template/event artifacts reference
the existing Action batch, not arbitrary source code. Inspection and every
runtime diagnostic label the activation `opaque`.

Malformed, duplicate, mismatched, non-client, missing-export, non-callable,
or integrity-unbound records fail closed. There is no fallback to normal module
resolution, an authored function body, `eval`, dynamic import expressions,
opaque render islands, server execution, or hydration.

## Required proof

Positive and negative source fixtures, exact diagnostic evidence, stable ID and
artifact bytes, host-table integrity rejection, malformed artifact rejection,
one generated-browser module activation, cold resume fallback, and an audit
that the compiler neither parses package implementation nor exposes opaque
activation as a State/Context/Form write are required before registry admission.
