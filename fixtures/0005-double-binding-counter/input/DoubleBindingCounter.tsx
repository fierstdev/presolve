@route("/double-binding")
@component("x-double-binding-counter")
class DoubleBindingCounter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  incrementAgain() {
    this.count++;
  }

  render() {
    return (
      <section>
        <button onClick={() => this.increment()}>
          <span>Count: {this.count}</span>
        </button>
        <button onClick={() => this.incrementAgain()}>
          Mirror: {this.count}
        </button>
      </section>
    );
  }
}
