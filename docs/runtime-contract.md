# Runtime Contract

EdgeZero compiler output and the browser runtime communicate through the template manifest embedded in `#ez-template-manifest` and emitted as `template.manifest.json`.

## Versioning

- `schema_version` is required at the manifest root.
- The current supported manifest schema is `1`.
- The current browser runtime version is `0.0.0` and is exposed as `window.__EDGEZERO__.runtime_version`.
- Runtime state also exposes `window.__EDGEZERO__.supported_schema_version`.

The runtime accepts only manifests whose `schema_version` exactly matches its supported schema version. Missing, older, or future schema versions are fatal boot errors until an explicit compatibility policy is added.

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
