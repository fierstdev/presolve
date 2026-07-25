# Resumability

Presolve publishes a resumability artifact with every supported interactive
application build. It allows the browser runtime to continue from a
compiler-defined application snapshot instead of reconstructing the entire
application through a generic hydration renderer.

Resumability is a compiler capability, not an application API. Normal
components, state, actions, computed values, Context, slots, and forms use the
same authoring model whether an application starts cold or resumes.

## What the compiler publishes

`pnpm build` emits static HTML and compiler-owned browser artifacts in `dist/`.
For an interactive application this includes `resume.runtime.json`, a
compiler-issued manifest describing:

- resume boundaries and DOM anchors;
- retained state and form slots with their codecs;
- capture and restore programs;
- activation policies and browser chunks; and
- the build identity that binds the snapshot to the published artifact set.

These files are implementation artifacts. Do not edit them, generate a
snapshot by hand, or couple application code to their internal IDs. The CLI
and runtime validate their integrity before using them.

## Cold start and resume

On a cold visit, the browser uses the static page and the narrow runtime
artifacts published by the compiler. When a compatible snapshot is available,
the runtime validates its build identity and structure before restoring the
compiler-defined slots and bindings.

If validation fails, Presolve falls back to a cold start. It does not attempt
best-effort restoration against an incompatible artifact or silently reuse
stale state. That behavior is deliberate: a correct cold start is safer than
a partially restored application.

This is different from a generic hydration model. Presolve does not mount a
second application renderer and replay the component tree merely to make the
page interactive. The compiler has already emitted the required ownership,
binding, and activation plan.

## What authors need to do

Most applications need no resumability-specific syntax. Write ordinary
compiler-admitted components and use the framework vocabulary normally:

```tsx
import { action, component, state, Component } from "@presolve/core";

@component()
export class Counter extends Component {
  count = state(0);

  @action()
  increment(): void {
    this.count += 1;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
```

The compiler decides which values are retained, how they are encoded, when an
action is activated, and how the DOM binding is restored. Keep values within
the supported serializable and capability boundaries. If Presolve cannot
lower an operation safely, `pnpm check` fails with a compiler diagnostic; it
does not switch to a general client runtime.

Forms, Context, slots, repeated components, and computed values follow the
same rule: their exact instance and ownership relationships come from the
compiler artifact, not runtime discovery.

## Inspect a build

Use the compiler explanation surface before inspecting generated JavaScript:

```sh
pnpm build
presolve explain app/routes/index.tsx
```

Use `dist/resume.runtime.json` when you need to diagnose a deployed artifact
or verify that a build includes resumability metadata. Treat it as a
read-only, versioned compiler product. The output of `presolve explain` is the
supported source-level explanation for components, state, bindings, and
generated artifacts.

## Deployment

Deploy the complete compiler-published artifact set. Do not upload only the
HTML, replace generated runtime files, or combine assets from different
builds: resumability validation relies on the matching build identity and
artifact inventory. The Cloudflare adapter validates this inventory during
`pnpm deploy:prepare` and `pnpm deploy`.

See [production builds](production.md) and the
[Cloudflare deployment reference](../reference/cloudflare.md).

## Alpha boundary

Resumability is supported for compiler-admitted application semantics in this
alpha. It is not a promise that arbitrary TypeScript, unmodeled browser
effects, or arbitrary npm package state can be captured and restored. Keep
third-party behavior behind declared compiler capability boundaries as
described in [third-party packages](third-party-packages.md).
