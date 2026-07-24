@component("x-alpha-effect")
class AlphaEffect extends Component {
  count = state(1);

  @effect()
  report() { console.log(this.count); }

  render() { return <p>Alpha</p>; }
}
