import { action, component, state, Component } from "presolve";

@component()
export class Counter extends Component {
  count = state(0);

  @action()
  increment(): void {
    this.count += 1;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
