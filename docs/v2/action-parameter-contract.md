# V2 synchronous Action-parameter contract

This contract extends the initial V2 Action-field subset with the one
parameter form already represented by the compiler artifact and browser
runtime. It is not general JavaScript-handler adoption.

## Admitted source form

A canonical, authority-proven V2 Action field may use one or more inline
parameters only when all of the following hold:

1. the handler is synchronous, block-bodied, and has no unsupported
   statements;
2. every parameter is a unique identifier with one exact primitive TypeScript
   annotation: `string`, `number`, `boolean`, or `null`;
3. every parameter is used by exactly one direct assignment
   `this.<canonical-state> = <parameter>`; and
4. the assigned canonical State has the same primitive type, established by
   its annotation or serializable initial value.

The action may also contain the previously admitted literal, numeric, and
boolean State operations. Parameters are not values captured from outer scope:
they are positional inputs supplied by an event record.

## Event and artifact projection

Only compiler-retained static serializable event arguments may call an Action
with parameters. Every invocation must supply the exact arity and primitive
types of the Action signature. Missing, extra, dynamic, or incompatible
arguments fail before artifact publication.

Parser facts retain the ordered parameter names, annotations, and spans. V2
lowering validates those facts and converts each parameter use to its zero-based
ordinal. The published `assign_parameter` operation carries that ordinal; the
source parameter name and handler source never enter the runtime artifact.
The existing action-batch and endpoint identities remain unchanged.

## Explicit exclusions

The following still reject and require later authored contracts: expression
bodies, async handlers, `AbortSignal`, event-object forwarding, defaults,
rest/destructured parameters, free captures, server imports, dynamic event
arguments, branching, loops, calls, and arbitrary statements. This contract
does not authorize a source translator or a general handler interpreter.

## Acceptance

- The parser retains typed inline parameters without assigning `action`
  meaning to the call.
- Compiler artifacts publish the ordinal operand and retain the existing
  canonical Action endpoint, batch, State storage, and event identities.
- A real-browser decorator-free project proves a static numeric event argument
  updates canonical State through `assign_parameter` after a normal Action
  update, with no runtime diagnostics.
- Unknown, untyped, unused, or type-incompatible parameter assignments and
  mismatched event arguments reject before publication.
