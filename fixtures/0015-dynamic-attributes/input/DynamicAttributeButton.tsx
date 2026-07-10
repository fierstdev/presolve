@route("/dynamic-attributes")
@component("x-dynamic-attribute-button")
class DynamicAttributeButton extends Component {
  disabled = state(false);
  label = state("Ready");

  lock() {
    this.disabled = !this.disabled;
    this.label = "Locked";
  }

  render() {
    return (
      <button disabled={this.disabled} title={this.label} onClick={() => this.lock()}>
        Status: {this.label}
      </button>
    );
  }
}
