@route("/logical-and-status")
@component("x-logical-and-status")
class LogicalAndStatus extends Component {
  enabled = state(true);

  toggle() {
    this.enabled = !this.enabled;
  }

  render() {
    return (
      <button onClick={() => this.toggle()}>
        {this.enabled && <span>On</span>}
      </button>
    );
  }
}
