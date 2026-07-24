import { component } from "presolve";

@component()
export class Documentation extends Component {
  render() {
    return <article><h1>Documentation</h1><p>Learn the small authoring vocabulary, compiler-owned routes, package capabilities, and production deployment path.</p><ul><li><a href="/docs/getting-started/">Getting started</a></li><li><a href="/docs/components/">Components and state</a></li><li><a href="/docs/deployment/">Cloudflare deployment</a></li></ul></article>;
  }
}
