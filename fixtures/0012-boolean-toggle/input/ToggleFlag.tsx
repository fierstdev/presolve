@route("/toggle-flag")
@component("x-toggle-flag")
class ToggleFlag extends Component {
  enabled = state(false);

  toggle() {
    this.enabled = !this.enabled;
  }

  render() {
    return (
      <button onClick={() => this.toggle()}>
        Enabled: {this.enabled}
      </button>
    );
  }
}
