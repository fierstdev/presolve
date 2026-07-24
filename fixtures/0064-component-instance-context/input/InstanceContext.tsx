@component("x-theme") class Theme extends Component {
  @context() color: string = "default";
  render() { return <div />; }
}
@component("x-leaf") class Leaf extends Component {
  @consume(Theme.color) color!: string;
  render() { return <span>{this.color}</span>; }
}
@component("x-light") class Light extends Component {
  @provide(Theme.color) color: string = "light";
  render() { return <Leaf />; }
}
@component("x-nearest") class Nearest extends Component {
  @provide(Theme.color) color: string = "nearest";
  render() { return <Leaf />; }
}
@component("x-dark") class Dark extends Component {
  @provide(Theme.color) color: string = "dark";
  render() { return <Nearest />; }
}
@component("x-default-branch") class DefaultBranch extends Component {
  render() { return <Leaf />; }
}
@component("x-context-page") @route("/") class ContextPage extends Component {
  render() { return <main><Light /><Dark /><DefaultBranch /></main>; }
}
