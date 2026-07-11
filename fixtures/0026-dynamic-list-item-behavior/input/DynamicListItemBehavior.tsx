@route("/dynamic-list-item-behavior")
@component("x-dynamic-list-item-behavior")
class DynamicListItemBehavior extends Component {
  items = state([
    { id: "north", label: "North", details: { region: "west" } },
    { id: "south", label: "South", details: { region: "east" } }
  ]);
  selections = state(0);

  refresh() {
    this.items = [
      { id: "north", label: "Northern", details: { region: "central" } },
      { id: "east", label: "East", details: { region: "coastal" } },
      { id: "south", label: "Southern", details: { region: "mountain" } }
    ];
  }

  select() {
    this.selections++;
  }

  render() {
    return (
      <section>
        <button onClick={() => this.refresh()}>Refresh</button>
        <p>{this.selections}</p>
        <ol>
          {this.items.map((item, index) => (
            <li key={item.id} title={item.details.region} data-label={item.label}>
              <button onClick={() => this.select()}>{index}: {item.label} ({item.details.region})</button>
            </li>
          ))}
        </ol>
      </section>
    );
  }
}
