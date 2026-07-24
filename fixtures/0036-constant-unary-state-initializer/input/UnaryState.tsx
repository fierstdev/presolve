@component("x-unary-state")
class UnaryState extends Component {
  negated: boolean = state(!(1 < 2));
  signed: number = state(-(1 + 2));

  render() {
    return <output>{this.signed}</output>;
  }
}
