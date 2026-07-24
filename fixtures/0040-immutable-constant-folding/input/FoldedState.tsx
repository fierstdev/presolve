@component("x-folded-state")
class FoldedState extends Component {
  total: number = state((1 + 2) * 3);

  render() {
    return <output>{this.total}</output>;
  }
}
