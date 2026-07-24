@component("x-computed-serialization")
class ComputedSerialization extends Component {
  @computed()
  get snapshot() { return { version: "v1", ready: true }; }

  render() {
    return <output>Serializable</output>;
  }
}
