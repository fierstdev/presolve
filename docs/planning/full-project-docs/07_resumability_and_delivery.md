# Resumability and Delivery

## Phase J prerequisite: ordinary template instance context

Before State retention, ordinary component-template execution is projected by
the compiler from Phase H component instances and canonical template products.
The runtime consumes the exact v4 template-manifest/v3 component-artifact
pair, `data-ez-ti` target markers, and paired text-binding markers. It carries
`RuntimeExecutionContext { component_instance_id, trigger_target_id,
declaration_event_id, action_batch_id }` through ordinary action and binding
execution. This is cold-runtime ownership infrastructure only: it does not
emit resume anchors/events, lazy activation, chunks, snapshots, or J10
markers. J1-A alone replaces declaration-level State storage addressing.

## Phase J prerequisite: instance-qualified Computed runtime slots

J1-C follows J1-P and precedes J1-A. It projects every executable component
instance plus each existing declaration-level E12 computed record into exact
`ComputedInstanceCacheSlotId` and `ComputedInstanceDirtySlotId` values. The
component runtime artifact v3 carries those immutable slots, and the runtime
resolves computed cache reads, writes, and dirtying through the active
`RuntimeExecutionContext.component_instance_id`. Cold boot creates no
fabricated cache value and initializes each dirty slot from E12's compiler
owned initial dirty value. J1-C creates addresses only: it adds no snapshot,
retention, restoration, resume-boundary, lazy-activation, or liveness policy.

J1-A then uses the same execution context to qualify State storage. J2 alone
classifies the already-existing State, computed-cache, and computed-dirty
addresses for resumability.

## Phase J prerequisite: instance-qualified State storage

J1-A projects every executable `(ComponentInstanceId, IrStorageId)` pair into
one typed `StateInstanceSlotId` and immutable registry record. Component
artifact v3 serializes the complete slot, value, type, serialization, and
ownership facts; the browser builds its closed lookup index only from those
records. `storageValues` uses exact instance-slot keys in the Phase J path,
while programs retain definition-level storage operands selected under the
J1-P execution context.

Cold boot initializes each exact slot once. Repeated instances do not share
State, binding updates, or computed invalidation. Component artifact v2 remains
accepted only with the legacy manifest-v3 cold path and no Phase J resume
product. J1-A adds no liveness, retention, snapshot, restore, activation, or
chunk policy; J2 classifies these now-canonical runtime addresses.

## Phase J canonical liveness

J2 creates one compiler-only `ResumeLivenessPlan` over existing runtime
storage. Every exact State slot, Computed cache/dirty slot, instance-selected
Context value slot, Form v5 value/dirty/touched/rule-result/aggregate/submission
slot, and Effect activation-metadata slot appears exactly once as retained,
recomputable, excluded, or blocked.

Mutable serializable State and required Form values are retained. Pure,
deterministic, eagerly scheduled Computed caches are recomputable only when
their exact direct and transitive State/Computed dependencies are themselves
retained or recomputable; otherwise a serializable cache is retained or the
required value is blocked. Context liveness combines the exact Phase H
instance-selected source slot with the canonical Context evaluation source
entry, preserving one shared provider slot for all consumers and carrying its
exact dependency evidence. Effect bodies are never snapshot values; existing
activation metadata is explicitly excluded from value capture.

The plan is deterministic and queryable by existing slot, owner, boundary
candidate, and retention reason. Internal integrity codes `EZASM1320` through
`EZASM1327` reject duplicate classification, missing storage ownership,
unknown dependencies, invalid policy reasons, recomputation without complete
proof, unsupported required values, invalid boundary promotion, and canonical
provenance/order/index drift. J2 changes no public schema and emits no boundary
graph, snapshot, program, marker, chunk, loader, or runtime resume behavior.

## Phase J canonical boundary graph

J3 creates one compiler-only `ResumeBoundaryGraph` from exact Phase H
component-instance, structural-region, ordinary-event, and Phase I Form
products plus J2 liveness. Each build root has one application-root boundary;
each planned or structural-template Component instance, structural region,
Form instance, resumable ordinary event, and enhanced Form submit has its own
unmerged boundary identity.

Ownership parentage is compiler-derived and parent-before-child:
application roots own root Components; Component boundaries own direct nested
Components, structural regions, and Forms; structural regions own their
structural-template Component boundaries. Interaction boundaries are not
ownership parents or children. Their activation references point to exact
owner/required boundaries, existing event or submission programs, and J2
retained slots without duplicating application state.

