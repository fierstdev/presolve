import { component } from "presolve";

@component()
export class Deployment extends Component {
  render() {
    return <article><h1>Cloudflare deployment</h1><p>Presolve validates the compiler artifact inventory, emits a Workers Static Assets projection, and delegates upload and version rollback to Wrangler.</p><p>Server package capabilities remain compiler handoffs until a capability-specific executor is published.</p></article>;
  }
}
