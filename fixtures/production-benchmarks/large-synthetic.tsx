@component("x-production-synthetic") @route("/")
class ProductionSynthetic extends Component {
  @state() a = 0;
  @state() b = 1;
  @state() c = 2;
  @computed() total = this.a + this.b + this.c;

  @action() advance(): void {
    this.a += 1;
    this.b += 2;
    this.c += 3;
  }

  render() {
    return <main>
      <button onClick={this.advance}>{this.a}</button>
      <output>{this.b}</output>
      <output>{this.c}</output>
      <strong>{this.total}</strong>
      <section><span>{this.a}</span><span>{this.b}</span><span>{this.c}</span></section>
    </main>;
  }
}
