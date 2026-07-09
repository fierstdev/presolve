# Authoring Model

## Goals

The authoring model should feel close to ordinary web UI code:

- TypeScript/TSX support.
- `html`` template support.
- Class-based components.
- Decorators or explicit helpers for capabilities.
- Signals/state without accessor noise in templates.
- Resources and actions colocated with UI.
- Web Components as a first-class output target.
- HTML forms, links, and semantics treated as real primitives.

## Component forms

### Class component

```tsx
@component("x-counter")
class Counter extends Component {
  @state count = 0;

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
```

Compiler inference:

```txt
state count: serializable number
binding b0 reads count
click handler increment: client-resumable, lazy-loadable
output target: x-counter custom element if requested
```

### Template component

```ts
@component("x-counter")
class Counter extends Component {
  @state count = 0;
  increment() { this.count++; }

  render = html`
    <button @click=${this.increment}>Count: ${this.count}</button>
  `;
}
```

The template syntax should not be a second-class path. It should produce the same semantic graph as TSX when possible.

## State

State should be explicit enough for the compiler, but not noisy.

```ts
@state editing = false;
@state count = 0;
```

Avoid requiring every read to use `count()` or `count.value` inside templates. The compiler can lower class fields to signals and preserve a normal authoring surface.

Rules:

1. Template reads of `@state` fields are reactive.
2. Methods mutating `@state` fields create invalidation edges.
3. Non-serializable state must be marked explicitly.
4. Deep mutation is either disallowed by default or requires explicit wrappers.

Recommended policy:

- Primitive and immutable object state are default-safe.
- Deep mutable state requires `state.object()` or `mutable()` so the compiler can explain tracking costs.

## Derived state

```ts
get fullName() {
  return `${this.first} ${this.last}`;
}
```

Compiler should detect derived reads and memoize/invalidate if the getter is pure enough. If not provable, explain the limitation.

Alternative explicit form:

```ts
fullName = derived(() => `${this.first} ${this.last}`);
```

## Resources

Resources declare data dependencies:

```ts
user = resource(({ params }) => getUser(params.id));
posts = resource(() => getPosts(this.user.id), { stream: true, staleTime: "5m" });
```

The compiler should know:

- where it can run,
- whether it can stream,
- what invalidates it,
- which bindings consume it,
- whether snapshots are serializable.

## Actions

Actions declare mutations and server/client boundaries:

```ts
save = action(async form => {
  "server";
  await updateUser(this.user.id, form);
  this.editing = false;
});
```

or:

```ts
@server
async save(form: FormData) {
  await updateUser(this.user.id, form);
}
```

Policy:

- Server-only code should be inferred from imports where possible.
- Explicit annotations should exist for clarity and boundary hardening.
- Illegal captures should fail compilation with direct explanations.

## Routes

```tsx
@route("/users/:id")
@component("user-profile")
class UserProfile extends Page {
  user = resource(({ params }) => getUser(params.id));

  render() {
    return <h1>{this.user.name}</h1>;
  }
}
```

Route declarations should be compile-time visible. Filesystem routing can be supported, but decorator route declarations provide stronger locality and inspectability.

## Forms

Native form authoring must work:

```tsx
<form action={this.save}>
  <label>
    Name
    <input name="name" value={this.user.name} />
  </label>
  <button>Save</button>
</form>
```

Enhanced forms should be optional:

```tsx
profileForm = form(UserSchema, {
  load: () => this.user,
  submit: this.save,
  optimistic: true
});
```

```tsx
<Form for={this.profileForm}>
  <Field name="email" />
  <Field name="role" as="select" />
  <Errors />
  <button disabled={this.profileForm.pending}>Save</button>
</Form>
```

The compiler should generate accessible labels/errors, native fallback, enhanced submit, pending state, and server/client split.

## Control flow

Support TSX-native control flow and optional framework primitives.

```tsx
{this.editing ? <Editor /> : <button>Edit</button>}
```

```tsx
<Show when={this.editing} fallback={<button>Edit</button>}>
  <Editor />
</Show>
```

The compiler can treat `Show`, `For`, `Await`, and `ErrorBoundary` as semantic primitives rather than runtime-only components.

## Async rendering

```tsx
<Await resource={this.posts} fallback={<PostsSkeleton />}>
  {posts => <PostList posts={posts} />}
</Await>
```

Async primitives should compile into streaming regions where the target supports it.

## Escape hatches

Required escape hatches:

```ts
opaque(() => externalLibraryCall())
clientOnly(() => import("./Chart"))
serverOnly(() => readFile("./data.json"))
noResume(this.expensiveHandler)
eager(this.onDragMove)
```

Escape hatches must appear in explain output.

## Authoring constraints

The authoring model should avoid:

- hooks-style invisible ordering constraints,
- mandatory dependency arrays,
- pervasive `.value` or function-call signal access in templates,
- hidden deep proxy behavior,
- manual island boundaries as the default,
- mandatory `use client`/`use server` file splitting,
- compiler magic that cannot be inspected.
