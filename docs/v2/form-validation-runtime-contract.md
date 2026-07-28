# V2 Form validation runtime contract

This contract closes the executable browser behavior for Presolve's canonical
built-in and Standard Schema Form rules. Standard Schema identity, publication,
and asynchronous execution are defined in
`form-standard-schema-authority-contract.md`.

## Artifact authority

- Forms runtime artifact schema v6 uses a closed tagged rule vocabulary:
  `none`, `number`, `length`, `pattern`, `field`, and `standard_schema`.
  It also carries the optional compiler-published validator-module path and
  exact sorted validator registry.
- The schema-v4 transition replaced debug-formatted rule arguments
  with a closed tagged vocabulary: `none`, `number`, `length`, `pattern`, and
  `field`.
- A rule record is executable only when its kind, argument tag, target Field,
  and optional dependency agree. The compiler and browser reject mismatches
  before Form initialization.
- Numeric arguments remain canonical finite-number strings in the artifact and
  are converted only after artifact validation. Lengths are non-negative safe
  integers. Patterns must compile as ECMAScript regular expressions.

## Built-in behavior

- `required()` rejects `null`, `undefined`, the empty string, and empty arrays.
- `min()` and `max()` compare finite numeric Field values.
- `minLength()` and `maxLength()` compare string or sequence length.
- `pattern()` evaluates its compiler-validated ECMAScript expression.
- `email()` applies Presolve's closed email-shape predicate.
- `equals()` and `notEquals()` read only their compiler-issued Field
  dependency.
- Non-presence rules accept an absent or empty value; `required()` owns
  presence.
- An unknown rule kind or malformed argument fails closed. It is never treated
  as valid.

## Standard Schema behavior

- Only authority-proven named imports become `standard_schema` rules.
- The browser resolves their compiler-issued IDs from
  `/presolve.validators.js`; application source is never reconstructed.
- Synchronous and Promise-returning results share one issue representation.
- Stale asynchronous generations are discarded.
- Validator output values do not coerce Fields implicitly.
- Pending or invalid validation blocks native submission.

## Input and resume behavior

- An `input` event emitted while an IME composition is active does not commit a
  Field value. The browser's final post-composition input commits normally.
- Binding updates occur before validation and cross-Field invalidation.
- Resume recreates binding and submission-host registries from exact artifact
  records, reinstalls Form listeners, awaits validation, and keeps controls
  interactive.
- File Fields continue to cold-reset and force deterministic Form revalidation
  as defined by `form-file-binding-contract.md`.

## Evidence

The decorator-free V2 Form browser fixture exercises every built-in unary rule,
an asynchronous Standard Schema rule, stale-result suppression, IME
suppression, value/checked/files bindings, native submission, file reset,
snapshot restoration, and an input update after resume.
