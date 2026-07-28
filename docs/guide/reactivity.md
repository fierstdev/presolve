# Reactivity

Presolve uses ordinary property access. There are no signals, `.value` reads,
runtime proxies, or authored dependency arrays.

## State and actions

Initialize component-owned state with `state()`. Declare an admitted write as
an `action(handler)` instance field.

```tsx
import { action, state, Component } from "presolve";

export class Counter extends Component {
  count = state(0);

  increment = action(() => {
    this.count += 1;
  });

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
```

State reads are synchronous. Writes made by an action are visible to later
statements in that action. The compiler batches the resulting derived and DOM
work when the outer action completes; nested action calls do not create an
independent flush.

Pass an action directly to an event, or use a compiler-admitted closure for a
captured value:

```tsx
<button onClick={() => this.remove(todo.id)}>Remove</button>
```

Do not write reactive state outside an action except during permitted field
initialization. Async action behavior is not part of the beta contract.

## Computed values

Use a synchronous, pure getter. The compiler derives its dependencies and owns
caching and invalidation; no `computed()` declaration is used in new beta
source.

```tsx
get remaining(): number {
  return this.todos.filter((todo) => !todo.done).length;
}
```

Computed getters may not write state, invoke actions, or use arbitrary
capabilities.

## Effects

Use `effect(handler)` for a synchronous terminal browser capability operation.
An effect runs after initial render and after an affected completed action
batch.

```tsx
import { effect } from "presolve";

updateTitle = effect(() => {
  document.title = `${this.remaining} remaining`;
});
```

Effects cannot mutate reactive state or call actions or other effects. A
compiler-admitted cleanup function is ordered by the component lifecycle. The
compiler decides scheduling from the dependencies it can prove.
