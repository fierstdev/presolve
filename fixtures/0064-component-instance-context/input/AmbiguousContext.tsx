@component("x-ambiguous-theme") class AmbiguousTheme extends Component {
  @context() color!: string;
  render() { return <div />; }
}
@component("x-ambiguous-leaf") class AmbiguousLeaf extends Component {
  @consume(AmbiguousTheme.color) color!: string;
  render() { return <span />; }
}
@component("x-ambiguous-page") @route("/ambiguous") class AmbiguousPage extends Component {
  @provide(AmbiguousTheme.color) first: string = "first";
  @provide(AmbiguousTheme.color) second: string = "second";
  render() { return <AmbiguousLeaf />; }
}
