@component("x-context-diagnostic-parity")
class ContextDiagnosticParity extends Component {
  @context()
  theme: string;

  @provide(ContextDiagnosticParity.theme)
  providedTheme: boolean = "dark";

  @consume(ContextDiagnosticParity.theme)
  selectedTheme!: number;

  render() {
    return <main />;
  }
}
