@component("x-typed-state")
class TypedState extends Component {
  count: number = state(0);
  status: "idle" | "loading" = state("idle");
  title: string = state("Typed state");
  enabled: boolean = state(true);
  empty: null = state(null);

  render() {
    return <p>{this.count}</p>;
  }
}
