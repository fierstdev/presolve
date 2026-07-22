# Phase M M2 framework types contract

**Status:** M2 implementation authority and completion record.

## Scope

M2 establishes the isolated `framework/` workspace and its first private
package, `@presolve/framework-types`. It supplies only ambient TypeScript
declarations needed for the existing Counter form:

```tsx
@component("x-counter")
class Counter extends Component {
  count = state(0);
  @action() increment() { this.count += 1; }
  render() { return <button onClick={this.increment}>Count: {this.count}</button>; }
}
```

The fixture source is byte-identical to the existing canonical Counter example.
It is type-checked with the repository-pinned TypeScript 7.0 native
`tsc --project` using an explicit `types` entry, then checked through the
accepted explicit `presolve check` command path.

## Package boundary

`framework/packages/framework-types/src/index.d.ts` is the only package source
in M2. It declares ambient `Component`, `component`, `action`, `state`, and minimal
preserved-TSX typing. It emits no JavaScript and does not implement a
decorator, State, JSX factory, renderer, scheduler, Context, product reader,
compiler adapter, or runtime behavior.

The Counter fixture uses a local TypeScript type-root relay solely because this
private workspace package is not installed during repository verification. The
relay references the package declaration file; it contains no duplicate type or
semantic definition. Application source continues to use no framework import,
alias, transform, or rewritten compiler form.

## Completion evidence

`./scripts/verify-m2-framework-types.sh` proves all of the following:

1. workspace/package identity, privacy, declaration entrypoint, and absence of
   non-declaration package source;
2. absence of framework runtime or transform vocabulary;
3. exact byte equality with `examples/counter/src/Counter.tsx`;
4. TypeScript resolution of `@presolve/framework-types` from the fixture's
   `types` configuration using the pinned TypeScript 7.0 CLI; and
5. canonical compiler success through the explicit Counter configuration and
   source mapping.

The verifier invokes the M0/M1 plan verifier and `git diff --check`. Repository
layout and identity audits cover the new `framework/` root.

TypeScript 7.1 is intentionally not accepted through a version range. M8 may
add it to the framework compatibility matrix only after an explicit pinned-7.1
rerun of this fixture and every later declaration fixture.

## Exclusions and next boundary

M2 does not expose Computed, Effect, Context, Slot, or Form declarations; it
does not add a source transform or invoke a compiler from JavaScript. Its Action
declaration is type-only and adds no event wrapper or reactive behavior.
