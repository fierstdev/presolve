# Structural effect-instance activation contract

`ComponentInstancePlan` distinguishes a finite static instance from a
`StructuralTemplate` nested below a conditional or keyed list. V2 effect
cleanup may not treat that template as an active component instance: one
conditional selection or each keyed occurrence has a separate lifetime.

## Template authority

The compiler must project each valid V2 effect owned by a structural-template
component instance to a structural effect template. That record contains only
compiler-issued identities: the component-instance template ID, effect
declaration ID, structural-region ID, parent template/instance, depth, field
declaration order, and existing main/cleanup program references. It contains
no DOM selector, callback, source spelling, or user-controlled list key.

Each structural-region program must additionally publish its exact renderer
address: the canonical owning component ID and the generated conditional or
keyed-list template-node ID. The compiler derives that pair from the same
semantic template entity that issued the structural-region ID. The browser
validates the pair against the template manifest before it may use the program;
it must never search the DOM for a matching shape or infer an address from a
selector.

The same structural program carries one ordered occurrence template for every
structural component instance: its template-instance ID, resolved invocation
ID, and target component ID. This is inactive compiler metadata, not a DOM
lookup or an active instance. A later materializer must use it to stamp the
compiler-issued occurrence marker at successful insertion.
The runtime indexes those records by invocation ID at boot and rejects a
duplicate before rendering begins.

Static planned instances remain ordinary effect-instance records. A structural
effect template is inactive metadata and must not execute or register cleanup.

## Runtime activation and teardown

The structural DOM runtime is the sole authority that observes a successful
conditional insertion or keyed-list creation. It must issue a runtime instance
identity from the compiler template identity plus the exact structural
occurrence identity, activate parent-before-child, and invoke the ordinary V2
effect runner under that instance context.

Before DOM removal, it resolves active instances below the affected structural
region and calls the shared disposal operation child-before-parent, with
reverse field order per component instance. Only after cleanup and
listener/binding teardown may it remove DOM nodes and delete active records.
Reconciliation that retains a keyed occurrence preserves its identity and
never re-runs or disposes its effects.

## Admission boundary

Cleanup-bearing V2 fields remain rejected until artifact validation, runtime
dynamic activation, conditional/keyed removal disposal, and nested/repeated
browser fixtures prove exact activation, retained-key stability, cleanup before
removal, and child-before-parent order together. This contract does not
authorize DOM-derived fallback identities, global cleanup sweeps, or
adapter-side lifecycle implementations.
