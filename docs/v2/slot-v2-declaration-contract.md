# V2 Slot declaration contract

This contract adds the decorator-free source form for the existing canonical
Slot model. It does not add runtime Slot behavior, a new Slot kind, or a
second composition decoder.

## Source and authority

Inside a TypeScript-authority-proven V2 `Component`, an instance field may
declare a Slot only as:

```ts
children: SlotContent = slot();
header: SlotContent = slot();
```

`slot` must resolve through the V2 TypeScript authority to Presolve's canonical
`slot` intrinsic. The compiler sends the exact direct-call callee position and
accepts only a matching authority response. An unrecognized call is an
ordinary initializer; it never becomes a Slot by spelling, import text, or
runtime inspection.

The call has zero arguments, the field is non-static, and its declared type is
exactly `SlotContent`. `children` retains the existing default-Slot identity;
every other field name retains the existing named-Slot identity. V2 Slot fields
cannot carry legacy semantic decorators. `@slot()` remains a legacy
compatibility input and shares the same canonical product.

## Canonical product

The authority-resolved field lowers to one `CanonicalAuthoredDeclarationKindV1::Slot`.
The V2 component graph then constructs the existing `SlotDeclaration`,
`SlotId`, Slot-content fragments, Slot bindings, and runtime artifacts. No
adapter parses source, selects a Slot outlet, or reconstructs composition from
the DOM.

## Proof required for structural Slot hosts

The browser authority fixture must use this V2 form for a component with a
conditional and keyed host around `<slot />`. Caller-owned projected State,
binding, and event behavior must remain live after host replacement and keyed
reorder; malformed Slot authority, artifact binding, marker, or ownership
evidence must fail closed. Slot-projected structural resume remains out of
scope until a dedicated capture contract exists.
