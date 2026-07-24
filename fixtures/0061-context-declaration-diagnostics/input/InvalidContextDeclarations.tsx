@component("x-invalid-context-declarations")
class InvalidContextDeclarations extends Component {
  @context("invalid") invalidContext!: string;
  @provide() invalidProvider: string = "invalid";
  @consume() invalidConsumer!: string;
  @provide(Missing.theme) unresolvedProvider: string = "missing";

  @context() duplicateTarget!: string;
  @provide(InvalidContextDeclarations.duplicateTarget)
  duplicateSecond: string = "second";
  @provide(InvalidContextDeclarations.duplicateTarget)
  duplicateFirst: string = "first";

  render() { return <main />; }
}
