@component("x-comparison-state")
class ComparisonState extends Component {
  ready: boolean = state(((1 + 2) * 3) >= 9);
  different: boolean = state(10 !== 4);

  render() {
    return <output>{this.ready}</output>;
  }
}
