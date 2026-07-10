@route("/double-binding")
@component("x-double-binding-counter")
class DoubleBindingCounter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return (
      <section>
        <button onClick={() => this.increment()}>
          Count: {this.count}
          Mirror: {this.count}
        </button>
      </section>
    );
  }
}
