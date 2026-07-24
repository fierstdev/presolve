@component("x-computed-constant-folding")
class ComputedConstantFolding extends Component {
  @computed()
  get answer() { return 1 + 2; }

  render() {
    return <output>Folded</output>;
  }
}
