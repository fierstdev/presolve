import { component } from "presolve";

@component()
export class Comparison extends Component {
  render() {
    return <article><h1>Capability comparison</h1><p>Presolve treats component identity, dependency topology, updates, routes, artifact integrity, and deployment handoffs as compiler products.</p><table><thead><tr><th>Concern</th><th>Presolve</th><th>Runtime-first frameworks</th></tr></thead><tbody><tr><td>Rendering authority</td><td>Compiler-published HTML and narrow runtime plans</td><td>General-purpose framework renderer</td></tr><tr><td>Routing authority</td><td>Compiler file-route manifest</td><td>Framework or application router</td></tr><tr><td>Deployment inventory</td><td>Digest-bound compiler artifacts</td><td>Build-tool output convention</td></tr></tbody></table><p>These are architecture differences, not a benchmark claim. Performance comparisons must publish reproducible workloads and measured outputs before making numerical assertions.</p></article>;
  }
}
