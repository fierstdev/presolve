@route("/decrement")
@component("x-decrement-counter")
class DecrementCounter extends Component {
  count = state(2);

  decrement() {
    this.count--;
  }

  render() {
    return (
      <button onClick={() => this.decrement()}>
        Count: {this.count}
      </button>
    );
  }
}
