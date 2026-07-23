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

The selected package must be an explicitly imported `opaque` export in the
existing semantic-package resolution table. Its closed terminal contract binds
the import specifier to the package coordinate, resolved version, SHA-256
integrity, exported name, runtime module, `() -> void` signature, client
boundary, and `cold_fallback` resume policy. This reuses the compiler's one
integrity authority rather than creating an unverified opaque-runtime table.
It is still an explicit host input: there is no package-manager discovery,
lockfile read, or package implementation inspection.

## Initial boundary

The first opaque capability is client-only and has no application inputs or
outputs. A valid resolution is a compiler semantic product only; the later
runtime slice will dynamically import only the exact host-bound module, obtain
only the declared export, check that it is callable, and call it with no
arguments after the compiler-owned Action batch starts. Failure must be
recorded as an opaque runtime diagnostic and leave compiler-owned State, Form,
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

N9-D implements the semantic artifact projection as `RuntimeOpaqueArtifact`
schema v1. It contains resolved activation, owner, method, package coordinate,
module/export, and closed terminal contract; runtime location is populated only
by the exact existing semantic-package runtime-module table. N9-E publishes
that exact artifact as `opaque.runtime.json` and embeds it before `runtime.js`.
Generated execution and resume consumption remain subsequent slices.

N9-F consumes the embedded artifact only after strict runtime validation. A
delegated compiler event carries its exact method identity to the matching
terminal; the runtime imports only its emitted exact module location, verifies
the declared export is callable, and invokes it with no arguments. Failure is
diagnostic-only and opaque terminals force the normal resume path to cold
fallback before restoration. Browser evidence is still required for admission.

N9-G supplies the positive browser proof: missing exact runtime mapping fails
the build, while a host-bound module is imported only after the authored
compiler Action event and its declared terminal export is called. Malformed,
non-callable, and snapshot resume fallback cases remain required negative
evidence before registry admission.

Malformed, duplicate, mismatched, non-client, missing-export, non-callable,
or integrity-unbound records fail closed. An opaque declaration must match an
actual imported opaque semantic-package export; a pure/resource/nonsemantic
binding is rejected. There is no fallback to normal module resolution, an
authored function body, `eval`, dynamic import expressions, opaque render
islands, server execution, or hydration.

## Required proof

Positive and negative source fixtures, exact diagnostic evidence, stable ID and
artifact bytes, host-table integrity rejection, malformed artifact rejection,
one generated-browser module activation, cold resume fallback, and an audit
that the compiler neither parses package implementation nor exposes opaque
activation as a State/Context/Form write are required before registry admission.
