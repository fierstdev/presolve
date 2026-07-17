# Runtime Contract

EdgeZero compiler output and the browser runtime communicate through the template manifest embedded in `#ez-template-manifest` and emitted as `template.manifest.json`.

## J1-A State instance storage

In the template-manifest-v4/component-artifact-v3 path, `storageValues` is
keyed only by the exact compiler-emitted `StateInstanceSlotId`. Component
artifact records provide both the complete slot ID and the closed
`(component_instance_id, storage_id)` index. Runtime State programs retain
definition-level `IrStorageId` operands, but every read or write resolves that
operand under `RuntimeExecutionContext.component_instance_id` before touching
the map.

Cold boot initializes every serialized State slot exactly once. Actions,
ordinary bindings, computed invalidation, Context/effect programs, Forms
bridges, and later resume operations may not use State names, component names,
DOM ancestry, map order, runtime counters, or declaration-level storage IDs as
runtime keys. A missing, duplicate, malformed, stale, or cross-instance slot
is a fatal artifact-integrity error.

The manifest-v3/component-artifact-v2 pair remains a legacy cold-boot
compatibility path only. Manifest v4 rejects component artifact v2, and no
Phase J resume product may use that legacy pair.

## J1-C computed instance slots

For the ordinary template manifest v4/component artifact v3 path, the browser
runtime registers compiler-emitted computed slot records before evaluation.
`computedCaches` is keyed by `ComputedInstanceCacheSlotId` and computed dirty
state by `ComputedInstanceDirtySlotId`; a declaration-level computed ID only
selects the exact compiler-emitted slot under
`RuntimeExecutionContext.component_instance_id`. Duplicate slot IDs, duplicate
`(component instance, computed)` projections, malformed ownership prefixes, or
missing projections are artifact-integrity failures and are not repaired.

Cold boot evaluates only compiler-planned dirty slots. A cache has no made-up
value before that evaluation; each dirty slot starts from the existing
compiler-owned E12 initial value. This does not create a resume policy,
snapshot record, retained-slot classification, or lazy activation behavior.

## Versioning

- `schema_version` is required at the manifest root.
- The current template manifest schema is `4`.
- The current component runtime artifact schema is `3`.
- The exact Phase J cold runtime pair is template manifest v4/component
  artifact v3.
- Template manifest v3/component artifact v2 remains accepted only as the
  legacy cold-boot pair without a Phase J resume product or snapshot.
- The current browser runtime version is `0.0.0` and is exposed as `window.__EDGEZERO__.runtime_version`.
- Runtime state also exposes `window.__EDGEZERO__.supported_schema_version`.

The runtime rejects missing, future, mixed, or otherwise unsupported manifest
and component-artifact versions. It consumes only compiler-generated
definition, instance, State/computed slot, initialization-batch, Slot-binding,
instance-Context, ordinary-template, and structural-region identities. It
performs no tag lookup, Slot-name matching, parent or Provider search,
component discovery, or virtual-DOM diffing. The complete frozen component
boundary is documented in [Component contract](component-contract.md). Phase J
resume uses the exact manifest-v6/snapshot-v1 registry protocol and closed
anchor/event indexes; its same-build, no-discovery, failure, schema, and
diagnostic rules are frozen in the
[Resumability contract](resumability-contract.md).

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
