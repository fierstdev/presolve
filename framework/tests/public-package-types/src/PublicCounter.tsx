import {
  action,
  component,
  computed,
  state,
  Component,
  slot,
  type SlotContent,
} from "presolve";

@component()
export class PublicCounter extends Component {
  @slot() children!: SlotContent;
  count = state(0);

  @computed()
  get label(): number {
    return this.count;
  }

  @action()
  increment(): void {
    this.count += 1;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.label}</button>;
  }
}
