@component("x-layout")
class Layout extends Component {
  @slot() children!: SlotContent;
  render() { return <main><header>Representative</header><slot /></main>; }
}
