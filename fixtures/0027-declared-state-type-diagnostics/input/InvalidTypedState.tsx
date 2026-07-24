@component("x-invalid-typed-state")
class InvalidTypedState extends Component {
  count: number = state("zero");
  title: string = state(1);
  enabled: boolean = state(null);
  empty: null = state(false);
  valid: number = state(0);
  status: "idle" | "loading" = state(1);
  collection: string = state(["not checked"]);

  render() {
    return <p>{this.count}</p>;
  }
}
