import {
  action,
  component,
  computed,
  form,
  serialize,
  state,
  Component,
  slot,
  type Form,
  type SlotContent,
} from "presolve";

@component()
export class PublicCounter extends Component {
  @slot() children!: SlotContent;
  @form() @serialize("json") profile!: Form;
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
