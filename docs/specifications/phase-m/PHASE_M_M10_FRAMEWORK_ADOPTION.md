# Phase M M10 framework adoption

**Status:** M10-A Resource and M10-B JSX/capability classification conformance
are complete. Remaining M10 work is aggregate freeze evidence only.

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

## M10-B — capability disposition and JSX conformance

The table below is a framework projection of the frozen compiler registry; it
does not authorize a source form absent from `presolve explain --capabilities`.

| Registry capability | Framework disposition |
| --- | --- |
| `component`, `component_invocation`, `state`, `serializable_state_replacement`, `static_action_parameters`, `action_parameter_state_types`, `serializable_action_locals`, `structured_serializable_action_locals`, `action`, `computed`, `effect`, `context`, `slot`, `keyed_structural_list`, `form` | Existing framework spelling; the compiler continues to validate the bounded placement/body/template rules. |
| `module_bindings`, `template_interpolation`, `static_index_access`, `boolean_computed_conditional`, `builtin_math_abs`, `builtin_math_min_max`, `builtin_math_rounding` | Existing TypeScript syntax in an existing component/Computed context; no decorator or framework helper is added. |
| `jsx_html_attribute_aliases`, `typed_aria_bindings`, `keyboard_action_event` | M10-B declaration conformance: `className`/`htmlFor`, the twelve admitted ARIA attributes, and `onKeydown` now have exact TypeScript attribute types. `onClick` uses the same zero-argument Action event type. |
| `semantic_package_bindings`, `semantic_package_pure_identity` | Compiler/build input only. Application or package authors supply ordinary package typings; callers supply the exact semantic-package contract. The framework performs neither resolution nor package execution. |
| `resources` | M10-A declaration conformance: `@resource(...)` and `Resource<Data, Error>`. Exact package contract and runtime-module mapping remain compiler build inputs. |
| `opaque_typescript` | Existing N10 declaration conformance: `@opaque(packageSpecifier, exportName)` only on the compiler-admitted terminal Action form. |
| `advanced_types`, `semantic_package_exports` | Intentionally unavailable. The framework supplies no compatibility spelling, arbitrary package-call type, fallback runtime, or escape hatch. |

`PresolveIntrinsicAttributes` deliberately leaves unknown intrinsic attributes
as `unknown`. That does not admit them: it prevents this declaration package
from becoming a second DOM language while canonical compiler diagnostics retain
the sole approval/rejection authority. Known M10-B attributes catch their exact
string/boolean mismatch early; compiler checks still decide whether the element,
binding expression, event target, and Action method satisfy the semantic form.

`scripts/verify-m10b-capability-conformance.sh` checks that each registry
capability has exactly this documented framework disposition, TypeScript 7.0
accepts the exact aliases/ARIA/keyboard source, and the unchanged compiler
check succeeds. It adds no JSX transform, DOM renderer, event dispatcher, or
framework validation pass.
