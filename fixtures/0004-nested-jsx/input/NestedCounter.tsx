@route("/nested")
@component("x-nested-counter")
class NestedCounter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return (
      <section>
        <button onClick={() => this.increment()}>Count: {this.count}</button>
      </section>
    );
  }
}
