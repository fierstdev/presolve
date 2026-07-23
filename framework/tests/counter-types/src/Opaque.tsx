import { trackPurchase } from "@acme/analytics";

@component("x-opaque")
class OpaqueTerminal extends Component {
  @action()
  @opaque("@acme/analytics", "trackPurchase")
  track(): void {}

  render() {
    return <button onClick={this.track}>Track</button>;
  }
}
