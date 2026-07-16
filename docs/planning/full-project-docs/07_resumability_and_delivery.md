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
