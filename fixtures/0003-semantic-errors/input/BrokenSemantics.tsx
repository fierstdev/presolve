@route("/semantic-errors")
class BrokenSemantics extends Component {
  count = state(0);

  render() {
    return (
      <button onClick={() => this.increment()}>
        Count: {this.missingCount}
      </button>
    );
  }
}
