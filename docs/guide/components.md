# Components

A Presolve beta component is an exported class that extends `Component` and
has one instance `render()` method. `render()` returns TSX.

```tsx
import { Component } from "presolve";

export class Welcome extends Component {
  name = "Ada";

  render() {
    return <section><h1>Hello, {this.name}</h1></section>;
  }
}
```

Extending `Component` is compiler-owned authoring evidence; importing a
component does not create a runtime component registry.

## Inputs

Undecorated instance fields express component inputs. A definite-assignment
field is required; an initialized field supplies its default.

```tsx
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
export class ProfilePage extends Component {
  render() {
    return <main><UserCard /></main>;
  }
}
```

The compiler resolves invocation and instance identity. Dynamic component
expressions, mixins, and constructors that establish component semantics are
outside the beta surface.
