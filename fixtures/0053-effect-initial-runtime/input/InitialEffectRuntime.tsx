@component("x-effect-initial-runtime")
class InitialEffectRuntime extends Component {
  count = state(2);
  title = state("Presolve initial effect");

  @computed()
  get doubled() { return this.count * 2; }

  @action()
  update() {
    this.count += 1;
    this.title = "Presolve after action";
  }

  @effect()
  report() {
    console.log(this.doubled);
    document.title = this.title;
    localStorage.setItem("presolve-effect-initial", "ready");
  }

  render() { return <button onClick={() => this.update()}>{this.count}</button>; }
}
