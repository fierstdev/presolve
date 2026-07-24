import { component } from "presolve";

@component()
export class GettingStarted extends Component {
  render() {
    return <article><h1>Getting started</h1><ol><li>Create an app with npm create presolve.</li><li>Author routes under app/routes.</li><li>Run presolve dev.</li><li>Run presolve deploy cloudflare when ready.</li></ol><p>Presolve discovers routes and publishes only compiler-validated output.</p></article>;
  }
}
