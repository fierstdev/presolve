# General-purpose Resource authoring contract

## Scope

A general-purpose Resource is a component-owned asynchronous value backed by
one integrity-qualified package export. The compiler proves the complete use
site, bundles only the selected client/shared endpoint through the project's
Vite installation, and owns activation, cancellation, value codecs,
invalidation, diagnostics, and resume behavior.

This surface is distinct from `loader(...)`: use `resource(...)` for work the
browser may execute and `loader(...)` for route-owned server execution.

## Canonical authoring

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

  get profileState() {
    return this.profile.state;
  }

  get profileData(): Profile | null {
    return this.profile.data;
  }

  render() {
    return <article data-state={this.profileState}>
      {this.profileData?.name ?? "Loading profile"}
    </article>;
  }
}
```

The TypeScript authority proves the canonical `resource` and
`ResourceContext` symbols, exact `Resource<Data, Error>` field type, one async
handler parameter, Promise completion, and one direct call to an exact named
package import. Aliases retain identity. Lookalike types or functions, `any`,
default or namespace imports, captures, extra statements, a non-Promise
endpoint, missing authority evidence, and a server-only endpoint fail closed.

`ResourceContext` contains the compiler-owned `AbortSignal` and a frozen empty
input record. Resource inputs, retry, and explicit invalidation are not part of
this beta source form.

## Package contract and Vite publication

The selected semantic-package export must have kind `resource`, a client or
shared execution boundary, abort cancellation, a declared snapshot or reload
resume policy, and a physical runtime-module mapping. This classification
applies to this exact use site; unrelated imports from the same package remain
ordinary TypeScript/Vite inputs.

During `presolve build`, project-local Vite bundles the selected named export
and its ordinary module dependencies into a content-addressed
`/presolve.resource.<digest>.js` asset. The Resource artifact records that
public location. Presolve never publishes the authoring callback as source and
never treats arbitrary package code as compiler semantics.

The `.presolve/resource-build/` workspace is replaced atomically on every
build. It is scratch state, not a retained build history.

## Reactive and lifecycle behavior

Resource `.data`, `.error`, and `.state` projections can feed a pure reactive
getter. Compiler-derived Resource dependencies participate in the same exact
computed and DOM-binding update plans as State dependencies. When an endpoint
settles, only its declared dependents become dirty and rerun.

Every component activation owns one controller and lifecycle generation. It
starts `pending` and reaches exactly one of `ready`, `failed`, or `cancelled`.
Page or component teardown aborts pending work. Data and thrown values must
decode through the compiler-issued `Data` and `Error` codecs; a mismatch emits
a stable runtime diagnostic instead of widening or stringifying the value.

Snapshot endpoints restore completed codec-valid values without importing the
endpoint. Reload endpoints execute one new generation only after the stable
resume phases. Invalid or partial snapshot evidence causes one atomic cold
fallback.

## Completion evidence

The beta gate proves:

1. exact TypeScript symbol, type, Promise, and named-import authority;
2. rejection of a route-loader lookalike as a general Resource;
3. canonical authored Resource and resource-reactive getter dependencies;
4. deterministic, content-addressed Vite publication with one current scratch
   build;
5. compiler-issued Resource, computed, component, resume, and publication
   artifacts; and
6. a real browser importing the package bundle, resolving the Resource,
   invalidating its getter, and rendering the resulting value.
