# Route-loader handoff contract

## Scope

This contract completes the original loader handoff gate at the compiler/publication
boundary. A route loader is an integrity-bound server capability selected by a
conventional route page. It is not browser code and this contract does not
define a server executor. The additional codec and bootstrap requirements are
fixed by the [Node capability executor contract](node-capability-executor-contract.md).

## Authored admission

`route_loader` retains source-faithful `@loader("importedEndpoint")` field
facts until route and package resolution. `build_route_loader_plan_v1` admits a
loader only when all of the following exact facts hold:

- the owner is a conventional route page in `FileRouteGraphV1`;
- the field declares `Resource<Data, Error>`;
- the designator resolves through the compiler binding table; and
- the selected semantic-package export is an integrity-bound `resource` with
  a valid `route_loader` capability.

The semantic-package contract admits that capability only on a `server` or
`shared` resource endpoint. Its closed inputs are route parameters, a declared
cache policy, and typed failure. Invalid declaration shape, route ownership,
binding, or capability facts fail with stable `PSROUTE1101` through
`PSROUTE1106` diagnostics; package contracts reject invalid capability
combinations before planning.

## Published handoff

`RouteLoaderPlanV1` is schema version 1 and is emitted as the deterministic
`route-loaders.plan.json` publication artifact. For every route, it records
the exact page component and each resolved loader's semantic identity, field,
package name/version/integrity, export, runtime module, type signature,
route-parameter input, cache scope/max age, and typed failure mode. The plan
contains no callback, package source, or executable loader implementation.

`file_route_publication` derives the plan only from the canonical component,
route graph, binding-table, and semantic-package products. The CLI's ordinary
`presolve check` and `presolve build` publish the same handoff; no separate
route matcher or loader resolver is permitted.

## Execution boundary

The browser runtime does not import or invoke a server route loader. The Node
deployment adapter consumes `route-loaders.plan.json` only to classify the
exact route as `node`; its generated static host returns stable 501 for that
route. This is intentional: request decoding, invocation, cache execution,
and response serialization require a later capability-specific executor
contract. Neither the compiler publication path nor a deployment adapter may
simulate that missing product.

## Completion evidence

The gate is complete when focused proofs establish all three boundaries:

1. compiler planning resolves only an integrity-bound, server/shared resource
   loader capability;
2. publication emits the exact handoff and excludes executable package source;
3. the default CLI check/build workflow writes `route-loaders.plan.json`.

The current focused tests are `route_loader` compiler tests,
`publishes_an_exact_route_loader_handoff_plan_without_server_execution`, and
`default_check_and_build_publish_a_compiler_route_loader_handoff`.
