# Presolve

`@presolve/framework` is the public TypeScript authoring vocabulary for
Presolve applications. Generated projects install it under the local
`presolve` alias, preserving the canonical authoring import while using a
registry-safe scoped package identity. It supplies inert compiler intrinsics
and TypeScript declarations; the Presolve compiler is the sole authority that
gives those forms semantic meaning.

```tsx
import { action, state, Component } from "presolve";

export class Counter extends Component {
  count = state(0);

  increment = action(() => { this.count++; });

  render() { return <button onClick={this.increment}>{this.count}</button>; }
}
```

Install `@presolve/cli` as a development dependency to build an application.
