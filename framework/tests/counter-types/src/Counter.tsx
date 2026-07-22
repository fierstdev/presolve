@component("x-counter")
class Counter extends Component {
  count = state(0);

  @action()
  increment() {
    this.count += 1;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