Blocked Component-instance and J2 liveness products remain explicit
`ResumeBoundaryBlock` records. Internal integrity codes `EZASM1328` through
`EZASM1336` reject duplicate identities, invalid owners, missing/multiple
parents, cycles, unreachable boundaries, nonreciprocal edges, Phase H/I
correspondence drift, provenance drift, and ordering/index drift. J3 changes
no public schema and emits no policy, marker, snapshot, capture/restore
program, chunk, loader, or runtime resume behavior.

## Phase J canonical activation policy

J4 assigns exactly one compiler-only activation decision or explicit block to
every J3 boundary. Application bootstrap/registries/event delegation, immediate
Phase I Form runtime behavior, and required post-restore Computed recomputation
are `Eager`. Exact ordinary-event and enhanced Form-submit roots are
`Interaction` when their required boundaries and J2 retained values are
available. Boundaries with no independent executable work are `None`.

The fixed correctness precedence is `Eager > Visible > Interaction > Manual >
None`. No earlier frozen product authorizes visibility- or manual-driven
activation, so J4 emits zero `Visible` and zero `Manual` policies. Unsupported
lazy event payloads use an explicit eager fallback only when the existing
program remains valid; missing references or blocked prerequisites remain
`ResumeActivationBlock` records. `EZASM1337` through `EZASM1342` validate one
decision per boundary, exact prerequisites, known events/boundaries, source
authority, lazy-payload handling, and deterministic order/indexes. No public
schema or runtime behavior changes.

## Phase J deterministic chunk graph

J5 creates one eager chunk root and one isolated lazy chunk per exact J4
Interaction root. Visible and Manual roots remain empty because J4 emitted no
such policies. Program closure is drawn only from canonical activation
prerequisites and J3 event/Form program references; there is no raw-source
call graph, size splitting, root merging, shared lazy chunk, or lazy-to-lazy
dependency. Module path stems are stable before deterministic content hashing.
Internal integrity codes `EZASM1343` through `EZASM1348` cover duplicate
inclusion, missing programs, dependency cycles, root correspondence, unrelated
programs, and deterministic output drift. No public schema changes.

## Phase J canonical resume schemas

J6 creates exactly one compiler-only `ResumeBoundarySchema` for every J3
boundary. A schema contains only J2 retained and recomputable slots; J2 blocked
slots remain explicit `ResumeSchemaBlock` records and excluded Effect scheduler
metadata is never serialized. Every included slot reciprocally maps its exact
existing runtime address to one `ResumeSlotId`, one canonical semantic type,
and one closed codec.

The codec vocabulary is limited to null, boolean, number, string, homogeneous
array, canonically ordered object properties, and an explicit nullable wrapper.
There is no reflective codec, runtime object walking, or runtime type guessing.
Authored State, Computed cache, Context, and Form Field values derive codecs
from canonical semantic types. Compiler-owned Form dirty/touched/aggregate
slots are boolean, validation results are arrays of strings, and submission
state is string-valued, matching the existing frozen runtime representations.
Unsupported tuples, resources, and non-null unions block instead of acquiring a
generic encoder.

Internal integrity codes `EZASM1349` through `EZASM1354` cover malformed
semantic types, duplicate object properties, unsupported values, missing slot
reciprocity, identity collisions, and canonical ordering/index drift. J6 makes
no public schema or runtime change.

## Phase J generated capture programs

J7 creates one `ResumeCaptureProgram` per boundary in canonical
parent-before-child order. Only J2 retained slots receive instructions;
recomputable slots remain absent from snapshot values. Each retained slot has
one closed `ReadSlot`, `EncodeSlot`, and `AppendValueRecord` triple using the
exact J6 slot ID and codec. There is no generic callback, reflection, property
walking, application mutation, or wait-for-quiescence loop.

Internal snapshot model v1 uses an application-atomic envelope with fixed
manifest version 6, `capturedAt: null`, build-derived snapshot identity,
canonical boundary/value order, and compact canonical JSON. Standalone
artifact encoding adds exactly one trailing newline; embedded encoding does
not. The generated encoder accepts only the closed null/boolean/number/string/
array/object/nullable shapes, writes object properties in schema order, and
rejects non-finite numbers, negative zero, missing/extra object properties,
and runtime shape guessing.

