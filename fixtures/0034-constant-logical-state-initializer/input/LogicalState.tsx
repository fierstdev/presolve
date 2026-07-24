@component("x-logical-state")
class LogicalState extends Component {
  ready: boolean = state((1 < 2) && (3 >= 3));
  fallback: boolean = state(false || (10 !== 4));

  render() {
    return <output>{this.ready}</output>;
  }
}
