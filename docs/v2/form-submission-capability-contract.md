# V2 Form submission capability contract

This contract admits one persistence boundary for decorator-free
`defineForm()` without turning submit handlers into interpreted JavaScript.
The compiler retains one exact source call, resolves one integrity-bound client
capability, publishes one callable registry record, and owns its lifecycle.

## Admitted authoring form

```tsx
import { saveProfile } from "profile-service";

profile = defineForm({
  fields: {
    name: field({ initial: "" }),
  },
  submit: async ({ value, signal }) => saveProfile(value, signal),
});
```

The submit member must be an async, single-parameter inline handler whose
entire body is one direct call to a named import. The call arguments must be
exactly `value` and `signal` in that order. The existing inline State-update
subset remains admitted independently. Member calls, namespace/default
imports, ambient globals such as `fetch`, extra statements, captures, argument
reordering, and arbitrary source execution remain excluded.

The parser retains only direct-call syntax and exact spans. It does not assign
capability meaning. Binding/package resolution must prove the named import and
closed package export before an executable submission record exists.

## Package contract

The named export has semantic-package kind `capability` and must carry:

```json
{
  "type_signature": "(FormValue, AbortSignal) -> Promise<void>",
  "resume_policy": "cold_fallback",
  "form_submission": {
    "execution_boundary": "client",
    "cancellation": "abort",
    "input": "form_value",
    "result": "void"
  }
}
```

Package name, version, integrity, export, and runtime module are exact
compiler inputs. Package source is never inspected. A missing or mismatched
contract/runtime module fails publication.

## Lifecycle

- Validation completes successfully before capability invocation.
- The first argument is the canonical nested Form value built from
  compiler-issued Field paths, never a DOM scan.
- Each accepted submission receives a fresh compiler-owned `AbortController`.
- The signal is aborted by Form reset, component teardown, or page lifecycle
  disposal. A second submit while the same Form is `Submitting` is ignored; it
  does not implicitly cancel or duplicate the active call.
- Fulfillment moves the Form to `Completed`; rejection moves it to `Failed`
  unless the signal was aborted, in which case it moves to `Cancelled`.
- `Submitting` is never resumed. Existing snapshot validation falls back
  cold for an active submission.

## Publication and proof

Forms artifact schema v7 publishes the exact client capability coordinate and the
closed `/presolve.form-submissions.js` registry path. The ergonomic builder
uses project Vite to bundle only the contract-selected runtime module and named
export, includes the output in the digest inventory, and rejects missing or
drifting records.

Real-browser evidence covers fulfillment, rejection, reset-driven
cancellation, duplicate-submit suppression, canonical nested values including
files, cold boot, and resumed submission. Identical builds produce identical
Forms artifacts, capability bundles, and publication manifests.