Capture proceeds only when the full compiler-defined quiescence vector is
clear. Pending or unknown Form submission state is rejected; stable states are
`Idle`, `Invalid`, `Failed`, and `Completed`. Internal integrity codes
`EZASM1355` through `EZASM1358` cover program correspondence, instruction
shape, capture-envelope policy, and ordering/output drift. J7 does not expose
snapshot build output or add browser runtime behavior.

## Phase J generated restore programs

J8 creates one `ResumeRestoreProgram` per J3 boundary and the complete fixed
R0-R20 application schedule. Boundary allocation and program order preserve
J3 parent-before-child order. Every J2 retained or recomputable slot receives
one explicit phase assignment; retained values decode through their J6 codec,
while recomputable Computed cache/dirty pairs are omitted from snapshot decode
and regenerated once by the exact compiler-planned evaluator at R5.

Mutable State restores at R3, retained Computed cache state at R4, Context
Provider values at R6, exact Context Consumer bindings at R7, Form Field values
at R11, dirty/touched state at R12, rule/aggregate validation at R13, and stable
submission state at R14. Each program allocates its boundary record at R2 and
ends with `MarkBoundaryRestored` at R19. The closed instruction vocabulary has
no authored callback, constructor, initializer replay, render call, validator,
submit handler, or Effect-body execution.

J8 deliberately emits no DOM-binding install instruction before J10 supplies
canonical anchor identities and no Effect-subscription instruction before the
later runtime-establishment slice. Those fixed schedule phases remain present
and empty rather than fabricating references. Internal integrity codes
`EZASM1359` through `EZASM1362` cover dangling program references, wrong
phases/duplicate writes, missing completion, and ordering/output drift. No
public schema or runtime behavior changes.

## Phase J executable resume manifest

J9 advances the sole resume manifest authority from v5 to v6 and publishes the
J1-J8 boundary, liveness, schema, capture, restore, chunk, activation, and
normalized Phase I resume records. Snapshot schema v1 and runtime protocol v1
are now public version fields. V5 is rejected at the v6 runtime boundary; no
compatibility adapter or second planning authority remains.

The standalone `resume.runtime.json` artifact uses compact canonical JSON with
one trailing newline. The generated page embeds those exact bytes in
`#ez-resume-runtime`, without reformatting or reserializing them. The parser
rejects unknown fields, version drift, malformed shapes, duplicate identities,
and every unresolved cross-reference before runtime consumption.

`ResumeBuildId` is SHA-256 lowercase hexadecimal over framed canonical bytes
for every executable runtime artifact, the v6 manifest with its build ID set to
the fixed zero sentinel, normalized eager/lazy chunk bytes, the anchor/event
marker plan, runtime protocol v1, and snapshot schema v1. Absolute source-root
prefixes, provenance/spans, wall-clock time, diagnostics, output directory,
and machine information do not influence the fingerprint. Executable changes
do. Repeated and reversed builds remain byte-identical.

## Phase J exact resume anchors and event markers

J10 freezes one compiler-owned marker plan derived from ordinary
instance-qualified template targets plus the J3/J5 boundary and chunk
authorities. Element, Form-control, and event targets use exact ID-only
`data-ez-r` and `data-ez-e` attributes. Dynamic text uses a zero-layout
`<template data-ez-r>` marker. Existing conditional and keyed-list comment
ranges receive exact `ez-r-start`/`ez-r-end` pairs; no second structural
representation exists.

Every public manifest anchor and event has exactly one emitted page marker.
Static-only output emits none. Marker validation rejects missing or unstable
targets, duplicate anchors, wrong kinds, structural-pair mismatch, and
noncanonical ordering/output. The template manifest remains unchanged; resume
manifest v6 is the sole Phase J marker authority.

## Phase J resume runtime registry and bootstrap

J11 adds runtime Resume registry contract v1. The eager bootstrap validates
the existing runtime artifacts plus resume manifest v6 before selecting a
path. An absent snapshot is a normal cold boot. A structurally valid,
same-build snapshot allocates closed, ID-keyed definition and runtime
registries without running State initializers, Component initialization,
Context sources, Forms, or Effects.

Snapshot parse/schema/build/protocol and artifact failures discard the entire
candidate registry before invoking the existing cold path exactly once. No
partially allocated boundary, slot, Context, Component, Form, structural,
Effect, or activation state survives. A second bootstrap is rejected.
Development evidence exposes only deterministic version, mode, failure,
build-ID, boundary-ID, and slot-ID facts.

