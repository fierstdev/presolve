@route("/aria-validity")
@component("x-aria-validity-button")
class AriaValidityButton extends Component {
  invalid = state(false);

  toggle() {
    this.invalid = !this.invalid;
  }

  render() {
    return <button aria-invalid={this.invalid} onClick={() => this.toggle()}>Validate</button>;
  }
}
