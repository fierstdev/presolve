# Phase N N6-C2 resource declaration projection contract

N6-C2 projects a resolved endpoint source fact into the compiler's existing
ResourceDeclaration and ResourceActivation products. It remains an internal,
non-executable projection.

A declaration is projected only when all of the following are true:

- `@resource("localEndpoint")` selected one integrity-checked semantic-package
  resource endpoint;
- the field is declared as `Resource<Data, Error>`;
- the compiler can resolve `Data` and `Error` as serializable semantic types;
- the endpoint supplies the client, server, or shared execution boundary.

The projection owns a canonical ResourceId from the component and field name,
a compiler-generated stable key, no inferred input dependencies, explicit-only
retry/invalidation defaults, and the field provenance. It creates exactly one
idle ResourceActivation for each planned instance of the owning component.
The endpoint boundary, data type, and error type are retained in the existing
ResourceDeclaration product.

This does not admit a framework feature. `PSC1046` continues to reject every
`@resource` source declaration. There is no endpoint invocation, source input
syntax, activation scheduling, cancellation, generated artifact, browser
runtime, snapshot, or resume behavior. Those products remain mandatory before
the rejection may be removed.

Verification is `scripts/verify-n6c2-resource-declaration-projection.sh`.
