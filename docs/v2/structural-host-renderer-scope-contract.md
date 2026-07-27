# Structural host renderer scope contract

This contract extends the structural materialization boundary with the compiler
inputs required to render every structural host exactly. It exists because a
conditional branch owned by an initially static root can use an already-issued
component-instance ID, while a keyed item or a nested component host cannot.
Neither case may be approximated from markup, a tag name, or a runtime walk.

## Scope classes

Each compiler-issued structural host renderer has exactly one scope class.

1. `static-instance` names one existing ordinary component-instance ID. Its
   branch markers use that ID unchanged.
2. `structural-occurrence` names the compiler structural-template instance
   that owns the host. Its renderer is parameterized only by the opaque parent
   occurrence identity produced under the structural occurrence identity
   contract.
3. `keyed-occurrence` names the exact list host and its compiler list item
   template. Its renderer is parameterized only by the keyed reconciler's
   normalized occurrence input; source keys, item values, DOM order, and DOM
   nodes are not renderer inputs.

The compiler must select the class from the validated component-instance and
template plans. Runtime code must not select, upgrade, or infer it.

## Required renderer program

For every scope class, the versioned component artifact must retain one
compiler-rendered program with all of the following:

- the structural region and exact host component/template-node/semantic-entity
  address;
- the exact host scope class and compiler source instance or template instance;
- both conditional branches or the keyed item fragment, as applicable;
- the complete ordinary target, binding, event, slot, and nested structural
  membership reachable from that fragment; and
- the exact placeholder discipline used to substitute an opaque occurrence
  identity and, for keyed scope, the reconciler-issued local occurrence input.

The program is invalid when a required host scope, fragment, placeholder, or
membership record is absent, duplicated, or inconsistent with the component
artifact. A conditional fragment cannot be attached to a keyed-list host, and
a keyed fragment cannot be attached to a conditional host.

Schema v11 `conditional_host_fragments` began with the `static-instance`
subset. Schema v12 additionally carries `structural-occurrence` fragments for
nested hosts with no caller-owned Slot projection. Schema v13 adds keyed item
fragments with the same parent scope. Their absence for a Slot-bound host is
intentional and must keep materialization inactive.

## Slot ownership

A host that renders caller-owned Slot content must carry the exact
compiler-selected Slot projection program as part of its renderer record. It
may not flatten Slot content from the caller's DOM or synthesize an empty
projection. A nested host with such a projection is not eligible for the
schema-v11 subset.

## Runtime use

At activation, the runtime validates the renderer program before it mutates
the DOM. It receives only the already-selected scope input, substitutes only
the compiler-declared placeholder, checks each structural invocation anchor
for exactly one match, and registers only the program's membership records.
Any failure rolls back the whole occurrence and leaves the former range
unchanged.

This contract does not activate component materialization, Effects, cleanup,
or resume. It supplies the remaining renderer-scope product that must exist
before the materialization contract's conditional and keyed browser proofs can
be admitted.
