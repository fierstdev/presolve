@route("/broken")
@component("x-broken-counter")
class BrokenCounter extends Component {
  count = state(0);

  render() {
    return (
      <button onClick={() => this.increment()}
        Count: {this.count}
      </button>
    );
  }
}
