# Framework API

Import application authoring primitives from `presolve`.

```ts
import {
  action, effect, slot, state, Component, type SlotContent,
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
| `environment.public(name)` | Manifest-backed read of an admitted `PRESOLVE_PUBLIC_*` value. |

## Legacy compatibility declarations

`component()`, the zero-argument `action()`/`effect()` overloads, `computed()`,
`context()`, `provide()`, `consume()`, `form()`, `serialize()`, `field()`,
`validate()`, `submit()`, `resource()`, `loader()`, `serverAction()`, and
`opaque()` remain supported only as explicitly labelled alpha-compatibility
declarations. They are not the source form emitted by `create-presolve`, and a
decorator-free replacement for each one requires its own compiler contract.

See [components](../guide/components.md), [reactivity](../guide/reactivity.md),
[composition](../guide/composition.md), and
[forms/resources](../guide/forms-and-resources.md) for their exact admitted
boundaries.