The internal runtime API now provides `bootstrapResume`, `captureSnapshot`,
`activateByEvent`, and `activateBoundary`. J11 freezes their registry and
identity boundary; later slices execute restore/capture and lazy activation
programs through those APIs.

The J11 readiness audit also repaired the preexisting ordinary cold path:
generated strict-mode JavaScript no longer binds the reserved `arguments`
identifier, instance State correlates compiler records through the manifest
definition name, text-binding targets join the exact ordinary target index,
and numeric instance-slot initial values enter storage as numbers. The focused
probe constructs its success marker only after runtime assertions, so source
text cannot produce a false pass.

## Phase J State, Resource, and Computed restoration

J12 executes only the compiler-authored R3-R5 restore instructions. R3 decodes
and writes mutable State plus canonically retained Resource slots through their
J6 codecs. R4 installs retained Computed cache/dirty pairs. R5 invokes only
J2-approved recomputable Computed programs, once per exact instance-qualified
slot and in manifest topological order.

The resume path allocates boundary, component, State, and Computed runtime
records directly from closed manifest identities. Restored slots never execute
their authored initializer or evaluator, and no Effect body runs. A missing,
duplicate, malformed, or codec-incompatible snapshot value rejects the whole
candidate registry and selects one clean cold boot; partial writes cannot
escape.

Repeated component instances retain distinct State and Computed addresses.
The browser proof restores different State values into two instances,
recomputes their dependent Computed values exactly once, and confirms the
caches are clean and isolated before Ready. Later J16-J17 slices own resumed
DOM subscription establishment and future action delivery; J12 does not enter
those authorities early.

## Phase J Context restoration

J13 executes R6-R7 without evaluating an initial Context source. R6 decodes
each retained Provider or Context-default value through its J6 codec and
writes the exact instance-qualified Context slot named by the restore program.
R7 installs the emitted `ConsumerInstanceId` to exact selected source,
optional `ProviderInstanceId`, and value-slot relation.

The runtime cross-checks every installed relation against the frozen Component
runtime artifact. It does not select a Provider, traverse component ancestry,
look up a Context name, or derive an identity. Missing, duplicate, or divergent
relations reject the complete resume attempt; no partial Context registry
survives the cold fallback.

The browser proof restores distinct default, outer Provider, and nearest
override values for nested component instances. Each Consumer observes only
its compiler-selected exact slot, while Context initial evaluators and Effects
remain suppressed. The existing Phase G action-update batches remain the sole
future propagation order; J17 owns delivery of the first resumed interaction.

## Phase J Component, Slot, and structural restoration

J14 executes R8-R10 from closed Phase I records. It installs Component runtime
records and caller-owned Slot bindings directly by their compiler-emitted
instance and binding IDs, without constructors, render methods, tag matching,
Slot matching, parent traversal, or index-derived identity. Structural regions
restore their selected conditional/keyed-list State from the exact retained
State slot selected by the compiler; a value divergent from the already
rendered structural DOM rejects the candidate rather than reconstructing it.

Anchor validation resolves only emitted `data-ez-r` attributes and exact
`ez-r-start`/`ez-r-end` comment pairs. Resume never mutates the DOM for this
phase. A mismatch or missing anchor discards the entire registry and performs
one ordinary cold boot. The cold ordinary-runtime target index now recognizes
the existing compiler-emitted structural comment ranges, so that fallback
remains a valid Phase H continuation path.

## Definition

In EdgeZero, resumability means:

> The server can render meaningful HTML and serialize enough semantic state for the browser to continue specific interactions without replaying the whole component tree or downloading all application logic upfront.

Resumability is not a user-facing syntax ceremony. It is a compiler and runtime capability.

## Default delivery policy

1. Render HTML on the server where possible.
2. Include no component JavaScript for static regions.
3. Include a tiny event/resume loader for interactive regions.
4. Load interaction code only when the user interacts or prefetching is justified.
5. Patch exact bindings after state changes.
6. Preserve native form/link fallback.

## What is serialized

Serialize only what is needed:

- state slots required for resumable interaction,
- resource snapshots safe for the client,
- binding/component IDs,
- action/event references,
- route params where needed,
- form pending/error state where needed.

Do not serialize:

