@component("x-runtime-theme") class RuntimeTheme extends Component {
  @context() tone: string = "neutral";
  render() { return <span />; }
}
@component("x-runtime-leaf") class RuntimeLeaf extends Component {
  @consume(RuntimeTheme.tone) tone!: string;
  count = state(0);
  render() { return <button onClick={() => this.increment()}>{this.count}</button>; }
  increment() { this.count++; }
}
@component("x-runtime-card") class RuntimeCard extends Component {
  @slot() children!: SlotContent;
  @provide(RuntimeTheme.tone) tone: string = "card";
  render() { return <article><slot /><RuntimeLeaf /></article>; }
}
@component("x-runtime-page") @route("/") class RuntimePage extends Component {
  render() { return <main><RuntimeCard><RuntimeLeaf /></RuntimeCard><RuntimeCard><p>Sibling</p></RuntimeCard></main>; }
}
