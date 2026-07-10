@route("/reset-counter")
@component("x-reset-counter")
class ResetCounter extends Component {
  count = state(5);

  reset() {
    this.count = 0;
  }

  render() {
    return (
      <button onClick={() => this.reset()}>
        Count: {this.count}
      </button>
    );
  }
}
