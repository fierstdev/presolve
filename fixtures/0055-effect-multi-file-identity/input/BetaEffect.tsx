@component("x-beta-effect")
class BetaEffect extends Component {
  title = state("Beta");

  @effect()
  syncTitle() { document.title = this.title; }

  render() { return <p>Beta</p>; }
}
