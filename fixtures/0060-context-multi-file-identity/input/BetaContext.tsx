@component("x-shared-context")
class SharedContext extends Component {
  @context() value: string = "beta";
  @provide(SharedContext.value) providedValue: string = "beta-provider";
  @consume(SharedContext.value) selectedValue!: string;
  render() { return <main>Beta</main>; }
}
