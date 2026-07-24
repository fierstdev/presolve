import { component, Component } from "presolve";

@component()
export class Resumability extends Component {
  render() {
    return <article><h1>Resumability</h1><p>Presolve publishes static HTML, browser artifacts, and a compiler-owned resumability manifest. Compatible snapshots restore compiler-defined state and bindings without mounting a generic hydration renderer.</p><p>There is no separate resume API for normal components. Keep state and capabilities within compiler-admitted boundaries, inspect builds with <code>presolve explain</code>, and deploy the complete artifact inventory together.</p><p>When a snapshot does not match its published build, Presolve safely falls back to a cold start.</p></article>;
  }
}
