# Components

A Presolve component is an exported class marked with `@component()` and one
instance `render()` method. `render()` returns TSX.

```tsx
import { component, Component } from "@presolve/core";

@component()
export class Welcome extends Component {
  name = "Ada";

  render() {
    return <section><h1>Hello, {this.name}</h1></section>;
  }
}
```

The component decorator is a compiler intrinsic. It is intentionally inert at
normal JavaScript evaluation time; importing a component does not create a
runtime component registry.

## Inputs

Undecorated instance fields express component inputs. A definite-assignment
field is required; an initialized field supplies its default.

```tsx
@component()
export class UserCard extends Component {
  user!: { name: string };
  compact = false;

  render() {
    return <article>{this.user.name}</article>;
  }
}
```

Keep input writes with the owning component. State owned by the component uses
the `state()` intrinsic described in [reactivity](reactivity.md).

## Component use

Use a local or imported PascalCase class in TSX:

```tsx
@component()
export class ProfilePage extends Component {
  render() {
    return <main><UserCard /></main>;
  }
}
```

The compiler resolves invocation and instance identity. Dynamic component
expressions, inheritance, mixins, and constructors that establish component
semantics are outside the alpha surface.
