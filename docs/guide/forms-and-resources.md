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

## Resource status

The current beta does not expose a new-project Resource, loader, or server
action declaration. Retained compiler products can classify historical
handoffs for migration and deployment, but they are not current authoring
guidance.

Use a Form submission or package Action only when its exact documented client
contract fits. The static Cloudflare adapter does not execute arbitrary server
handoffs, and Presolve does not infer a Resource from `fetch()` or a
same-shaped object.
