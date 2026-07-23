# Phase N N6-C13 resource source publication contract

N6-C13 admits one activation-only Resource source form:

```tsx
import { loadProfile } from "profile-service";

@component("x-profile")
class Profile extends Component {
  @resource("loadProfile") profile!: Resource<string, string>;
  render() { return <main>Loading profile</main>; }
}
```

The decorator designator must resolve to one named or default import whose
caller-supplied semantic-package contract declares a `resource` export. The
declared field must resolve as `Resource<Data, Error>`. A browser `presolve
build` additionally requires `--package-runtime
profile-service=./profile-resource.js`; the compiler expands that mapping to
the exact package/version/integrity/runtime-module coordinate in the resource
artifact. Missing module locations fail with `PSRES1001`; a malformed generated
artifact fails with `PSRES1002`; server-only endpoints fail browser publication
with `PSRES1003`.

For a valid client or shared endpoint, the compiler publishes
`resources.runtime.json`, embeds identical JSON before `runtime.js`, and the
generated runtime performs the cold activation with an `AbortSignal`. It owns
the resource status and page-teardown cancellation. Package implementation is
loaded only through the exact declared runtime location; no source inspection,
package discovery, runtime fallback, or framework resource store is used.

This is deliberately activation-only. The field is not yet a render or
Computed dependency, and Resource inputs, invalidation, retry, component
destruction, snapshot/resume, and result/error source access remain deferred.
They require a later contract that defines their identities, dependencies,
serialization, and lifecycle transition rules.

Verification is `scripts/verify-n6c13-resource-source-publication.sh`.
