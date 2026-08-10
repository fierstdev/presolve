# Framework API

Import application authoring primitives from `presolve`.

```ts
import {
  action, defineForm, effect, field, loader, resource, slot, state, Component,
  type Resource, type ResourceContext, type RouteParameters, type SlotContent,
} from "presolve";
```

The V2 beta source surface is compiler-owned. These calls have no runtime
registration authority.

| Export | Use |
| --- | --- |
| `Component` | Extend this class to declare a component. |
| `state(initialValue)` | Creates compiler-owned component state from an initial value. |
| `action(handler)` | Instance-field action; the compiler lowers its admitted synchronous State writes. |
| synchronous getter | A pure, supported getter is a compiler-derived computed value; no `computed()` call is used. |
| `effect(handler)` | Instance-field browser Effect, with compiler-owned scheduling and cleanup. |
| `slot()` / `SlotContent` | Instance-field initializer and type for default or named Slots. |
| `defineForm(definition)` / `field(options)` | Declares a typed Form tree, validation, serialization, and optional client or server submission. |
| `required()`, `min()`, `max()`, `minLength()`, `maxLength()`, `pattern()`, `email()`, `equals()`, `notEquals()` | Compiler-planned Form validation rules. |
| `resource(handler)` / `ResourceContext` | Declares a client/shared package Resource with compiler-owned cancellation, codecs, reactive updates, and resume. |
| `loader(handler)` / `RouteParameters` | Declares a route-owned Resource executed by a supported server adapter. |
| `environment.public(name)` | Manifest-backed read of an admitted `PRESOLVE_PUBLIC_*` value. |

## Historical declarations

Presolve 0.1 declarations are retained for migration analysis, not as the
current beta authoring API. This reference and the generated scaffold are the
source of truth for new applications.

See [components](../guide/components.md), [reactivity](../guide/reactivity.md),
[composition](../guide/composition.md), and
[forms/resources](../guide/forms-and-resources.md) for their exact admitted
boundaries.
