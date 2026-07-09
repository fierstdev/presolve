# Forms, Resources, and Actions

## Why this matters

Forms and data loading are where many frameworks leak complexity. EdgeZero should treat them as semantic compiler primitives, not patterns users assemble from generic state, fetch, and validation libraries.

## Forms goals

1. Native form fallback by default.
2. Accessible labels, errors, and descriptions by construction.
3. Server actions and client enhancement from one authoring model.
4. Schema validation support without hard-locking to one schema library.
5. Pending/error/success state as compiler-visible state.
6. Optimistic updates where configured.
7. Resource invalidation linked to actions.
8. Strong DevTools/explain output.

## Native form example

```tsx
@route("/users/:id")
class UserPage extends Page {
  user = resource(({ params }) => getUser(params.id));

  save = action(async form => {
    "server";
    await updateUser(this.user.id, form);
  });

  render() {
    return (
      <form action={this.save}>
        <label>
          Email
          <input name="email" type="email" value={this.user.email} required />
        </label>
        <button disabled={this.save.pending}>Save</button>
      </form>
    );
  }
}
```

Compiler responsibilities:

```txt
native fallback: POST /actions/user-page.save
enhanced submit: yes
pending binding: button.disabled reads save.pending
validation: native email + required
accessibility: input has label
resource invalidation: user invalidated by save if configured/inferred
```

## Structured form API

```tsx
profile = form(UserSchema, {
  load: () => this.user,
  submit: this.save,
  optimistic: true,
  invalidate: [this.user]
});
```

```tsx
<Form for={this.profile}>
  <Field name="email" />
  <Field name="role" as="select" />
  <Errors />
  <button disabled={this.profile.pending}>Save</button>
</Form>
```

The structured API is for complex forms. It should not be required for simple forms.

## Field compiler behavior

`<Field name="email" />` should compile to:

- label if schema supplies display label or author provides one,
- input with correct type where inferable,
- `name`, `id`, and `autocomplete` where configured,
- associated error region,
- aria attributes when invalid,
- pending/dirty/touched metadata if used,
- native validation attributes where possible.

## Schema adapters

Do not build a proprietary validation island. Provide adapters:

```ts
form(zodSchema, options)
form(valibotSchema, options)
form(jsonSchema, options)
form(customValidator, options)
```

Compiler-visible schema metadata should include:

- field names,
- required/optional status,
- primitive types,
- enum options,
- min/max/length constraints,
- labels/descriptions if supplied,
- server-only validation markers.

## Action model

Actions are mutation boundaries.

```ts
save = action(async form => {
  "server";
  const input = UserSchema.parse(form);
  await db.users.update(input.id, input);
  return { ok: true };
});
```

Compiler should infer:

- server execution,
- form compatibility,
- serialized result shape,
- invalidated resources,
- pending state,
- error state,
- fallback route.

## Action return types

Recommended action result shape:

```ts
type ActionResult<T = unknown> =
  | { ok: true; data?: T; redirect?: string; invalidate?: ResourceRef[] }
  | { ok: false; fieldErrors?: FieldErrors; formError?: string; status?: number };
```

The compiler can also adapt thrown redirects and framework-native response objects.

## Resource model

```ts
user = resource(({ params }) => getUser(params.id), {
  key: ({ params }) => ["user", params.id],
  staleTime: "5m",
  stream: true
});
```

Compiler should track:

- execution location,
- cache key,
- dependencies,
- consuming bindings,
- stream eligibility,
- serialization shape,
- invalidation sources,
- prefetch eligibility.

## Resource states

Resources expose states:

```txt
pending
ready
error
refreshing
stale
```

But templates should remain ergonomic:

```tsx
<h1>{this.user.name}</h1>
```

The compiler can require explicit handling when a resource can be absent:

```txt
Resource `user` may be pending in client-only target.
Wrap in <Await> or provide a fallback.
```

## Cache invalidation

Actions should invalidate resources automatically when possible and explicitly when needed.

```ts
save = action(updateUser, {
  invalidates: [this.user, this.posts]
});
```

Potential inference:

- action imports `updateUser`, resource imports `getUser`, same key family by convention,
- schema or adapter declares invalidation,
- route action modifies current route resource.

Do not overpromise magical invalidation. Explain the inference and require explicit declarations when uncertain.

## Optimistic updates

```ts
save = action(updateUser, {
  optimistic(form) {
    this.user.name = form.get("name");
  },
  rollback: true
});
```

Compiler requirements:

- mark optimistic state as client-owned until confirmed,
- rollback on failure,
- explain affected bindings,
- warn if optimistic update touches non-serializable or server-only state.

## Accessibility requirements for forms

Compiler must validate:

- every field has an accessible name,
- errors are associated with fields,
- form-level errors are announced where needed,
- required fields communicate required state,
- disabled/pending states remain perceivable,
- custom controls provide keyboard behavior,
- invalid ARIA usage fails or warns.

## `edgezero explain` form output

Example:

```txt
Form: checkout-form
Action:
  submit -> server action checkout
Fallback:
  native POST /actions/checkout
Enhancement:
  fetch submit, lazy chunk checkout.submit.js
Fields:
  email: type=email, required, label=Email, error region=e0
  address: type=text, required, label=Address, error region=e1
Pending state:
  submit.pending -> button.disabled, button text
Accessibility:
  no issues
Resource invalidation:
  cart invalidated on success
```
