@component("x-local-constants")
class LocalConstants extends Component {
  render() {
    const title = "EdgeZero";
    const enabled = true;
    return <output>{this.count}</output>;
  }

  count = state(1);
}
