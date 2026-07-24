@component("x-runtime-computed")
class RuntimeComputed extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get label() { return this.doubled + 1; }

  render() {
    return <output>Computed</output>;
  }
}
