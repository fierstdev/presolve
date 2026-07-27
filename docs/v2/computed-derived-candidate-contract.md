# V2 derived computed-candidate contract

V2 computed getters are not framework intrinsics. This contract amends the
canonical authored-semantics boundary so an analysis-proven getter can enter
the canonical model without pretending that a `computed()` call or decorator
exists.

## Schema migration

`CanonicalAuthoredSemanticModelV1` advances its serialized schema version from
`2` to `3`. Schema version `3` extends the `DerivedComputedGetter` evidence on
its resulting `Computed` declaration with sorted, deduplicated direct computed
dependencies in addition to its reachable canonical State subjects. It has no
intrinsic identity.

Existing intrinsic and TSX candidates retain their schema-v1 meaning and
serialize no derived-evidence record. Consumers that require a supported V2
computed getter must require schema version `3`; they must not interpret an
ordinary getter, an intrinsic `Computed` declaration, or absent evidence as a
derived computed value.

## Initial admission subset

The parser selects all non-static getters from the general source AST, without
attaching framework meaning. The derived-candidate lowering admits a getter
only when:

1. its owner is a canonical V2 Component and the same canonical model contains
   its State declarations;
2. it is synchronous and has one parser-supported return expression;
3. the expression reaches one or more of that Component's canonical State
   fields through direct State reads or admitted computed getters, and reads no
   other `this.<member>` value;
4. every expression operation is in the existing pure expression subset, with
   arbitrary calls excluded from this first candidate slice.

This finite call-free subset proves purity. Its dependency analysis admits a
computed-to-computed edge only when the complete chain reaches canonical State
and has no cycle. A getter that fails any condition remains ordinary
JavaScript: it is neither an error inferred from its name nor a fallback to
legacy decorator lowering.

## Projection boundary

The V2 component-graph adapter may create a `ComponentMethod` with
`MethodSemanticRole::Computed` only for an exact canonical `Computed`
declaration carrying `DerivedComputedGetter` evidence and matching the parsed
getter. It preserves the existing computed semantic ID, expression graph,
cache, dirty flag, runtime artifact, and resume products. It must not inspect
decorators when projecting this candidate.

Calls stay outside the initial candidate subset. They require a subsequent
amendment that produces explicit call coverage before reaching the same
projection boundary.

## Acceptance

- A canonical V2 component with `state(0)`, `get doubled() { return
  this.count * 2; }`, and `get quadrupled() { return this.doubled * 2; }`
  receives schema-v3 State and direct-computed dependency evidence without a
  decorator or intrinsic identity.
- An ordinary getter, async getter, static getter, unsupported body, unknown
  member read, and call expression receive no derived candidate.
- The graph only projects an evidence-backed getter, and the existing computed
  runtime products retain its stable ID through cold and resumed execution.
