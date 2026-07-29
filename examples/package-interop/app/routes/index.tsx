import { action, Component } from "presolve";
import { recordVisit as sendVisit } from "presolve-example-analytics";

export class PackageInterop extends Component {
  record = action(() => {
    sendVisit();
  });

  render() {
    return (
      <main>
        <h1>Decorator-free package interoperability</h1>
        <p>The compiler proves the imported export and Vite bundles its browser implementation.</p>
        <button id="record-visit" onClick={this.record}>Record visit</button>
        <output id="visit-count" aria-live="polite">0</output>
      </main>
    );
  }
}
