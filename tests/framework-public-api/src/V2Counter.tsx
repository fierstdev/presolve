import { action, effect, state, Component } from "presolve";

export abstract class V2CounterBase extends Component {}

export class V2Counter extends V2CounterBase {
  count = state(0);

  increment = action(() => {
    this.count += 1;
  });

  syncTitle = effect(() => {
    document.title = String(this.count);
  });

  get label(): number {
    return this.count;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.label}</button>;
  }
}
