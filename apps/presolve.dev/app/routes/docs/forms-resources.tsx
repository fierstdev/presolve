import { component, Component } from "presolve";

@component()
export class FormsAndResources extends Component {
  render() {
    return <article><h1>Forms and resources</h1><p>Forms declare their host, fields, validation, serialization, and submit action. Resources, loaders, and server actions are explicit capability boundaries with compiler-issued plans.</p><p>The static Cloudflare adapter currently rejects executable server plans rather than running arbitrary server JavaScript.</p></article>;
  }
}
