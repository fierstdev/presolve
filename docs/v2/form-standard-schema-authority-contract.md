# V2 Form Standard Schema authority contract

This contract establishes the compile-time authority boundary for Standard
Schema v1 validators used by decorator-free `defineForm` Fields. Executable
module bundling and browser scheduling remain the next gate.

## Admitted authoring form

- A validator is a named import used directly in a Field's `validate` array.
- TypeScript must prove that the imported value exposes `~standard.version`
  equal to `1`, a string `vendor`, and a callable `validate` member.
- When the optional Standard Schema `types` witness is present, its input must
  be compatible with the canonical Field value type. Native TypeScript
  diagnostics fail the authority request before lowering.
- Local objects, inline schema construction, default imports, namespace member
  access, and values that only resemble the protocol are not admitted. The
  compiler does not evaluate source or infer protocol membership from names.

## Exact module identity

V2 TypeScript-authority schema v9 returns the parser-selected site joined to:

- the authored module specifier;
- the named export;
- the resolved declaration-module vector; and
- TypeScript's input and output type displays when available.

The compiler validates the response against the original request, records the
evidence in canonical authored semantics schema v5, and retains the exact
coordinate on the Form validation candidate. An authority response may not
change the module specifier, export, or source site.

## Fail-closed boundary

The compiler does not copy a validator function, serialize its source, invoke
it during compilation, or use `eval`/`Function` in the browser. Until the
runtime module is bundled from the original named export and published with an
integrity-bound location, the retained candidate emits `PSC1087` and cannot
become an executable validation rule. This makes missing runtime linkage
visible rather than silently omitting the authored validator.

## Next gate

The runtime gate must bundle the original named export, support synchronous and
Promise-returning Standard Schema results, normalize issues without treating a
transformed value as implicit Field coercion, suppress stale asynchronous
results, block submission while validation is pending or invalid, and prove
cold plus resumed behavior in a real browser.
