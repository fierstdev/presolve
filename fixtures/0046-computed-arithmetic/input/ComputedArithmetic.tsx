@component("x-computed-arithmetic")
class ComputedArithmetic extends Component {
  count = state(2);
  offset = state(3);

  @computed()
  get total() { return this.count * 4 + this.offset - 1; }

  render() {
    return <output>Arithmetic</output>;
  }
}
