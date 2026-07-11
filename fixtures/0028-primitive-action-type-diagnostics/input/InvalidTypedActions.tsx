@component("x-invalid-typed-actions")
class InvalidTypedActions extends Component {
  count: number = state(0);
  title: string = state("Ready");
  enabled: boolean = state(false);
  empty: null = state(null);
  status: "idle" | "loading" = state("idle");
  collection: string = state(["not checked"]);

  apply() {
    this.count = "zero";
    this.title = 1;
    this.enabled = null;
    this.empty = false;
    this.status = 1;
    this.collection = ["not checked"];
    this.count += 1;
    this.enabled = !this.enabled;
    this.count = 1;
  }

  render() {
    return <p>{this.count}</p>;
  }
}
