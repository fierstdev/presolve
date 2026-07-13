@component("x-effect-initial-runtime")
class InitialEffectRuntime extends Component {
  count = state(2);
  title = state("EdgeZero initial effect");

  @computed()
  get doubled() { return this.count * 2; }

  @action()
  update() {
    this.count += 1;
    this.title = "EdgeZero after action";
  }

  @effect()
  report() {
    console.log(this.doubled);
    document.title = this.title;
    localStorage.setItem("edgezero-effect-initial", "ready");
  }

  render() { return <button onClick={() => this.update()}>{this.count}</button>; }
}
