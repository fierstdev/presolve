# Server-action and capability contract

## Scope

This contract completes the beta server-action and capability gates at their
compiler/publication boundary. It makes server actions and route loaders named
admitted records in the canonical semantic-capability registry; it does not
define a server executor.

## Server-action admission and handoff

`route_server_action` accepts a declaration only when it belongs to a
conventional route page and is an empty, synchronous, zero-parameter
`@serverAction("importedEndpoint")` method. It must not also be an Action,
async, or contain body effects. The import must resolve through the canonical
binding table to an integrity-bound semantic-package `server_action` export.

The closed package contract requires `FormData -> ServerActionResult`,
`cold_fallback`, `form_data` input, a `json` or `redirect` response, and typed
failure. Invalid route ownership, declaration, binding, or capability facts
fail with stable `PSROUTE1111` through `PSROUTE1116` diagnostics. Neither
application method bodies nor package source are server code.

`RouteServerActionPlanV1` is schema version 1 and is published deterministically
as `route-server-actions.plan.json`. It carries only exact identity, component
and method, package/version/integrity/export/runtime module, type, resume,
input, response, and failure facts. `presolve check` and `presolve build`
publish this handoff through the ordinary file-route pipeline.

The Node release inventory consumes that artifact only to classify the exact
route as `node`; the static host returns stable 501. Request decoding,
invocation, cache behavior, and response serialization remain a later,
capability-specific executor contract and must not be approximated by browser,
compiler, or deployment code.

## Capability registry

`SemanticCapabilityRegistry` remains schema version 1 and is the only public
admission matrix. Each record fixes a source form, semantic owner, type and
dependency rules, resume policy, artifact impact, and proof fixture. It now
names `resources`, `route_loaders`, and `server_actions` as their completed
bounded paths:

- Resource schema v3 restores validated snapshot triples before dependent
  Computeds, or performs exactly one reload generation with no snapshot slot.
- Route loaders and server actions are server handoffs, not browser execution
  capabilities; both result in node-only route classification until an
  executor contract exists.

`presolve explain --capabilities` exposes deterministic JSON, human, and
migration projections. `presolve migrate` reports the same registry with the
explicit `report-only-no-source-rewrites` policy. The deferred
`semantic_package_exports` family remains deferred: no generic package export,
source rewrite, compatibility fallback, or opaque escape hatch is admitted.

## Completion evidence

Focused proofs establish compiler admission, package validation, deterministic
publication, Node classification, and public registry/migration projections:

1. `route_server_action` and `semantic_package` compiler tests;
2. `publishes_an_exact_route_server_action_handoff_without_server_execution`;
3. `default_check_and_build_publish_a_compiler_route_server_action_handoff`;
4. capability-registry and migration CLI projection tests.
