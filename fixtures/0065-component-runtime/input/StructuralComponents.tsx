@component("x-structural-leaf") class StructuralLeaf extends Component {
  render() { return <strong>Leaf</strong>; }
}
@component("x-structural-branch") class StructuralBranch extends Component {
  render() { return <section><StructuralLeaf /></section>; }
}
@component("x-structural-page") @route("/structural") class StructuralPage extends Component {
  visible = state(true);
  items = state([{ id: "a" }, { id: "b" }, { id: "c" }]);
  render() {
    return <main><button onClick={() => this.toggle()}>Toggle</button><button onClick={() => this.reconcile()}>Reconcile</button><button onClick={() => this.trim()}>Trim</button>{this.visible ? <div><StructuralBranch /><StructuralLeaf /></div> : <aside>Hidden</aside>}<ul>{this.items.map(item => <li key={item.id}><StructuralLeaf /></li>)}</ul></main>;
  }
  toggle() { this.visible = false; }
  reconcile() { this.items = [{ id: "c" }, { id: "d" }, { id: "a" }]; }
  trim() { this.items = [{ id: "d" }]; }
}
