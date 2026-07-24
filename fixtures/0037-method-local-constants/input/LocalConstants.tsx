@component("x-local-constants")
class LocalConstants extends Component {
  render() {
    const title = "Presolve";
    const enabled = true;
    return <output>{this.count}</output>;
  }

  count = state(1);
}
