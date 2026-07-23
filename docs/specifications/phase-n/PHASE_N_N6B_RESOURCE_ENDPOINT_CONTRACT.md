# Phase N N6-B Resource package endpoint contract

N6-B makes the existing `resource` semantic-package kind meaningful without
making it callable from application source. A package implementation remains
opaque; the compiler receives only a caller-supplied, integrity-checked export
contract and lowers nothing until a later N6 source/artifact slice.

Every `kind: "resource"` export must include a closed `resource_endpoint`:

```json
{
  "kind": "resource",
  "type_signature": "(ProfileKey) -> Resource<Profile, ProfileError>",
  "runtime_module": "dist/load-profile.js",
  "resume_policy": "snapshot",
  "resource_endpoint": {
    "execution_boundary": "shared",
    "cancellation": "abort",
    "resume": "snapshot"
  }
}
```

The compiler validates the endpoint contract at package-resolution time:

- `execution_boundary` is exactly `client`, `server`, or `shared`;
- `cancellation` is exactly `abort`, making cancellation a required compiler
  capability rather than a best-effort callback convention; and
- `resume` is exactly `reload` or `snapshot`.

No non-Resource export may carry `resource_endpoint`, and a Resource export
without it is rejected as `InvalidResourceEndpoint` before it can be resolved.

N6-B has no `@resource` source form, endpoint invocation, package loading,
runtime artifact, cache, server-action behavior, or resume implementation.
It provides the complete external endpoint vocabulary that the next source
lowering slice must select by identity and carry into its activation artifact.
