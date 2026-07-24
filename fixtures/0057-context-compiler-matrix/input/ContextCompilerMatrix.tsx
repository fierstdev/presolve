type ThemeName = string;

@component("x-context-compiler-matrix")
class ContextCompilerMatrix extends Component {
  count = state(1);
  other = state(0);
  selectedTheme = state("light");

  @computed()
  get doubled() { return this.count * 2; }

  @context()
  theme: ThemeName = "dark";

  @context()
  locale: string = "en";

  @context()
  total!: number;

  @context()
  unused!: string;

  @provide(ContextCompilerMatrix.theme)
  providedTheme: ThemeName = this.selectedTheme;

  @provide(ContextCompilerMatrix.total)
  providedTotal: number = this.doubled;

  @consume(ContextCompilerMatrix.theme)
  themeValue!: ThemeName;

  @consume(ContextCompilerMatrix.locale)
  localeValue!: string;

  @consume(ContextCompilerMatrix.total)
  firstTotal!: number;

  @consume(ContextCompilerMatrix.total)
  secondTotal!: number;

  @action()
  incrementTwice() {
    this.count += 1;
    this.count += 1;
  }

  @action()
  ignore() { this.other += 1; }

  render() {
    return <main><button onClick={() => this.incrementTwice()}>Increment</button><button onClick={() => this.ignore()}>Ignore</button></main>;
  }
}
