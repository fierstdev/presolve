import { component, Component } from "presolve";

@component()
export class Documentation extends Component {
  render() {
    return <article><h1>Documentation</h1><p>Presolve keeps the authoring model small and lets the compiler own application semantics.</p><ul><li><a href="/docs/getting-started/">Getting started</a></li><li><a href="/docs/project-structure/">Project structure and routes</a></li><li><a href="/docs/components/">Components</a></li><li><a href="/docs/reactivity/">State and actions</a></li><li><a href="/docs/composition/">Composition</a></li><li><a href="/docs/forms-resources/">Forms and resources</a></li><li><a href="/docs/packages/">Third-party packages</a></li><li><a href="/docs/editor/">VS Code</a></li><li><a href="/docs/deployment/">Cloudflare deployment</a></li></ul></article>;
  }
}
