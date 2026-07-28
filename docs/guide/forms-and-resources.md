# Forms and resources

Forms, resources, and their browser artifacts are compiler-owned capabilities.
Use their declarations only where their ownership and endpoint behavior are
clear to the compiler.

## Forms

Use `defineForm()` for new applications. Its schema is statically recoverable,
each leaf is declared with `field()`, and controls bind to the resulting Field
objects. No decorator is involved.

```tsx
import {
  Component,
  defineForm,
  email,
  field,
  required,
  state,
} from "presolve";

export class ProfileForm extends Component {
  saved = state(0);

  profile = defineForm({
    serialization: "form-data",
    fields: {
      identity: {
        name: field({
          initial: "",
          validate: [required()],
        }),
        email: field({
          initial: "",
          validate: [required(), email()],
        }),
      },
      newsletter: field({ initial: false }),
      attachments: field<File[]>({
        initial: [],
        validate: [required()],
      }),
    },
    submit: async ({ value, signal }) => {
      // The current inline beta subset can update canonical State.
      // Capability-backed persistence is the next submission gate.
      this.saved += 1;
    },
  });

  render() {
    return <form form={this.profile}>
      <input bind:value={this.profile.fields.identity.name} />
      <input type="email" bind:value={this.profile.fields.identity.email} />
      <input
        type="checkbox"
        bind:checked={this.profile.fields.newsletter}
      />
      <input
        type="file"
        multiple
        bind:files={this.profile.fields.attachments}
      />
      <button type="submit">Save profile</button>
      <output>{this.saved}</output>
    </form>;
  }
}
```

`bind:value` commits text and supported scalar controls, `bind:checked` commits
checkbox state, and `bind:files` commits a `File[]` on `change`. File fields
require `serialization: "form-data"`. Native file values cannot be restored
from a snapshot, so Presolve resets and revalidates that Field on resume while
retaining the other serializable Fields.

The canonical V2 path has completion evidence for `required`, numeric
`min`/`max`, string or sequence `minLength`/`maxLength`, `pattern`, and
`email`. Validation runs after a binding update and again before submission.
Cross-field rules and Standard Schema-compatible validators remain separate
beta gates and are not presented as completed V2 behavior. Browser validation
is advisory; a server boundary must validate untrusted data again.

The beta currently executes inline submission bodies made from admitted
canonical State updates. Imported capability calls, Standard Schema adapters,
and server-side revalidation are tracked as completion gates and fail closed
instead of being ignored.

## Legacy Form compatibility

`@form()` declares a component-owned form. `@field()` associates a stateful
field with it, and `@submit()` declares the action that handles its submission.
This decorator syntax remains an alpha-compatibility capability and is not
emitted by the V2 scaffold.

```tsx
import {
  action, component, field, form, serialize, submit, Component, type Form,
} from "presolve";

@component()
export class ProfileForm extends Component {
  @form() @serialize("json") profile!: Form;
  @field("profile", "identity.name") name = "";

  @action() @submit("profile")
  save(): void {
    // Persist through an admitted capability boundary.
  }

  render() {
    return <form form={this.profile}><input field={this.name} /></form>;
  }
}
```

Supported legacy serialization formats are `json`, `form-data`, and
`url-encoded`. `required()` creates a validation rule and `@validate(rule)`
attaches one to a legacy field.

## Resources (legacy compatibility)

Use `@resource(endpoint)` for a compiler-declared resource field. This is an
alpha-compatibility declaration. A resource
has `data`, `error`, and `state` (`idle`, `pending`, `ready`, `failed`, or
`cancelled`). `@loader(endpoint)` and `@serverAction(endpoint)` declare the
corresponding explicit endpoint boundaries.

Endpoints must be present in the compiler's application/package capability
information. The static Cloudflare adapter rejects executable server handoffs;
do not present a resource declaration as a general server-runtime substitute.
