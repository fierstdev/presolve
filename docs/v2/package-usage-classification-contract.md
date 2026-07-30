# V2 package-usage classification contract

## Goal

Presolve does not require a framework decorator merely because application
source imports a third-party package. Package participation is classified from
the authority-proven use site. The same package export may be an ordinary
build input in one module, a terminal browser invocation in an Action or
Effect, or a value-producing semantic dependency elsewhere.

This contract replaces package-wide admission with use-site admission. It does
not claim that a package written for an incompatible host platform or another
framework's component runtime can execute unchanged.

## Classification

Every external import selected by conventional application discovery belongs
to exactly one of these classes:

1. **Erased or adapter-owned use.** Type-only imports, CSS, CSS Modules,
   PostCSS/Tailwind plugins, imported media, and other Vite-owned inputs create
   no Presolve runtime semantic. TypeScript or Vite reports their own
   diagnostics. Presolve records no component, reactive, capability, or resume
   identity.
2. **Terminal client invocation.** A decorator-free Action or Effect may make
   one authority-proven direct call to a named import when the result is
   discarded. The project Vite installation bundles that exact export.
   Presolve diagnoses resolution, call shape, argument transport, execution
   boundary, publication integrity, and lifecycle; it does not diagnose the
   package implementation.
3. **Value-producing semantic use.** A package result that enters State,
   Computed, render output, a Form, a Resource, Context, serialization, or a
   resume product requires a closed usage contract for purity, dependency,
   value type/codec, execution boundary, and resume behavior. TypeScript return
   type alone cannot establish those facts.
4. **Framework or server integration.** Component exports, route loaders,
   server actions, codecs, and provider adapters require their dedicated
   compiler products. A package name, JSX spelling, or directory location does
   not grant those semantics.

Classification is per call site, not per package. A package may participate in
multiple classes without receiving blanket authority.

## Decorator-free terminal source form

The first executable adoption slice admits a synchronous decorator-free Action
whose complete body is one direct call to a named import:

```tsx
import { trackPurchase } from "@acme/analytics";
import { action, Component } from "presolve";

export class Checkout extends Component {
  track = action(() => {
    trackPurchase();
  });
}
```

TypeScript authority must resolve the call site to the exact named import and
declaration module. Default imports, namespace-member calls, computed members,
dynamic import, `eval`, `Function`, free captures, and server-owned modules are
not part of the first slice. The call result must be discarded. The admitted
first argument surface is empty; serializable arguments require an explicit
artifact amendment rather than retaining or evaluating handler source.

The legacy `@action() @opaque(package, export)` form remains compatibility-only
and is not evidence for this source form.

## Serializable Action arguments and Promise completion amendment

The next executable slice extends only the terminal Action form above. It does
not grant package-wide semantics or admit arbitrary Action source.

### Synchronous parameter forwarding

An Action may forward its complete ordered parameter list to the named package
export:

```tsx
import { recordMetric } from "@acme/analytics";
import { action, Component } from "presolve";

export class Checkout extends Component {
  record = action((category: string, value: number) => {
    recordMetric(category, value);
  });

  render() {
    return <button onClick={() => this.record("checkout", 1)}>Record</button>;
  }
}
```

Every forwarded value must be an existing canonical Action parameter with an
exact `string`, `number`, `boolean`, or `null` annotation. The package call
must forward every Action parameter exactly once and in declaration order.
The event artifact remains the value authority and transports the corresponding
static serializable arguments. Literals inside the package call, reordering,
duplication, omission, local variables, State reads, computed values, object or
array values, captures, spreads, and member expressions are not admitted by
this amendment.

The TypeScript authority must prove the resolved named export, its resolved call
signature, and compatibility with the authored call. A synchronous invocation
must not return a Promise. Its result remains discarded and cannot enter a
compiler-owned product.

### Promise-aware invocation

An asynchronous terminal Action has one additional compiler-injected final
parameter:

```tsx
import { recordMetric } from "@acme/analytics";
import { action, Component } from "presolve";

export class Checkout extends Component {
  record = action(async (category: string, signal: AbortSignal) => {
    await recordMetric(category, signal);
  });

  render() {
    return <button onClick={() => this.record("checkout")}>Record</button>;
  }
}
```

The final parameter must be named `signal`, have the exact TypeScript-resolved
DOM `AbortSignal` identity, and be forwarded as the final package argument.
The event caller supplies only the preceding serializable parameters. The
package call must be the Action's sole awaited statement, and the resolved call
signature must return `Promise<void>`. Non-awaited Promises, non-void Promise
results, additional statements, catches/finally blocks, and caller-supplied
signals fail closed.

