# Resumability Contract

Phase J freezes Presolve resumability v1: same-build continuation from a
compiler-generated snapshot, without replaying authored initialization, render,
validation, submission, action, or Effect bodies.

## Authority and scope

The compiler is the sole authority for typed resume identities, liveness,
boundaries, activation policies, chunks, codecs, capture/restore programs,
anchors, events, and diagnostics. The browser runtime consumes the exact
manifest and snapshot; it does not reinterpret source, artifacts, or DOM.

Only compiler-authorized State, stable Resource, Computed, Context, Component,
Slot, structural, Form, ordinary-binding, and Effect-establishment products are
eligible. Pending work, closures, functions, DOM nodes, and unsupported value
shapes are excluded and fail closed.

## Artifacts and versions

- Resume manifest: v6, emitted as `resume.runtime.json` and embedded unchanged.
- Resume snapshot: v1, canonical schema-driven JSON.
- Resume runtime protocol and registry contract: v1.
- ASM inspection: v12; check JSON: v6.
- Semantic graph: v6; template manifest: v4; component runtime artifact: v3;
  Context artifact: v2; Forms and Effect artifacts: v1.

`ResumeBuildId` binds executable resume inputs. A snapshot is accepted only for
the exact matching build and supported schema versions; malformed, stale, or
incomplete data rolls back atomically to cold boot.

## Capture, restore, and activation

Capture is permitted only at quiescence and uses closed per-boundary programs
and codecs. Restore follows the fixed R0–R20 schedule: allocate registries,
restore retained values, recompute only authorized Computed slots, establish
Context/Component/Form state and bindings, then mark Ready. Effects are
subscribed without running their bodies during restore.

Anchors (`data-presolve-r`) and events (`data-presolve-e`) contain exact compiler IDs.
The listener finds only the nearest emitted event marker, looks up its closed
manifest entry, and loads the exact deterministic chunk. Failed chunks never
dispatch the action; successful chunks activate once.

Phase K may project an interaction root through a deterministic shared
registration chunk before its root chunk. Shared extraction never merges root
identity, state, or scheduling; shared chunks depend only on eager code, and a
shared import failure retains the same isolated, non-retried activation failure
contract. The packed production artifact preserves the exact Phase J build,
anchor, event, activation, and resume-manifest authority.

## No-discovery and security boundary

The runtime never parses source, inspects names, walks ancestry for ownership,
searches Providers/Forms/controls/actions/bindings/chunks, infers codecs,
constructs semantic or resume IDs, discovers dependencies/boundaries, replays
render, hydrates, diffs a virtual DOM, uses index identity, or serializes
closures/functions/DOM. Exact emitted marker lookup and closed manifest indexes
are the only allowed lookup mechanisms.

Serialized values are closed-schema data, never an authorization mechanism.
Server-only values and pending work remain outside the snapshot; the embedding
environment remains responsible for CSP, escaping, CSRF, and action authority.

## Inspection and diagnostics

Full/selected ASM, text inspection, and check JSON project the same ordered
resume diagnostics. `PSC1096`–`PSC1111` cover unsupported values, missing
owners/programs/chunks, retention/recomputation/policy/order failures, boundary
or chunk cycles, anchors, schema collisions, stable-state violations, artifact
mismatches, unsupported lazy payloads, and excluded topology. Each record uses
an established identity and source evidence when available; no Phase J identity
is fabricated for an unresolved earlier candidate.

## Deliberate exclusions

Visible/Manual activation remains unsupported in v1. Resume does not provide
general hydration, arbitrary JavaScript serialization, semantic discovery,
runtime migration across builds, portal/multi-root/runtime-created topology, or
pending submission resumption.
