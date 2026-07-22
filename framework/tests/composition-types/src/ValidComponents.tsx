@component("x-card")
export class Card extends Component {
  @slot()
  children!: SlotContent;

  @slot()
  header!: SlotContent;

  render() {
    return <article><slot name="header" /><slot /></article>;
  }
}

@component("x-leaf")
class Leaf extends Component {
  render() { return <span />; }
}

@component("x-local-page")
class LocalPage extends Component {
  render() { return <main><Leaf /><Leaf /></main>; }
}
