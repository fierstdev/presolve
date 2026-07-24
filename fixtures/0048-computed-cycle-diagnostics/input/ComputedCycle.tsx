@component("x-computed-cycle")
class ComputedCycle extends Component {
  @computed()
  get alpha() { return this.beta; }

  @computed()
  get beta() { return this.alpha; }

  render() {
    return <output>Cycle</output>;
  }
}
