@route("/step-counter")
@component("x-step-counter")
class StepCounter extends Component {
  count = state(4);

  addTwo() {
    this.count += 2;
  }

  subtractThree() {
    this.count -= 3;
  }

  render() {
    return (
      <section>
        <button onClick={() => this.addTwo()}>
          Add: {this.count}
        </button>
        <button onClick={() => this.subtractThree()}>
          Subtract: {this.count}
        </button>
      </section>
    );
  }
}
