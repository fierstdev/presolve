@component("x-effect-fixture-matrix")
class EffectFixtureMatrix extends Component {
  count = state(1);
  title = state("Presolve effects");

  @computed()
  get total() { return this.count * 2; }

  @action()
  incrementTwice() {
    this.count += 1;
    this.count += 1;
  }

  @action()
  rename() { this.title = "Presolve renamed"; }

  @effect()
  syncTitle() { document.title = this.title; }

  @effect()
  persistTotal() {
    console.log(this.total);
    localStorage.setItem("presolve-total", "ready");
  }

  @effect()
  rememberTotal() {
    console.info(this.total);
    sessionStorage.setItem("presolve-total", "ready");
  }

  @effect()
  bootstrap() { console.warn("effect fixture ready"); }

  render() {
    return <div><button onClick={() => this.incrementTwice()}>{this.count}</button><button onClick={() => this.rename()}>{this.title}</button></div>;
  }
}
