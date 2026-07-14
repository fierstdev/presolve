@component("x-context-owner")
class ContextOwner extends Component {
  @context() importedValue: string = "owner-default";
  render() { return <main>Owner</main>; }
}

export { ContextOwner };
