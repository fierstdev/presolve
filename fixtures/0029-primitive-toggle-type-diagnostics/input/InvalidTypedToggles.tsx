@component("x-invalid-typed-toggles")
class InvalidTypedToggles extends Component {
  count: number = state(0);
  title: string = state("Ready");
  enabled: boolean = state(false);
  empty: null = state(null);
  status: "idle" | "loading" = state("idle");

  apply() {
    this.count = !this.count;
    this.title = !this.title;
    this.empty = !this.empty;
    this.enabled = !this.enabled;
    this.status = !this.status;
  }

  render() {
    return <p>{this.count}</p>;
  }
}
