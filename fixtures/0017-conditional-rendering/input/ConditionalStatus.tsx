@route("/conditional-status")
@component("x-conditional-status")
class ConditionalStatus extends Component {
  enabled = state(true);

  toggle() {
    this.enabled = !this.enabled;
  }

  render() {
    return (
      <button onClick={() => this.toggle()}>
        {this.enabled ? <span>On</span> : <span>Off</span>}
      </button>
    );
  }
}
