# V2 Form Standard Schema authority and runtime contract

This contract establishes the compile-time and browser boundary for Standard
Schema v1 validators used by decorator-free `defineForm` Fields. The compiler
retains exact TypeScript identity, Vite bundles the original named exports, and
the browser executes only the validator registry named by the Forms artifact.

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

## Runtime publication

The compiler does not copy a validator function, serialize its source, invoke
it during compilation, or use `eval`/`Function` in the browser. For an
ergonomic project build, the CLI generates an internal entry module that
imports only the authority-proven named exports and asks the project's Vite
installation to bundle them. Vite is a direct scaffold dependency, not an
undeclared global tool.

The resulting `presolve.validators.js` is published through the same
file-route inventory and digest calculation as other static artifacts. Forms
runtime artifact schema v5 carries the exact absolute publication path and the
sorted validator IDs required by the route. A missing Vite installation,
unresolvable source module, missing export, failed bundle, missing registry
export, or protocol mismatch fails the build or browser boot. There is no
silent validator omission or source-evaluation fallback.

## Browser execution

- The runtime imports the compiler-published module before allocating Forms.
- Every required validator ID must exist and expose Standard Schema v1.
- `validate` may return synchronously or return a Promise.
- Issues are normalized to stable message/path records. Thrown or malformed
  results fail validation closed.
- A successful transformed `value` is not implicit Field coercion. The
  canonical Field retains the value committed through its binding.
- Each Field owns a monotonically increasing validation generation. A stale
  Promise cannot overwrite a newer value's result.
- Submission awaits current validation and remains blocked while any Field is
  pending or invalid.
- Resume restores serializable Field state, cold-resets non-resumable file
  Fields, and awaits revalidation before publishing runtime readiness.

## Evidence

The decorator-free V2 Form browser fixture bundles an authority-proven local
TypeScript schema and proves cold plus resumed execution. It covers
synchronous built-ins alongside an asynchronous Standard Schema validator,
out-of-order Promise completion, issue normalization, non-coercion, native
submission, file reset/revalidation, and post-resume interactivity. The probe
uses a runtime-created marker, so source text cannot satisfy the browser gate.
