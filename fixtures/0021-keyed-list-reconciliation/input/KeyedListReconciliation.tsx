@route("/keyed-list-reconciliation")
@component("x-keyed-list-reconciliation")
class KeyedListReconciliation extends Component {
  labels = state(["North", "South"]);

  reconcile() {
    this.labels = ["South", "East", "North"];
  }

  trim() {
    this.labels = ["East"];
  }

  render() {
    return (
      <section>
        <button onClick={() => this.reconcile()}>Reconcile</button>
        <button onClick={() => this.trim()}>Trim</button>
        <ol>
          {this.labels.map((label, index) => <li key={label}>{index}: {label}</li>)}
        </ol>
      </section>
    );
  }
}
