import { ContextOwner } from "./ContextOwner";

@component("x-context-users")
class ContextUsers extends Component {
  @provide(ContextOwner.importedValue)
  importedProvider: string = "imported-provider";

  @consume(ContextOwner.importedValue)
  importedConsumer!: string;

  render() { return <main>Users</main>; }
}
