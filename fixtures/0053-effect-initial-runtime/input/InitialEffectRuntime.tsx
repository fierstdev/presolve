@component("x-effect-initial-runtime")
class InitialEffectRuntime extends Component {
  count = state(2);
  title = state("EdgeZero initial effect");

  @computed()
  get doubled() { return this.count * 2; }

  @effect()
  report() {
    console.log(this.doubled);
    document.title = this.title;
    localStorage.setItem("edgezero-effect-initial", "ready");
  }

  render() { return <p>{this.count}</p>; }
}
