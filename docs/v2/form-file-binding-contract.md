# V2 Form file binding contract

This contract amends the alpha Forms serialization boundary for the canonical
V2 `defineForm()` surface. It does not change legacy decorator semantics.

## Authority

- A file field is admitted only when TypeScript authority proves that the
  resolved canonical `field` signature has a `File[]` value type from the
  configured TypeScript DOM library and type checking accepts its initializer.
- Parser spelling, an authored alias name, `type="file"`, and `bind:files` do
  not independently establish a file field.
- The canonical authoring form is `field<File[]>({ initial: [] })`.

## Binding

- `bind:files` is valid only on a static `<input type="file">` whose field is
  authority-proven as `File[]`.
- The binding commits `Array.from(input.files ?? [])` on `change`.
- Programmatic field writes never assign `input.files`. Form and field reset
  clear the native control through its permitted empty `value`.
- `bind:value` and `bind:checked` remain invalid for file inputs.

## Serialization and resume

- `File[]` is runtime validated, FormData serializable, and structured-cloneable.
  It is not JSON, URL-encoded, HTML, or resume serializable.
- A Form containing a file field must select `serialization: "form-data"`.
- File values, dirty/touched state, and validation results are excluded from a
  resume snapshot. On resume they restart from the empty initial value and all
  Form validation is deterministically recomputed after controls are rebound.
- Other serializable fields in the same Form retain their normal resume slots.
- Forms runtime artifact schema v4 admits the closed `Files` channel,
  `FileArray` normalization, and `File[]` semantic type alongside typed
  validation-rule arguments.

## Failure behavior

The compiler fails closed for a dynamic input type, an unproven or non-array
platform value, a non-empty file initializer, a mismatched binding attribute,
or a non-FormData serialization format. Runtime code never derives file
semantics by scanning the DOM.
