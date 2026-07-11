@route("/list-key-diagnostics")
@component("x-list-key-diagnostics")
class ListKeyDiagnostics extends Component {
  labels = state(["North", "North"]);

  render() {
    return (
      <section>
        <ol>{this.labels.map((label) => <li>{label}</li>)}</ol>
        <ol>{this.labels.map((label, index) => <li key={index}>{label}</li>)}</ol>
        <ol>{this.labels.map((label) => <li key={label.length}>{label}</li>)}</ol>
        <ol>{this.labels.map((label) => <li key={label}>{label}</li>)}</ol>
      </section>
    );
  }
}
