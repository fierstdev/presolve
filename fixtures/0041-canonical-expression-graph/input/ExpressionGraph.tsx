@component("x-expression-graph")
class ExpressionGraph extends Component {
  total: number = state((1 + 2) * 3);

  render() {
    return <output>{this.total}</output>;
  }
}
