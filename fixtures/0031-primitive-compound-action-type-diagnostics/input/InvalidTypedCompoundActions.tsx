@component("x-invalid-typed-compound-actions")
class InvalidTypedCompoundActions extends Component {
  count: number = state(0);
  title: string = state("Ready");
  enabled: boolean = state(false);
  status: "idle" | "loading" = state("idle");

  apply() {
    this.title += 1;
    this.count -= true;
    this.enabled += null;
    this.count += ["not checked"];
    this.count -= 2;
    this.status += "ignored";
  }

  render() {
    return <p>{this.count}</p>;
  }
}
