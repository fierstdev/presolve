@route("/null-selection")
@component("x-null-selection")
class NullSelection extends Component {
  selection = state(null);

  render() {
    return <p>Selection:{this.selection}</p>;
  }
}
