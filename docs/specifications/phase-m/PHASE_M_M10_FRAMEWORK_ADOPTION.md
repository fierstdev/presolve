# Phase M M10 framework adoption

**Status:** M10-A Resource type conformance is complete. Remaining M10
classification work may expose only already-admitted Phase N forms.

## Authority and boundary

M10 is a narrow amendment to the M9 framework freeze after the Phase N N10
freeze. It consumes the compiler's schema-v1 semantic capability registry and
does not change compiler parsing, lowering, diagnostics, artifacts, runtime,
resume protocol, package resolution, or execution.

For every Phase N capability, M10 assigns exactly one framework disposition:

1. existing framework spelling, now admitted by the compiler;
2. declaration/type conformance required;
3. compiler/build input only; or
4. intentionally unavailable source.

The framework remains declaration-only. TypeScript types may describe an
admitted source shape, but the compiler remains the authority for placement,
dependency derivation, package integrity, artifact generation, scheduling, and
runtime behavior.

## M10-A — Resource type conformance

The Resource source form was compiler-admitted in N6-C13/C14 but was absent
from the frozen M9 declaration package. M10-A adds only the exact existing
syntax:

```tsx
import { loadProfile } from "profile-service";

@component("x-profile")
class Profile extends Component {
  @resource("loadProfile") profile!: Resource<string, string>;

  @computed()
  get profileName(): string | null {
    return this.profile.data;
  }
}
```

`resource(endpointDesignator)` is a declaration-only field decorator.
`Resource<Data, Error>` exposes only readonly `data`, `error`, and `state`
projections, with the compiler's exact lifecycle union. It is not a framework
Promise, fetch API, signal, store, subscription, cache, cancellation handle,
or retry interface.

The compiler accepts the field only when its string designator resolves to an
exact imported semantic-package `resource` contract. Browser publication still
requires the explicit canonical package contract and runtime-module mapping.
The framework neither reads nor supplies either input.

N6-C14 currently permits direct Resource projections only in a same-owner
Computed getter. The TypeScript declaration intentionally cannot turn every
type-correct property access into a compiler-admitted use; compiler diagnostics
remain authoritative for those placement restrictions.

## Evidence

`scripts/verify-m10a-resource-conformance.sh` proves the declaration-only
boundary, TypeScript 7.0 resolution, exact Resource source through the
canonical `presolve build` path with caller-supplied package inputs, and the
existing compiler-owned generated-browser activation/read evidence. It does
not introduce a framework Resource runtime.

## Explicit non-goals

M10-A does not add Resource inputs, generic fetch or Promise support, retry,
invalidation APIs, component-destruction hooks, snapshot codecs, resume reads,
resource Context/Form/Effect access, package discovery, runtime mapping
discovery, or an opaque fallback. Those remain available only when a future
compiler contract admits them.
