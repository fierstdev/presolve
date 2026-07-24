# Framework API

Import application authoring primitives from `presolve`.

```ts
import {
  action, component, computed, consume, context, effect, field, form,
  loader, opaque, provide, required, resource, serialize, serverAction, slot,
  state, submit, validate, Component, type Form,
} from "presolve";
```

All decorators are compiler intrinsics with no runtime registration authority.

| Export | Use |
| --- | --- |
| `Component` | Base class for public component examples and component declarations. |
| `component()` | Class decorator that declares a compiler component. |
| `state(initialValue)` | Creates compiler-owned component state from an initial value. |
| `action()` | Method decorator for a transactional interaction boundary. |
| `computed()` | Getter decorator for a pure synchronous derived value. |
| `effect()` | Method decorator for a synchronous terminal effect. |
| `slot()` / `SlotContent` | Field decorator and type for default/named slots. |
| `context()` | Field decorator that declares a Context identity. |
| `provide("Class.field")` | Field decorator that exposes a Context value. |
| `consume("Class.field")` | Field decorator that receives a Context value. |
| `form()` | Field decorator that declares a form. |
| `Form` | Declaration-only type required on an `@form()` field. |
| `serialize(format)` | Form field decorator; formats are `json`, `form-data`, `url-encoded`. |
| `field(form, path?)` | Field decorator for a form value and optional static nested path. |
| `required()` / `validate(rule)` | Creates and attaches a validation rule. |
| `submit(form)` | Method decorator that associates an action with a form submission. |
| `resource(endpoint)` | Field decorator for a resource boundary. |
| `loader(endpoint)` | Field decorator for a loader boundary. |
| `serverAction(endpoint)` | Method decorator for a server-action boundary. |
| `opaque(packageSpecifier, exportName)` | Method decorator for a declared terminal package boundary. |

`Resource<Data, Error>` exposes readonly `data`, `error`, and `state`.
`ContextDesignator` is the string form `` `${string}.${string}` ``. Refer to
the guides for [components](../guide/components.md),
[reactivity](../guide/reactivity.md), [composition](../guide/composition.md),
and [forms/resources](../guide/forms-and-resources.md) for usage constraints.
