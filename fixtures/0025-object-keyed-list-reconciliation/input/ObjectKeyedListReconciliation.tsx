@route("/object-keyed-list-reconciliation")
@component("x-object-keyed-list-reconciliation")
class ObjectKeyedListReconciliation extends Component {
  items = state([
    { id: "north", label: "North", details: { region: "west" } },
    { id: "south", label: "South", details: { region: "east" } }
  ]);

  reconcile() {
    this.items = [
      { id: "south", label: "South", details: { region: "east" } },
      { id: "east", label: "East", details: { region: "central" } },
      { id: "north", label: "North", details: { region: "west" } }
    ];
  }

  trim() {
    this.items = [{ id: "east", label: "East", details: { region: "central" } }];
  }

  render() {
    return (
      <section>
        <button onClick={() => this.reconcile()}>Reconcile</button>
        <button onClick={() => this.trim()}>Trim</button>
        <ol>
          {this.items.map((item, index) => <li key={item.id}>{index}: {item.label} ({item.details.region})</li>)}
        </ol>
      </section>
    );
  }
}
