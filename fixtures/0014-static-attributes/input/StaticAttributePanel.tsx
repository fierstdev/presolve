@route("/static-attributes")
@component("x-static-attribute-panel")
class StaticAttributePanel extends Component {
  label = state("Ready");

  render() {
    return (
      <section id="panel-root" aria-label='Status "Panel"' hidden>
        <button type="button" data-mode="safe & sound" title="Use <carefully>">
          Label: {this.label}
        </button>
      </section>
    );
  }
}
