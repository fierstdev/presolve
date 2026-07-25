# Structural components and props contract

`presolve_compiler::structural_component` is schema v1 of the V2 component and
props projection. It consumes explicit TypeScript-authority facts for component
inheritance and generic props. Class names, filenames, exports, decorators, and
source heritage spelling do not establish component identity here.

Only a `resolved_presolve_component` fact produces a component record. An
unresolved base or unsupported mixin heritage becomes a diagnostic rather than
an inferred component. Existing component products remain unchanged until their
recognition boundary explicitly adopts this projection.

`Component<Props = {}>` becomes either an explicit resolved object shape or an
empty default shape. Props are represented in deterministic field order.
`children` is never injected: it exists only when the resolved props object
explicitly declares it. A route root with unresolved generic props is rejected
at this boundary, before publication or runtime lowering can assume a shape.

This projection contains no TSX assignability, publication serialization, or
runtime behavior. Those later products consume its canonical records and the
separate TypeScript and codec authorities.
