@route("/static-keyed-list")
@component("x-static-keyed-list")
class StaticKeyedList extends Component {
  labels = state(["North", "South"]);

  render() {
    return (
      <ol>
        {this.labels.map((label, index) => <li key={label}>{index}: {label}</li>)}
      </ol>
    );
  }
}
