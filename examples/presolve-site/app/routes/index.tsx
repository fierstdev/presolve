import { component } from "presolve";

@component()
export class Home extends Component {
  render() {
    return <article><h1>Presolve</h1><p>A compiler-founded framework that publishes the HTML, runtime, route, and deployment facts your application actually needs.</p><p><a href="/docs/getting-started/">Start building</a></p><h2>Why Presolve</h2><ul><li>Components are compiler semantics, not a JavaScript renderer convention.</li><li>Routes and artifacts are compiler-owned products.</li><li>Deployments validate immutable artifact digests before upload.</li></ul></article>;
  }
}
