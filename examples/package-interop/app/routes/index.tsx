import { action, Component } from "presolve";
import {
  recordVisit as sendVisit,
  recordVisitAsync as sendVisitAsync,
} from "presolve-example-analytics";

export class PackageInterop extends Component {
  record = action((
    category: string,
    value: number,
    enabled: boolean,
    metadata: null,
  ) => {
    sendVisit(category, value, enabled, metadata);
  });

  recordAsync = action(async (category: string, signal: AbortSignal) => {
    await sendVisitAsync(category, signal);
  });

  render() {
    return (
      <main>
        <h1>Decorator-free package interoperability</h1>
        <p>The compiler proves the imported export and Vite bundles its browser implementation.</p>
        <button
          id="record-visit"
          onClick={() => this.record("checkout", 2, true, null)}
        >
          Record visit
        </button>
        <output id="visit-count" aria-live="polite">0</output>
        <output id="visit-detail" aria-live="polite">idle</output>
        <button id="record-slow" onClick={() => this.recordAsync("slow")}>
          Record slowly
        </button>
        <button id="record-fast" onClick={() => this.recordAsync("fast")}>
          Replace with fast record
        </button>
        <button id="record-fail" onClick={() => this.recordAsync("fail")}>
          Record failure
        </button>
        <output id="async-result" aria-live="polite">idle</output>
      </main>
    );
  }
}
