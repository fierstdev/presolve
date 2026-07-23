# Phase N N7-B nested Form field-path contract

N7-B extends the frozen declaration-level Form model without introducing
object-proxy forms, DOM-derived ownership, or a generic path interpreter.
It admits a finite, compiler-validated static path as the serialized key of a
normal compiler-owned Field.

## Source form

The existing `@field()` decorator gains one optional second, static string
argument:

```tsx
@form()
profile!: Form;

@field("profile", "address.street")
street = "";

@field("profile", "preferences.email")
emailUpdates = true;
```

One-argument `@field("profile")` remains the root-path spelling, whose field
path is the authored property name. The second argument does not identify a
different field instance and must not be a runtime expression; it only changes
the compiler-issued serialized path of this exact Field declaration.

## Path grammar and identity

A path is one or more identifier segments separated by `.`. A segment begins
with an ASCII letter or `_` and continues with ASCII letters, digits, or `_`.
Empty segments, numeric indexes, brackets, escapes, prototype names,
duplicates, dynamic values, and paths longer than 16 segments are rejected.

`FieldId` remains a declaration identity based on the owner Form and authored
property. N7-B adds a canonical `FormFieldPath` to the Field entity and runtime
artifacts. Paths are unique per Form, even if their local property names differ.
This preserves every existing field and slot identity while giving
serialization an explicit, inspectable nested shape.

## Types and execution

Each Field remains independently typed, initialized, controlled, validated,
dirty/touched-tracked, and instance-qualified. N7-B does not infer an object
type from the sibling paths and does not expose a mutable parent object to
application source. Field controls continue to operate on their exact Field
slots only.

The three existing submission formats acquire the following compiler-owned
projection:

* `json` creates nested plain objects according to canonical path segments;
* `form-data` and `url-encoded` use the canonical dotted path as the emitted
  key, preserving the existing primitive/array conversion rules; and
* reset, validation, control writes, and resume continue to use the unchanged
  exact Field identity rather than traversing a serialized object.

Collisions such as `address` and `address.street` are rejected because their
serialized object shapes are ambiguous. A path segment never invokes a
property getter, prototype lookup, or user code.

## Artifacts, runtime, and resume

The Forms runtime artifact advances by one schema version and stores each
Field's ordered `path` segments. The generated runtime validates those segments
and constructs JSON submission objects only from artifact records; it never
reads a browser form to reconstruct an object. Existing Form instance slots and
resume records remain unchanged, because each leaf Field still owns its own
value/dirty/touched/validation slots.

Malformed path records, duplicate paths, and prefix collisions fail artifact
validation before activation. Resume accepts N7-B only when the exact existing
leaf slots and artifact schema match; no serialized JSON object is captured as
an authority.

## Deliberate exclusions

N7-B excludes field arrays, dynamic or computed segments, index paths,
optional-path semantics, object-valued controls, custom serialized names,
cross-field object validators, browser `FormData` authority, and arbitrary
object mutation. Those require their own identity and update contracts.

## Required proof

Implementation must prove positive root and nested paths, each submission
format, duplicate and prefix-collision diagnostics, malformed artifact
rejection, repeated component-instance isolation, deterministic artifacts,
resume restoration of leaf slots, and a generated-browser controlled-control
submission proof.
