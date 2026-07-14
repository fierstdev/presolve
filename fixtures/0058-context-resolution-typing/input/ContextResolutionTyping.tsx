type ThemeAlias = string;

@component("x-context-contracts")
class ContextContracts extends Component {
  localTheme = state("dark");
  localFlag = state(true);

  @context() exact!: string;
  @context() fallback: string = "en";
  @context() noSource!: string;
  @context() valueMismatch!: boolean;
  @context() declarationMismatch!: string;
  @context() defaultMismatch: number = "bad";
  @context() choice: string | number = "wide";
  @context() aliasValue: ThemeAlias = "alias";
  @context() unknown!: MissingAlias;

  @provide(ContextContracts.exact)
  exactProvider: string = this.localTheme;

  @provide(ContextContracts.valueMismatch)
  valueMismatchProvider: boolean = "bad";

  @provide(ContextContracts.declarationMismatch)
  declarationMismatchProvider: boolean = this.localFlag;

  @consume(ContextContracts.exact)
  exactConsumer!: string;

  @consume(ContextContracts.fallback)
  fallbackConsumer!: string;

  @consume(ContextContracts.choice)
  narrowedChoice!: string;

  @consume(ContextContracts.choice)
  widenedChoice!: string | number | boolean;

  @consume(ContextContracts.aliasValue)
  aliasConsumer!: ThemeAlias;

  @consume(ContextContracts.unknown)
  unknownConsumer!: MissingAlias;

  render() { return <main />; }
}

@component("x-context-boundary")
class ContextBoundary extends Component {
  boundaryTheme = state("boundary");

  @provide(ContextContracts.exact)
  boundaryProvider: string = this.boundaryTheme;

  @consume(ContextContracts.exact)
  boundaryConsumer!: string;

  render() { return <section />; }
}

@component("x-context-toolbar")
class ContextToolbar extends Component {
  @consume(ContextContracts.noSource)
  unresolvedValue!: string;

  @consume(ContextContracts.exact)
  scopedExact!: string;

  render() { return <aside />; }
}
