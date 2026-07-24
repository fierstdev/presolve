# Framework authoring

The `presolve` package is an authoring vocabulary, not a runtime framework.
Decorator calls are inert in ordinary JavaScript execution. A successful
Presolve compilation assigns their meaning, storage, scheduling, and generated
runtime behavior.

```tsx
import { action, component, state, Component } from "presolve";

@component()
export class Counter extends Component {
  count = state(0);

  @action()
  increment() {
    this.count++;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
```

`@component()` and explicit capability boundaries are intentionally visible.
Everything the compiler can derive losslessly—component identity, input fields,
render structure, dependencies, batching, cache policy, routes, and artifact
inclusion—is inferred.

Most static components use only `@component()`. Stateful components generally
add `state()` and `@action()`. Forms, Context/slots, resources, server
boundaries, and opaque package contracts remain explicit because they express
ownership or capability meaning that cannot be inferred safely.
