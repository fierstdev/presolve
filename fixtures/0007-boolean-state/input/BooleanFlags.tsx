@route("/boolean-flags")
@component("x-boolean-flags")
class BooleanFlags extends Component {
  enabled = state(true);
  disabled = state(false);

  render() {
    return (
      <section>
        <p>Enabled:{this.enabled}</p>
        <p>Disabled:{this.disabled}</p>
      </section>
    );
  }
}
