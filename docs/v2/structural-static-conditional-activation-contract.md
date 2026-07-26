# Structural static-conditional activation contract

This amendment admits one deliberately narrow execution slice of the
[structural component materialization contract](structural-component-materialization-contract.md):
a conditional host owned by an initially static component instance may
materialize the compiler-issued occurrences in its selected branch. It does
not admit keyed hosts, nested structural hosts, Slot-projected hosts, dynamic
Effects, cleanup, or resume.

## Authority and eligibility

The runtime may enter this slice only for one validated schema-v15 structural
program whose host is a conditional template node and whose selected
`conditional_host_fragments` record has:

1. `host_scope: "static-instance"`;
2. an exact `host_instance` equal to the already-active host component
   instance; and
3. the compiler-rendered branch selected by the ordinary conditional binding;
   and
4. the exact ordered invocation membership published for that branch.

No fragment is inferred from `when_true_html`, a component name, a tag, or a
host DOM shape. A missing, duplicated, wrong-scope, or wrong-host fragment is
a fail-closed artifact error. Keyed and `structural-occurrence` scopes remain
inactive under this amendment.

## Host transaction

Before changing the conditional range, the runtime renders the selected
compiler fragment detached. It may inspect only
`data-presolve-structural-invocation` attributes in that newly rendered,
compiler-issued fragment. Each observed value must name exactly one occurrence
in the selected program and the observed set must equal the schema-v15 branch
membership exactly. This is integrity validation of compiler anchors, not
component discovery: it may not inspect tag names, classes, source markup, or
arbitrary live application DOM.

The transaction then replaces the precise conditional anchor range, processes
observed anchors in the program's `create_order`, and invokes the existing
occurrence materializer with:

- the active static host instance as `parent_scope`; and
- `conditional:true` or `conditional:false` as `local_occurrence`.

If any host, anchor, occurrence, rendering, registration, or attachment step
fails, it disposes every newly created occurrence in reverse creation order
and restores the prior range exactly. A successful later branch replacement
first disposes the prior branch's structural occurrences in reverse creation
order, then removes its DOM range. Repeating an update for the already
selected branch is idempotent: it must not create a second occurrence or
listener.

## Explicit exclusions

This slice does not authorize nested component materialization, keyed-list
creation/reorder/retention/removal, Slot projection, Effect execution or
cleanup, resume restoration, selector/tag discovery, source translation, or
global teardown. Those paths remain governed by their existing contracts and
must receive their own focused activation amendments and browser evidence.

## Required proof

The implementation must prove, using a decorator-free TypeScript-authority
fixture, initial and toggled conditional branch materialization, a live
State/action/binding in the materialized component, removal disposal without
leaked event or binding records, host sibling identity preservation, and
rejection of an invalid host fragment or invocation anchor. The legacy
decorator fixture remains compatibility-only.