### Artifact and lifecycle

The terminal-invocation artifact records, for each invocation:

- the ordered event-argument indexes and primitive codecs;
- synchronous or Promise completion;
- whether the compiler appends an owned `AbortSignal`;
- `replace_previous_per_component_instance` concurrency for Promise work;
- `abort_on_replacement_teardown_or_pagehide` cancellation; and
- `restore_event_without_replay` resume behavior.

The runtime validates those facts before readiness. It constructs the package
argument list from the accepted event record, never from DOM text or retained
handler source. A Promise invocation owns one `AbortController`. A later
activation of the same invocation for the same component instance aborts the
prior activation before calling the export. Component teardown and `pagehide`
abort pending work. Fulfillment records `complete`; an expected abort records
`cancelled` without a failure diagnostic; other rejection or synchronous throw
records `failed` and preserves the package error under
`PSR_PACKAGE_INVOCATION_FAILURE`.

Pending package work is not serialized. Snapshot publication retains only the
existing event binding and terminal-invocation artifact. Resume never imports
or replays a prior activation; the next accepted event starts a new generation
from the validated registry. A stale settlement from an aborted generation
cannot overwrite the current evidence.

### Diagnostic ownership

`PSV2P1001` remains the source-admission diagnostic and now distinguishes
unproven named imports, argument mapping, signature/completion, and signal
identity failures. Artifact mismatches remain fatal boot diagnostics.
Presolve owns transport, generation, cancellation, failure recording, and
resume behavior. TypeScript owns assignability and declaration resolution,
Vite owns platform bundling, and the package owns its implementation and
cooperation with the supplied `AbortSignal`.

## Publication and integrity

The compiler publishes an immutable terminal-invocation record joined to the
canonical Action and component identity. The ergonomic CLI generates a Vite
entry that imports only the authority-proven module/export pair and publishes
one callable registry through the ordinary file-route digest inventory.

The registry coordinate, compiler build identity, module specifier, named
export, call-site provenance, client execution boundary, and stateless
event-restore policy are exact artifact facts. Package source is never serialized
into a compiler semantic contract, inspected for framework behavior, or
executed during compilation.

Resolution failure, a missing/non-callable bundled export, digest drift,
unsupported source shape, a used return value, or server ownership fails
closed. No source evaluation, compatibility decorator synthesis, or generic
client renderer is permitted.

## Runtime and lifecycle

The terminal invocation runs once after the owning Action batch completes. It
cannot read or write compiler-owned State except through arguments admitted by
a later contract. It owns no reactive dependency and no snapshot codec.
Component teardown does not attempt to reverse a completed external side
effect. A pending asynchronous value is not awaited by this synchronous first
slice; Promise-aware cancellation requires its own capability contract.

The runtime records completion or failure evidence and reports a stable runtime
diagnostic for a rejected invocation. It never interprets package internals.
Because a completed activation has no resumable state, the canonical Action
event is restored without replaying the package side effect. A later accepted
event invokes the package export normally from the restored registry.

## Diagnostic ownership

Presolve diagnostics name only facts Presolve owns:

- canonical use-site classification and source span;
- TypeScript-proven import/export identity;
- admitted call/argument/result shape;
- client/server execution boundary;
- compiler artifact and publication integrity; and
- activation/resume lifecycle.

TypeScript remains authoritative for types and module declarations. Vite
remains authoritative for bundling and host-platform compatibility. Package
runtime errors retain the package error as evidence under a Presolve activation
diagnostic, without claiming compiler knowledge of package implementation.

## Acceptance

The first slice is complete only when:

1. the decorator-free source above passes without `@component`, `@action`, or
   `@opaque`;
2. aliases resolve and same-spelled local functions do not acquire package
   authority;
3. default/namespace/dynamic/server calls and used return values fail with
   stable diagnostics;
4. deterministic double builds produce identical terminal records and registry
   bytes;
5. a real browser proves the package export executes once per accepted Action,
   remains interactive after compatible resume, and reports missing/rejecting
   exports without corrupting compiler-owned State; and
6. `presolve explain --capabilities` reports terminal package invocation as an
   admitted bounded capability while retaining broader value-producing package
   use as explicitly contract-required.

Later slices may add serializable arguments, Promise cancellation, pure return
values, codecs, or component adapters only by amending the relevant
compiler/artifact/lifecycle products. They may not broaden this terminal call
implicitly.
