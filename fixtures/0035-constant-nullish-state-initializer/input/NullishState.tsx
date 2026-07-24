@component("x-nullish-state")
class NullishState extends Component {
  label: string = state(null ?? "fallback");
  total: number = state(5 ?? (10 / 0));

  render() {
    return <output>{this.label}</output>;
  }
}
