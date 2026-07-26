# Structural occurrence identity contract

This contract defines the opaque runtime identity required to materialize a
compiler-issued structural component template. It is an execution-local key,
not a `SemanticId`, and it is never serialized into a resume snapshot under
the current cold-only structural boundary.

## Inputs

The runtime may create an occurrence identity only after it has validated the
matching schema-v12 structural program and occurrence record. Its four inputs
are:

1. `parent_scope`: the static parent component-instance ID for a top-level
   structural occurrence, or the opaque identity of the materialized parent
   occurrence for a nested structural occurrence;
2. `region`: the compiler-issued structural-region ID;
3. `template_instance`: the compiler-issued structural-template instance ID;
   and
4. `local_occurrence`: `conditional:true` or `conditional:false` for a
   selected conditional branch, or `keyed:<key>` for the exact normalized key
   issued by the existing keyed-list reconciler.

The reconciler's normalized key is the input, not the authored key expression,
raw item, DOM order, element identity, or a string recovered from markup.
An invalid, duplicate, absent, or retained-key decision follows the existing
list reconciliation diagnostics and must not manufacture a replacement key.

## Codec v1

The exact key is:

```text
presolve-structural-occurrence:v1:<hex(parent_scope)>.<hex(region)>.<hex(template_instance)>.<hex(local_occurrence)>
```

`hex(value)` is the uppercase hexadecimal encoding of the UTF-8 bytes of the
non-empty input, with exactly two hexadecimal characters per byte. The prefix
and three `.` separators are literal. No source value, list key, or semantic
identifier may bypass this encoding. A decoder must reject a missing prefix,
wrong field count, empty field, non-hex digit, odd-length field, malformed
UTF-8, or a decoded empty value.

The byte framing prevents collisions between nested repeated parents and makes
the identity suitable as a map key without treating it as a compiler semantic
identity. The only valid parent scope is the exact active parent context issued
by the same materializer; a DOM ancestor, selector, tag name, or source span
is not a substitute.

## Lifetime and ordering

The first successful insertion creates the key. A retained keyed occurrence
keeps it across value updates and reordering. Conditional replacement or keyed
removal disposes it before DOM removal; re-insertion creates a new key even
when the same local key appears later. Children are created only with the
parent occurrence identity as their `parent_scope`, and are disposed before
that parent.

The codec alone does not authorize state-slot creation, binding/event
registration, Effect execution, cleanup, or resume. Those operations must use
the validated materializer program defined by
[`structural-component-materialization-contract.md`](structural-component-materialization-contract.md).

## Required proof

Before any runtime consumer relies on this codec, focused tests must prove:

1. deterministic byte-identical encoding and decode rejection for malformed
   keys;
2. distinct nested/repeated identities with identical local list keys;
3. retained-key stability and remove/reinsert renewal;
4. conditional branch identity renewal; and
5. no identity construction from DOM nodes, attributes, or source strings.
