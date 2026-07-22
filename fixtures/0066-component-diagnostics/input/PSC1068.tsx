@component("x-diagnostic") class Diagnostic extends Component {
  @slot("invalid") value!: SlotContent;
  render() { return <main />; }
}
