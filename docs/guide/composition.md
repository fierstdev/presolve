# Composition

Composition keeps ownership explicit while leaving component and instance
planning to the compiler.

## Slots

Declare slot fields with `slot()` and the `SlotContent` type. `children` is the
default slot; other field names are named slots.

```tsx
import { slot, Component, type SlotContent } from "presolve";

export class Card extends Component {
  children: SlotContent = slot();
  actions: SlotContent = slot();

  render() {
    return <article><slot /><footer><slot name="actions" /></footer></article>;
  }
}
```

Pass named slot content through a direct `<template slot="name">` child:

```tsx
<Card>
  <p>Body</p>
  <template slot="actions"><button>Save</button></template>
</Card>
```

Slot content remains owned by the caller even though it is placed in the
callee's outlet. Do not use a different nested-children syntax until the
compiler supports its lowering.

## Context (legacy compatibility)

The current public API declares Context with decorators on a static field and
uses a stable `"Class.field"` designator for providers and consumers. This is
an alpha-compatibility form, not decorator-free V2 source.

```tsx
import { component, consume, context, provide, Component } from "presolve";

@component()
export class ThemeRoot extends Component {
  @context() static mode = "light";
  @provide("ThemeRoot.mode") modeForChildren = ThemeRoot.mode;

  render() { return <main><ThemeLabel /></main>; }
}

@component()
export class ThemeLabel extends Component {
  @consume("ThemeRoot.mode") mode!: string;

  render() { return <span>{this.mode}</span>; }
}
```

Treat provider-owned values as read-only to consumers. Expose an action through
normal component composition when a consumer must request a change. Provider
resolution is compiler-planned; it is not runtime tree traversal.
