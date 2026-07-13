@component("x-alpha-computed")
class AlphaComputed extends Component {
  count = state(2);

  @computed()
  get doubled() { return this.count * 2; }

  render() {
    return <output>Alpha</output>;
  }
}
