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

## Context status

Context projection and resume exist in compiler products, but the current beta
does not expose a new-project Context declaration. The compiler fails closed
instead of inferring a shared-value API from an ordinary field or runtime tree.
Use explicit component inputs and Actions for current applications.

Historical Context declarations are retained only for migration analysis; they
are not part of this guide's authoring surface.
