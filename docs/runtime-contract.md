# Runtime Contract

EdgeZero compiler output and the browser runtime communicate through the template manifest embedded in `#ez-template-manifest` and emitted as `template.manifest.json`.

## Versioning

- `schema_version` is required at the manifest root.
- The current template manifest schema is `2`.
- Legacy template manifest schema `1` remains accepted only when the effect
  artifact contains no completed-action activation plans.
- The current browser runtime version is `0.0.0` and is exposed as `window.__EDGEZERO__.runtime_version`.
- Runtime state also exposes `window.__EDGEZERO__.supported_schema_version`.

The runtime rejects missing, future, or otherwise unsupported manifest
versions. Schema `2` validates compiler-generated action-batch identities;
legacy schema `1` cannot activate completed-action effects.

Phase H also embeds and emits `component.runtime.json` schema `2`. The runtime
requires that exact version and consumes only compiler-generated definition,
instance, initialization-batch, Slot-binding, instance-Context, and structural
region identities. It performs no tag lookup, Slot-name matching, parent or
Provider search, component discovery, or virtual-DOM diffing. The complete
frozen component boundary is documented in [Component contract](component-contract.md).

## Diagnostics

Runtime diagnostics are exposed as `window.__EDGEZERO__.diagnostics`. Each diagnostic has:

- `code`: stable machine-readable code.
- `message`: developer-facing summary.
- `detail`: structured context.
- `fatal`: `true` when runtime boot cannot continue.

Stable runtime diagnostic codes in schema `1`:

- `EZR_MISSING_MANIFEST`
- `EZR_INVALID_MANIFEST_JSON`
- `EZR_UNSUPPORTED_SCHEMA`
- `EZR_MISSING_ELEMENT_ANCHOR`
- `EZR_MISSING_BINDING_ANCHOR`
- `EZR_UNRESOLVED_EVENT`
- `EZR_UNRESOLVED_ACTION`
- `EZR_INVALID_STATE_OPERATION`

Fatal boot failures set `data-ez-runtime="error"` on the document element and still expose `window.__EDGEZERO__.diagnostics`.
