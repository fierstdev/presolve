import { component, slot, Component, type SlotContent } from "presolve";

@component()
export class SiteLayout extends Component {
  @slot() children!: SlotContent;

  render() {
    return <main><header><a href="/">Presolve</a><nav><a href="/docs/">Docs</a><a href="/examples/">Examples</a><a href="/compare/">Compare</a></nav></header><slot /><footer>Compiler-founded UI for the web.</footer></main>;
  }
}
