# Phase M M6-B Context language conformance

**Status:** M6-B implementation authority.

## Decision

Context is the one Phase M composition family that requires a production
compiler-language refinement. The inherited form declared Context on an
instance field and named it from another decorator as a static-member
expression. TypeScript has no sound way to represent that relation: an instance
field is not present on the class static side, and a self-reference within the
same class encounters the class temporal-dead-zone rule.

The framework-facing Context language is therefore:

```tsx
@component("x-theme")
class Theme extends Component {
  @context()
  static mode: "light" | "dark" = "light";
}

@component("x-reader")
class Reader extends Component {
  @provide("Theme.mode")
  providedMode: "light" | "dark" = this.selectedMode;

  @consume("Theme.mode")
  mode!: "light" | "dark";
}
```

`@context()` marks a typed static field. Its optional literal initializer is
Context default metadata; it is not mutable static State, a JavaScript runtime
value, or a framework Context object. `@provide()` and `@consume()` take one
literal qualified designator with grammar `Identifier.Identifier`. The compiler
validates both identifier segments and resolves that source text to the exact
decorated static Context declaration before it constructs any Context identity
or runtime artifact.

The quoted designator is compile-time syntax, not a dynamic string key. There
is no framework registry, map lookup, reflection, injection, proxy, or
provider/consumer runtime. A malformed or unresolved designator follows the
existing canonical compiler diagnostic path.

## Compiler and TypeScript responsibilities

The compiler accepts a static `@context()` declaration, accepts the qualified
literal form in Provider and Consumer decorators, and lowers the resolved
designator through its existing Context identity, type, ownership, dependency,
runtime-plan, artifact, resumability, and browser-execution products. It keeps
existing static-member expression designators accepted for compiler source that
still uses them; that is compiler compatibility, not framework API. The
framework documents and types only the form above.

`@presolve/framework-types` exposes:

```ts
type ContextDesignator = `${string}.${string}`;
function context(): PresolveFieldDecorator;
function provide(designator: ContextDesignator): PresolveFieldDecorator;
function consume(designator: ContextDesignator): PresolveFieldDecorator;
```

This type admits dotted literals while compiler validation remains authoritative
for identifier grammar, existence, type compatibility, visibility, and
selection. It emits no JavaScript, does not execute decorators, and cannot
change Context scheduling or lookup.

## Explicit boundaries

The following remain unavailable in the framework:

- `context<T>()`, Context factories, imported Context handles, and symbols;
- instance `@context()` declarations;
- `@provide(Owner.member)` expression designators in framework source;
- dynamic, computed, or non-literal Context designators;
- consumer writes, fallbacks, hooks, runtime lookup, and Context mutation APIs;
- a framework reactive runtime, source rewrite, diagnostic suppression, or
  compatibility shim.

This is a source-language change rather than a transitional alias. Framework
fixtures use the new spelling exactly and the compiler sees that spelling
unchanged. No legacy framework source is accepted or translated.

## Evidence

`scripts/verify-m6-context-conformance.sh` proves all of the following:

1. the source is byte-identical to the canonical Context runtime fixture;
2. TypeScript 7.0 resolves the static declaration and dotted designators;
3. the canonical explicit compiler check accepts the unchanged source;
4. the existing real-browser Context probe observes compiler-planned source
   binding and update behavior; and
5. no framework runtime or artifact decoder is introduced.

The browser proof is the compiler's existing Context-runtime fixture. It
continues to establish exact Context source binding and updates through emitted
plans rather than a framework-owned lookup or scheduler.
