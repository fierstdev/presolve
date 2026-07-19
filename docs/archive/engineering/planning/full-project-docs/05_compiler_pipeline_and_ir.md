# Compiler Pipeline and IR

## Compiler responsibilities

The compiler must do more than transform syntax. It must infer and preserve UI intent.

Responsibilities:

1. Parse authoring syntax.
2. Normalize templates and components.
3. Build semantic graphs.
4. Infer reactivity and ownership.
5. Validate accessibility and platform semantics.
6. Split server/client/event/resource code.
7. Emit target-specific artifacts.
8. Produce source maps and explanation metadata.
9. Support incremental compilation.
10. Feed editor tooling and DevTools.

## Pipeline

```txt
1. Source discovery
2. Parse TS/TSX/html templates
3. Symbol resolution
4. Component normalization
5. Template graph construction
6. Reactive graph construction
7. Resource/action analysis
8. Server/client ownership analysis
9. Accessibility graph validation
10. Style graph analysis
11. Streaming and lazy-loading analysis
12. IR optimization
13. Target emission
14. Explain/debug metadata emission
```

## EdgeZero IR

The compiler should lower source to a framework-specific intermediate representation. The IR should be stable enough for diagnostics and target emitters, but not necessarily public in v1.

Conceptual IR:

```ts
type ComponentIR = {
  id: ComponentId;
  tagName?: string;
  route?: RoutePattern;
  template: TemplateIR;
  state: StateSlotIR[];
  resources: ResourceIR[];
  actions: ActionIR[];
  events: EventIR[];
  styles: StyleIR[];
  a11y: A11yIR;
  ownership: OwnershipIR;
  chunks: ChunkIR[];
  explain: ExplainIR;
};
```

### Template IR

```ts
type TemplateIR = {
  nodes: TemplateNode[];
  bindings: BindingIR[];
  branches: BranchIR[];
  lists: ListIR[];
  slots: SlotIR[];
  staticSubtrees: StaticSubtreeIR[];
};
```

### Binding IR

```ts
type BindingIR = {
  id: string;
  sourceSpan: SourceSpan;
  targetNode: NodeId;
  targetKind: "text" | "attribute" | "property" | "class" | "style";
  reads: SymbolId[];
  updateMode: "text-data" | "set-attribute" | "set-property" | "toggle-class" | "replace-branch";
};
```

### Event IR

```ts
type EventIR = {
  id: string;
  eventName: string;
  targetNode: NodeId;
  handler: SymbolId;
  captures: SymbolId[];
  resumable: boolean;
  lazy: boolean;
  chunk: ChunkId;
  fallback?: FallbackIR;
};
```

### Resource IR

```ts
type ResourceIR = {
  id: string;
  loader: SymbolId;
  params: SymbolId[];
  execution: "server" | "client" | "universal";
  stream: boolean;
  staleTime?: Duration;
  serializable: boolean;
  consumers: BindingId[];
  invalidatedBy: ActionId[];
};
```

### Action IR

```ts
type ActionIR = {
  id: string;
  handler: SymbolId;
  execution: "server" | "client" | "universal";
  formCompatible: boolean;
  optimistic?: boolean;
  captures: SymbolId[];
  invalidates: ResourceId[];
  pendingState: StateSlotId;
  errorState: StateSlotId;
};
```

## Compiler phases in detail

### 1. Source discovery

Inputs:

- project config,
- route files,
- component files,
- server files,
- CSS files,
- package manifests,
- target config.

Output:

- module graph,
- candidate components,
- target capabilities.

### 2. Parse and normalize

Support:

- TSX,
- `html`` templates`,
- decorators,
- class fields,
- CSS modules/scoped CSS,
- route metadata.

Normalization converts authoring syntax to canonical component and template IR.

### 3. Symbol resolution

The compiler resolves:

- class fields,
- state fields,
- methods,
- imported symbols,
- server/client-only APIs,
- resource/action references,
- template identifiers.

### 4. Purity and capture analysis

Needed for:

- derived values,
- event lazy loading,
- resumability,
- serialization,
- server/client split.

Diagnostic example:

```txt
Handler onSubmit cannot be lazy-loaded because it captures non-serializable value `stripeClient`.
Suggestion: initialize Stripe inside a clientOnly block or pass a serializable token.
```

### 5. Accessibility analysis

Runs on the semantic template graph, not emitted HTML alone.

It should understand framework primitives like `Field`, `Form`, `Modal`, `Dialog`, and `Errors`.

### 6. Optimization

Optimizations:

- static subtree hoisting,
- binding-level update plans,
- event-level chunks,
- resource prefetching,
- route-level critical CSS,
- dead CSS removal,
- server-only code pruning,
- client-only code isolation,
- branch-level lazy loading,
- Web Component registration elision.

### 7. Target emission

Emitters should support:

- HTML,
- JS chunks,
- CSS chunks,
- manifest files,
- custom elements,
- server handlers,
- source maps,
- explanation metadata,
- type declarations for published components.

## Explain metadata

Explain metadata should be machine-readable and human-renderable.

```json
{
  "component": "x-counter",
  "source": "src/components/Counter.tsx",
  "state": [
    { "name": "count", "serializable": true, "type": "number" }
  ],
  "bindings": [
    { "id": "b0", "reads": ["count"], "updateMode": "text-data" }
  ],
  "events": [
    { "event": "click", "handler": "increment", "lazy": true, "resumable": true, "chunk": "counter.increment.js" }
  ],
  "clientJs": {
    "initial": "0.8kb",
    "onInteraction": [{ "event": "click", "chunk": "1.1kb" }]
  }
}
```

## Incremental compilation

Incremental compilation must be treated as core infrastructure, not polish.

Invalidation should be graph-based:

- template edit invalidates template graph and affected a11y/style/debug outputs,
- state edit invalidates reactive and serialization graphs,
- action edit invalidates server/client split and lazy chunks,
- CSS edit invalidates style graph and critical CSS,
- route edit invalidates route manifest and affected resources.

## Compiler API

Expose a programmatic API for tools:

```ts
const project = await edgezero.loadProject({ root: process.cwd() });
const result = await project.explain("src/components/Counter.tsx");
console.log(result.bindings);
```

This enables editor plugins, CI tools, design-system analyzers, and AI code assistants.
