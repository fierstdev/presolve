@component("x-typed-state")
class TypedState extends Component {
  count: number = state(0);
  status: "idle" | "loading" = state("idle");

  render() {
    return <p>{this.count}</p>;
  }
}
