# V2 authoring-syntax cutover contract

The V2 beta source surface is the decorator-free language defined by the base
specification.  This contract makes that source surface, rather than the
existing alpha compatibility syntax, the default for public declarations,
examples, scaffolds, and new compiler work.

## Canonical source forms

- A component is a class whose TypeScript-resolved base is the exported
  `Component`; a class decorator is neither required nor consulted.
- `Component<Props = {}>` declares props.  Route or TSX reachability, never a
  class name or a decorator, controls publication reachability.
- `state(initial)` in an instance field declares reactive state.
- `action(handler)` in an instance field declares an action.  It replaces a
  decorated method.
- A synchronous reactive getter is computed; there is no `computed()`
  intrinsic.
- `defineForm`, `resource`, `createContext`, and module-level `server.action`
  use the forms specified in `02-language/` of the base archive.  No new V2
  feature may introduce a decorator spelling.

This document does not invent lowerings for forms, resources, context, or
server actions.  Their individual contracts must be amended only when their
documented source form has a TypeScript-resolved and source-faithful candidate
path into the canonical authored semantic model.

## Recognition and compatibility boundary

Canonical recognition is selected from the general source AST and classified
by resolved TypeScript symbol identity.  It produces the existing canonical
authored semantic model; it must not add a parser subset, text recognizer, or
parallel semantic pipeline.

`typescript-authority` serializes an ordered `componentHeritage` base-symbol
chain for a queried class. It resolves aliases and walks indirect class bases,
but assigns no framework meaning. The canonical intrinsic registry classifies
that chain, and only a matching resolved `Component` identity may be supplied
to component-inheritance lowering.

`legacy_decorator_lowering` remains an alpha compatibility adapter.  It may
lower a resolved legacy decorator into that same canonical model, but it must
not be used to recognize canonical V2 source, drive generated source, or
define public documentation.  Compatibility examples and fixtures must be
labelled `legacy` and must not be used as beta acceptance evidence.

The current `presolve migrate` command remains report-only.  Removing or
rewriting decorators requires a separately versioned source-transform product
with before/after fixtures, source-location proof, and an amendment to
`migration-contract.md`; no text replacement or AST heuristic is authorized.

## Delivery sequence and evidence

1. Public declarations, README examples, and `create-presolve` emit only
   canonical V2 source.  The scaffold test asserts the absence of `@` syntax.
2. Each canonical source form receives an AST-selected, resolved-symbol
   candidate path and a focused fixture.  Component inheritance is first and
   is implemented by `component_inheritance_lowering`: it joins only
   TypeScript-proven heritage clauses to the canonical authored model. State
   initializer lowering follows the same rule: a parser-selected direct field
   call joins an exact resolved `state` callee only after its owning component
   has been proven. Action-field lowering applies the same component-ownership
   and resolved-callee rule to `action(handler)`; decorated methods remain in
   the legacy adapter.
3. Existing downstream products adopt that canonical model one at a time.
   A compatibility lowering never becomes evidence that the V2 source form
   works.
4. A beta gate proves a decorator-free representative application through
   parsing, semantic recognition, publication, cold behavior, and resumed
   behavior.  Legacy fixtures remain a separately named compatibility gate.

The first step is intentionally small: it corrects the public promise without
claiming that every legacy-only downstream lowering has already been rewritten.
