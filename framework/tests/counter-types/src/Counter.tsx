@component("x-counter")
class Counter extends Component {
  count = state(0);

  render() {
    return <button>Count: {this.count}</button>;
  }
}
