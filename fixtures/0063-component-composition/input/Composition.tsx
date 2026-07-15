@component("x-theme-contract")
class ThemeContract extends Component {
  @context()
  color: string = "blue";
  render() { return <span />; }
}

@component("x-slot-consumer")
class SlotConsumer extends Component {
  @consume(ThemeContract.color)
  color!: string;
  render() { return <em>{this.color}</em>; }
}

@component("x-card")
class Card extends Component {
  @slot()
  children!: SlotContent;
  @slot()
  header!: SlotContent;
  render() { return <article><header><slot name="header" /></header><section><slot /></section></article>; }
}

@component("x-shell")
class Shell extends Component {
  render() {
    return <Card><template slot="header"><h1>Title</h1><small>Caption</small></template><SlotConsumer /><p>Body</p></Card>;
  }
}

@component("x-page")
@route("/")
class Page extends Component {
  render() { return <main><Shell /><Card /><Card><p>Repeated</p></Card></main>; }
}
