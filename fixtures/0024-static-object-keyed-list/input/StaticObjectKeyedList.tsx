@route("/static-object-keyed-list")
@component("x-static-object-keyed-list")
class StaticObjectKeyedList extends Component {
  items = state([
    { id: "north", label: "North", details: { region: "west" } },
    { id: "south", label: "South", details: { region: "east" } }
  ]);

  render() {
    return (
      <ol>
        {this.items.map((item, index) => <li key={item.id}>{index}: {item.label} ({item.details.region})</li>)}
      </ol>
    );
  }
}
