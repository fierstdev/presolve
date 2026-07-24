@component("x-method-parameters")
class MethodParameters extends Component {
  save(title: string, retries?: number) {
  }

  render() {
    return <output>{this.count}</output>;
  }

  count = state(1);
}
