@component("x-context-runtime-matrix")
class ContextRuntimeMatrix extends Component {
  count = state(1);
  other = state(0);

  @computed()
  get doubled() { return this.count * 2; }

  @context()
  total!: number;

  @provide(ContextRuntimeMatrix.total)
  providedTotal: number = this.doubled;

  @consume(ContextRuntimeMatrix.total)
  firstTotal!: number;

  @consume(ContextRuntimeMatrix.total)
  secondTotal!: number;

  @action()
  incrementTwice() {
    this.count += 1;
    this.count += 1;
  }

  @action()
  ignore() { this.other += 1; }

  @effect()
  reportTotal() {
    console.log(this.doubled);
  }

  render() {
    return <main><button onClick={() => this.incrementTwice()}>Increment</button><button onClick={() => this.ignore()}>Ignore</button></main>;
  }
}
