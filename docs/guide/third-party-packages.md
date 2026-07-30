# Third-party packages and compiler boundaries

Presolve applications can install ordinary npm packages. Whether Presolve
needs semantic knowledge depends on how an imported value is used:

| Use | Authority | Presolve meaning |
| --- | --- | --- |
| Types, CSS tooling, media, and build plugins | TypeScript or Vite | No component, reactive, capability, or resume identity. |
| Pure values used only where the compiler has an admitted value contract | TypeScript + Presolve | The value joins that exact contract; the package does not receive blanket framework semantics. |
| A result-discarded browser call from an Action | Presolve + TypeScript + Vite | Presolve proves the call shape and lifecycle; Vite bundles the exact named export. |
| Server capability implementation | Presolve capability + server adapter | The declared capability owns the transport and lifecycle. |
| Arbitrary package behavior | Package only | Not assumed to be compiler-understood. |

The supported terminal Action boundary is designed for
analytics, telemetry, browser SDK notifications, and similar result-discarded
operations.

## Synchronous package Action

Import one named export and make its call the complete Action body:

```tsx
import { recordEvent } from "@acme/analytics";
import { action, Component } from "presolve";

export class Checkout extends Component {
  record = action((
    category: string,
    value: number,
    enabled: boolean,
    metadata: null,
  ) => {
    recordEvent(category, value, enabled, metadata);
  });

  render() {
    return (
      <button onClick={() => this.record("checkout", 2, true, null)}>
        Record checkout
      </button>
    );
  }
}
```

The beta admits exact `string`, `number`, `boolean`, and `null` parameters.
Every parameter must be forwarded once and in order. TypeScript must prove that
the called symbol is the exact named import and that its declaration has the
same parameter types. Aliases retain their import identity; a same-spelled local
function does not.

The package return value must be discarded. Objects, arrays, optional or rest
parameters, reordered/duplicated values, namespace or default imports, dynamic
imports, computed calls, and a second statement are not silently accepted.

## Promise package Action

For cancellable asynchronous work, make the handler `async`, add one final
`signal: AbortSignal`, and make the package call the sole awaited statement:

```tsx
import { recordEventAsync } from "@acme/analytics";
import { action, Component } from "presolve";

export class Search extends Component {
  record = action(async (query: string, signal: AbortSignal) => {
    await recordEventAsync(query, signal);
  });

  render() {
    return (
      <button onClick={() => this.record("compiler framework")}>
        Record search
      </button>
    );
  }
}
```

The package declaration must be exactly compatible with
`(query: string, signal: AbortSignal) => Promise<void>`. Application callers do
not pass the signal; the public `action()` overload hides it from the returned
event-call signature.

Presolve owns one pending invocation per component instance and package Action:

- a newer call aborts the previous call before starting;
- component or structural teardown aborts pending work;
- `pagehide` aborts pending work;
- cancellation is recorded as cancellation, not package failure;
- stale settlements cannot overwrite the newer invocation's evidence; and
- resume restores the event boundary without replaying a pre-navigation call.

A thrown or rejected package error is preserved in
`PSR_PACKAGE_INVOCATION_FAILURE`. A missing or integrity-invalid generated
registry fails closed before the call can execute.

## What Vite does

TypeScript proves the imported symbol and exact signature. Presolve proves the
Action shape, argument codecs, component instance, concurrency, cancellation,
diagnostics, and resume policy. Project-local Vite bundles only the exact
authority-proven module/export pair into
`/presolve.package-invocations.js`.

Vite never decides that a handler is an Action or that arbitrary package source
is safe to resume. Presolve never executes the package during compilation or
serializes the handler's source for runtime evaluation.

## Historical package declarations

Earlier package declarations are retained only for migration analysis. Current
applications use the Action forms above; package uses that need values, codecs,
server execution, or framework-specific adapters require their own admitted
capability contract.
