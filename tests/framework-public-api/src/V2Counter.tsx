import { action, state, Component } from "presolve";

export class V2Counter extends Component<{ initialCount?: number }> {
  count = state(0);

  increment = action(() => {
    this.count += 1;
  });

  get label(): number {
    return this.count;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.label}</button>;
  }
}
