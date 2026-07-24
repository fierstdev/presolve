@component("x-context-diagnostic-products")
class ContextDiagnosticProducts extends Component {
  source = state("ready");
  @context() value!: string;
  @provide(ContextDiagnosticProducts.value)
  providedValue: string = this.source;
  @consume(ContextDiagnosticProducts.value)
  selectedValue!: string;
  render() { return <main />; }
}
