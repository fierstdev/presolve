@route("/keyed-list")
@component("x-keyed-list")
class KeyedList extends Component {
  items = state([]);

  render() {
    return (
      <ul>
        {this.items.map((item, index) => <li key={item.id}>{index}: {item.label}</li>)}
      </ul>
    );
  }
}
