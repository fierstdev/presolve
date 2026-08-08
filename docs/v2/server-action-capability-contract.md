# Server-action and capability contract

## Scope

This contract fixes the server-action and capability gates at their
compiler/publication boundary. It makes server actions and route loaders named
admitted records in the canonical semantic-capability registry. The executable
successor is the
[Node capability executor contract](node-capability-executor-contract.md).

## Server-action admission and handoff

`route_server_action` accepts a declaration only when it belongs to a
conventional route page and is an empty, synchronous, zero-parameter
`@serverAction("importedEndpoint")` method. It must not also be an Action,
async, or contain body effects. The import must resolve through the canonical
binding table to an integrity-bound semantic-package `server_action` export.

The executable package contract requires
`(FormData, AbortSignal) -> Promise<ServerActionResult>`,
`cold_fallback`, `form_data` input, a `json` or `redirect` response, and typed
failure. Invalid route ownership, declaration, binding, or capability facts
fail with stable `PSROUTE1111` through `PSROUTE1116` diagnostics. Neither
application method bodies nor package source are server code.

`route-server-actions.plan.json` schema version 2 preserves schema-v1 legacy
handoffs as `legacy_method` records and adds executable `canonical_form`
records. A canonical record exists only for an authority-proven
`defineForm({ submit: async ({ formData, signal }) => imported(formData,
signal) })` declaration. Authority schema v13 proves the surrounding Form,
named-import identity, canonical DOM parameters, and Promise completion before
lowering. The record adds the Form identity, exact compiler-issued request
path, and `abort` cancellation policy without making the legacy method
executable.

The Node deployment plan is schema version 3. It retains only canonical Form
records in its executable registry, bundles their exact named runtime exports
with the project-local Vite installation, inventories and verifies the bundle
digest, and executes the closed request/response lifecycle defined by the
[Node capability executor contract](node-capability-executor-contract.md).
Canonical route loaders execute through a separate digest-bound registry using
the schema-v2 loader plan and schema-v4 Resource bootstrap.

## Capability registry

`SemanticCapabilityRegistry` remains schema version 1 and is the only public
admission matrix. Each record fixes a source form, semantic owner, type and
dependency rules, resume policy, artifact impact, and proof fixture. It now
names `resources`, `route_loaders`, and `server_actions` as their completed
bounded paths:

- Resource schema v4 restores validated snapshot triples before dependent
  Computeds, or performs exactly one reload generation with no snapshot slot.
- Canonical Resource-field route loaders and canonical Form-bound server actions
  execute only through the schema-v3 Node plan. Legacy decorator handoffs and
  Cloudflare Static Assets remain non-executable.

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
4. capability-registry and migration CLI projection tests; and
5. `canonical_form_server_action_bundles_and_executes_through_the_node_host`,
   covering exact TypeScript rejection, deterministic plans/bundles, real
   browser Form lifecycles, both admitted form media types, JSON, redirect,
   typed failure, origin/body/method rejection, disconnect and shutdown abort,
   mixed static routes, and missing physical runtime/export failures.
