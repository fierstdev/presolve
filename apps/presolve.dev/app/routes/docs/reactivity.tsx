import { component, Component } from "presolve";

@component()
export class Reactivity extends Component {
  render() {
    return <article><h1>State and actions</h1><p>Use state() for component-owned values and action() for interaction boundaries. Computed getters and effects are compiler-derived; they do not require dependency arrays, signal wrappers, or a framework scheduler.</p><pre>{"count = state(0)\n@action() increment() { this.count += 1; }"}</pre></article>;
  }
}
