# Structural component materialization contract

This contract is the admission boundary between compiler-planned
`StructuralTemplate` component instances and runtime-created conditional or
keyed-list occurrences. It is deliberately narrower than effect cleanup: it
authorizes a renderer to materialize an already-planned component topology,
but does not by itself authorize Effect execution, cleanup registration, or
resume persistence for that topology.

## Compiler authority

The sole input is the validated component runtime artifact structural program:
its structural-region ID, exact host component/template-node address, ordered
`template_occurrences`, and the ordinary compiler products for the referenced
component. Component artifact schema v10 retains each occurrence's exact ordered
ordinary target, binding, and event membership plus compiler-rendered component
template HTML as inactive compiler projection data. An occurrence record is
valid only when all of the following agree:

1. `template_instance` is the compiler-issued structural-template instance;
2. `invocation` is the exact invocation that issued that template instance;
3. `component` is the resolved target of that invocation; and
4. the occurrence belongs to the structural region whose host renderer is
   currently reconciling.

The compiler must publish any additional renderer program as a versioned,
digest-covered artifact field. It must name a component-template root and all
ordinary target, binding, event, slot, and nested structural records needed to
render that occurrence. Runtime code may not reconstruct that program from
source text, tag names, DOM shape, selector searches, or a second component
graph.

Structural lifetime propagates through every descendant component edge of a
structural template. A nested invocation is therefore a structural template
under the same enclosing region even when its own declaration appears outside
the conditional or keyed-list source span. It must not be reclassified as an
eager instance merely because its local source template has no structural node.

Existing `when_true_html` and `item_template_html` are plain-template products.
They are not authority to render a component invocation as a raw custom HTML
element. A structural program containing a component occurrence must remain
inactive until its compiler-issued materializer program is present and
validated.

The ordinary compiler renderer stamps each structural invocation element with
its exact compiler-issued invocation ID. This marker is an integrity-checked
fragment anchor, not a tag lookup: a materializer must reject a missing or
duplicate marker and may never substitute a same-named element.

Schema v10 is not a runtime materializer. It establishes the validated compiler
template and membership set from which the later program must be constructed;
the runtime must not activate, infer, or partially use it for dynamic component
rendering.

## Runtime identity and reconciliation

For each successful insertion, the structural runtime creates one opaque
runtime occurrence identity from the structural region, the compiler template
instance, and the reconciler's exact occurrence identity. The identity is
runtime-local: it is not a new `SemanticId`, is never inferred from a DOM node,
and is not reused by a removed conditional branch or a removed keyed item.
Its exact parent-scoped, UTF-8 hex codec is fixed by the
[`structural occurrence identity contract`](structural-occurrence-identity-contract.md).

For a keyed list, retaining the same reconciler occurrence retains that opaque
identity. Reordering alone neither rematerializes nor disposes it. Duplicate
keys and invalid/missing occurrences remain the existing fail-closed runtime
diagnostics; they do not select a fallback template. Conditional replacement
creates an occurrence only after the new branch has been successfully inserted.

Materialization follows the artifact's ordered occurrence list, creates parents
before children, and stamps only compiler-issued target markers. Event and
binding registration is attached to those stamped records, not discovered by
walking for matching application markup. The runtime must roll back all records
created for an occurrence if any validation, render, marker, or registration
step fails.

## Teardown boundary

The materializer exposes one shared, idempotent occurrence-disposal operation.
It unregisters bindings and events in child-before-parent order before removing
the occurrence's DOM range and records. Effect cleanup is not part of this
contract. A later lifecycle amendment may invoke this disposal operation only
after it proves effect execution under the same opaque occurrence identity and
the required reverse declaration order.

Dynamic structural occurrences are cold-only under this contract. Resume keeps
the existing fail-closed boundary until a separate versioned resume product
proves that each live occurrence can be restored without DOM discovery or
identity fabrication.

## Required proof before activation

An implementation must add focused compiler/artifact and browser fixtures for:

1. conditional insertion and removal of a component invocation;
2. keyed creation, retention, reorder, and removal of repeated component
   invocations;
3. nested structural component parent-before-child creation and rollback;
4. exact event and binding registration without duplicate or leaked records;
5. rejection of a malformed host, occurrence, marker, or renderer program;
   and
6. no dynamic Effect execution or cleanup before the lifecycle amendment.

This contract does not permit decorator recognition, runtime compilation,
source translation, DOM-derived component discovery, global teardown sweeps,
or a parallel renderer outside the compiler artifact boundary.
