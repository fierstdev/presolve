# Resource lifecycle and resume contract

Resources are compiler-declared asynchronous values. This contract closes the
gap between the published endpoint resume policy and the browser lifecycle;
it does not make endpoint code, browser fetches, or DOM state authoritative.

## Cold lifecycle

Every runtime activation is identified by its compiler-issued activation and
declaration IDs. A client or shared endpoint receives only its compiler-issued
input record and an `AbortSignal`. It moves through `pending`, then exactly one
of `ready`, `failed`, or `cancelled`; page teardown aborts all live controllers.
An endpoint error is retained as the compiler-declared error value only when it
passes the declaration's codec. An unrepresentable result or error is a failed
activation with a diagnostic, never an implicitly stringified value.

## Resume policies

The resolved package endpoint owns the policy:

- `snapshot` restores only a completed `ready` or `failed` activation from
  compiler-authored Resource data/error slots and their exact codecs. It does
  not import or call the endpoint during resume.
- `reload` restores no Resource value. After stable State, computed, Context,
  component, and Form records have been restored, it starts one new generation
  through the exact compiler-issued activation and runtime-module coordinate.
- A `pending` or `cancelled` activation is never resumable. Missing, duplicate,
  malformed, stale, mismatched, or non-codec values select one atomic cold
  fallback. No partial Resource registry survives that fallback.

Browser runtime may reload only client/shared endpoints. A server-only reload
requires an explicit server executor; until that product exists it selects the
same cold fallback rather than attempting a browser call.

## Artifact and ordering

The resource artifact (schema v2) publishes the exact endpoint coordinate,
runtime location, policy, cancellation mode, lifecycle generation, input
dependencies, and compiler-issued closed codecs for the declaration's data and
error types. The browser validates that codec grammar before executing an
endpoint, then validates every completed data/error value against it; it never
uses JSON stringification as a serialization proof. The resume manifest
publishes one instance-qualified Resource record per resumable activation and
no source text. Resource restoration occurs before any computed read of that
Resource; reload completion invalidates only the compiler-authored dependent
computed records and bindings.

The compiler resume products reserve three exact slots for each `snapshot`
activation: terminal state plus nullable data and error. Both terminal values
use the declaration codec under a nullable wrapper, allowing a single closed
capture shape for either `ready` or `failed`. A Resource-reading computed
depends on that activation's data slot, so its recomputation cannot precede
Resource restoration. `reload` activations publish no snapshot slots.

## Acceptance

- Browser tests prove ready snapshot restoration without endpoint execution,
  typed failure restoration, cancellation on teardown, and exactly one reload
  generation.
- Malformed snapshot, pending/cancelled state, missing runtime module, and
  server-only reload take one cold fallback with no partial Resource registry.
- Artifact and resume products are deterministic under reversed input order.
