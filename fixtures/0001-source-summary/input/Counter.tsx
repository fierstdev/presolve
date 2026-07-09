@route("/counter")
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count += 1;
  }

  render() {
    return (
      <button onClick={() => this.increment()}>
        Count: {this.count}
      </button>
    );
  }
}
