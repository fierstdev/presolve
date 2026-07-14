@component("x-shared-context")
class SharedContext extends Component {
  @context() value: string = "alpha";
  @provide(SharedContext.value) providedValue: string = "alpha-provider";
  @consume(SharedContext.value) selectedValue!: string;
  render() { return <main>Alpha</main>; }
}
