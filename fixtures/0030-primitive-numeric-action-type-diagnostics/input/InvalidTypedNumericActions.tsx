@component("x-invalid-typed-numeric-actions")
class InvalidTypedNumericActions extends Component {
  count: number = state(0);
  title: string = state("Ready");
  enabled: boolean = state(false);
  empty: null = state(null);
  status: "idle" | "loading" = state("idle");

  apply() {
    this.title++;
    this.enabled--;
    this.empty++;
    this.count--;
    this.status++;
  }

  render() {
    return <p>{this.count}</p>;
  }
}
