@component("x-beta-computed")
class BetaComputed extends Component {
  label = state("EdgeZero");

  @computed()
  get title() { return this.label; }

  render() {
    return <output>Beta</output>;
  }
}