- database handles,
- secrets,
- server-only environment values,
- arbitrary closures,
- full component instances unless target requires it,
- data not consumed by client-resumable interactions.

## HTML as continuation format

HTML should carry enough markers to resume interaction, but those markers must be minimal and inspectable.

Example:

```html
<form data-ez-c="checkout-form" data-ez-action="a0" action="/actions/checkout">
  <button data-ez-bind="b4">Pay</button>
</form>
<script type="application/ez-state" nonce="...">
  {"c0":{"submit.pending":false}}
</script>
```

The exact marker syntax should be optimized later. Requirements:

- small,
- CSP-compatible,
- stream-friendly,
- stable enough for DevTools,
- not required in no-JS static-only output.

## Event resumability

Compiled event flow:

```txt
1. User clicks element.
2. Runtime finds event marker.
3. Runtime resolves handler chunk.
4. Runtime loads chunk.
5. Runtime resumes required state slots.
6. Handler runs.
7. Signal graph invalidates exact bindings.
8. DOM patcher updates nodes.
```

The author should not write resumability markers manually.

## Server actions

Server actions provide a controlled mutation boundary.

Requirements:

- native form POST fallback,
- enhanced fetch/WebSocket submission where configured,
- CSRF protection integration,
- validation integration,
- redirect handling,
- streamed errors where possible,
- invalidation of resource graph,
- optimistic update hooks,
- no accidental client bundling of server-only imports.

## Progressive enhancement

Every form and link should start as real HTML.

```tsx
<form action={this.save} method="post">
  <input name="email" type="email" required />
  <button>Save</button>
</form>
```

Compilation modes:

- no JS: native form POST,
- basic JS: enhanced fetch submit,
- resumable: lazy action handler and pending/error patches,
- streaming: validation/result fragments stream back into regions,
- live: server-driven updates where target supports it.

## Interactivity boundary inference

The compiler should infer interactive regions from:

- event handlers,
- client-only APIs,
- mutable client state,
- form enhancement,
- browser resources,
- custom-element export requirements.

Manual boundary annotations should exist only for override cases:

```tsx
<Chart clientOnly />
<ExpensivePanel eager />
<StaticMarketingBlock noClient />
```

## Chunking strategy

Chunk by user-visible interaction where possible.

Examples:

```txt
Initial /checkout
  loader.js
  checkout.css
  HTML document

On click "Apply coupon"
  coupon.apply.js

On submit checkout form
  checkout.submit.js
  payment.validation.js
```

The `edgezero size --by-interaction` command should make this visible.

## Streaming

Streaming is a first-class target behavior.

Authoring:

```tsx
<Await resource={this.recommendations} fallback={<Skeleton />}>
  {items => <Recommendations items={items} />}
</Await>
```

Compiler inference:

```txt
region recommendations
  can flush placeholder immediately
  resource can stream
  error boundary: nearest ErrorBoundary
  client reorder: not required
```

## Failure behavior

A serious delivery model must define failure modes.

### JavaScript disabled

- HTML remains usable where possible.
- Forms post natively.
- Links navigate natively.
- Nonessential client widgets degrade visibly.

### Chunk load failure

- Retry according to policy.
- Surface error to nearest boundary.
- Preserve native fallback where possible.
- Log diagnostic in development.

### Serialization failure

- Compilation fails when statically provable.
- Runtime fails closed when dynamic and unsafe.
- Diagnostic names captured value and boundary.

### Network failure during action

- Pending state resolves to error.
- Optimistic state rolls back if configured.
- Form data is preserved where possible.

## Security constraints

Resumability must not become a serialization vulnerability.

Rules:

1. Server-only values are never serialized.
2. Action IDs are not authorization.
3. CSRF protections are integrated by target adapter.
4. Serialized state is escaped and CSP-compatible.
5. Resource snapshots require explicit public exposure.
6. Dev-only metadata is stripped in production unless opted in.

## Target-specific behavior

### Static target

- Generate static HTML and CSS.
- No server actions.
- Optional client interactions.

### SSR target

- Generate HTML per request.
- Serialize state where needed.
- Actions use server adapter.

### Streaming SSR target

- Support async regions.
- Flush early.
- Preserve fallback/error semantics.

### Resumable web target

- Include resume loader.
- Event handlers lazy by default.
- No full hydration baseline.

### Web Component library target

- Generate custom elements.
- Define component API manifest.
- Runtime includes custom-element upgrader where needed.
