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
      if (signal.aborted) return;
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

The beta has completion evidence for `required`, numeric
`min`/`max`, string or sequence `minLength`/`maxLength`, `pattern`, and
`email`, plus the cross-field `equals` and `notEquals` rules. Validation runs
after a binding update and again before submission. A named imported Standard
Schema v1 validator is also supported when TypeScript proves its protocol and
field compatibility; synchronous and asynchronous results use compiler-owned
generation ordering so a stale result cannot replace a newer one. Browser
validation is advisory; a server boundary must validate untrusted data again.

An inline submit handler may use the admitted State-update subset. It may also
make one exact imported client persistence call. A route Form can hand off to
one canonical Node server action:

```tsx
import { saveProfile } from "profile-service";

profile = defineForm({
  serialization: "form-data",
  fields: {
    name: field({ initial: "", validate: [required()] }),
  },
  submit: async ({ formData, signal }) => saveProfile(formData, signal),
});
```

TypeScript proves the named import, canonical DOM `FormData` and `AbortSignal`
types, and Promise completion. The Node deployment adapter then bundles only
that selected export and owns method, origin, content type, body limits,
duplicate submission, request cancellation, response codecs, and shutdown.
Cloudflare Static Assets is intentionally static-only and rejects this server
capability during deployment preparation.

## Client and shared resources

Use `resource()` for a component-owned asynchronous value that the browser may
execute:

```tsx
import {
  Component,
  resource,
  type Resource,
  type ResourceContext,
} from "presolve";
import { loadProfile } from "profile-service";

type Profile = { name: string };
type ProfileError = { code: string };

export class ProfileCard extends Component {
  profile: Resource<Profile, ProfileError> = resource<Profile, ProfileError>(
    async (context: ResourceContext) => loadProfile(context),
  );

  get profileData(): Profile | null {
    return this.profile.data;
  }

  get profileError(): ProfileError | null {
    return this.profile.error;
  }

  get profileState() {
    return this.profile.state;
  }

  render() {
    return <article>
      {this.profileState === "failed"
        ? this.profileError?.code
        : this.profileData?.name ?? "Loading profile"}
    </article>;
  }
}
```

The callback is not a generic client escape hatch. It has one canonical
`ResourceContext` parameter and one direct named package call. The selected
semantic-package export declares its client/shared execution boundary,
data/error types, cancellation, and snapshot or reload policy. Project-local
Vite publishes only that export into a content-addressed browser asset.
Presolve owns the `idle` → `pending` → `ready`/`failed`/`cancelled` lifecycle,
aborts pending work on teardown, validates results with compiler-issued codecs,
and updates only the derived values and DOM bindings that depend on it.

## Route loaders

Use `loader()` when a file route needs server-owned data:

```tsx
import {
  Component,
  loader,
  type Resource,
  type RouteParameters,
} from "presolve";
import { loadPost } from "post-service";

export class Post extends Component {
  post: Resource<{ title: string }, { code: string }> = loader(
    async (params: RouteParameters, signal: AbortSignal) =>
      loadPost(params, signal),
  );

  render() {
    return <article>{this.post.data?.title ?? "Loading"}</article>;
  }
}
```

The Node adapter decodes route parameters, executes the integrity-bound export,
validates typed success or failure, applies the declared public/private/no-store
cache policy, and bootstraps the exact Resource activation. Server module code
is not shipped to the browser. Request disconnect and host shutdown abort
pending work.

Presolve never infers a Resource from arbitrary `fetch()` or a same-shaped
object. Use ordinary packages normally when no compiler lifecycle is needed;
use `resource`, `loader`, or a Form server action only when their complete
documented contracts fit the use site.
