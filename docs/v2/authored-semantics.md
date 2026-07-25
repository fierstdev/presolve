# Canonical authored semantics

`presolve_compiler::authored_semantics` owns schema version 1 of the canonical
authored-semantic model. It is the normalization boundary between the general
source AST and resolved TypeScript facts; it is not a replacement parser,
TypeScript checker, or second compiler pipeline.

## Inputs and authority

The parser contributes a `ParsedFile` only to establish the source product,
path, and valid source extent. A caller selects syntax candidates from its
source-faithful AST, then asks `packages/typescript-authority` to classify
framework uses by resolved canonical symbol identity. The normalizer accepts
only those resolved classifications. It does not read decorator names, local
names, import specifiers, or textual source to recognize a framework feature.

TSX bindings and event references are first-class candidates because they are
not framework intrinsics. Their syntax selection and TypeScript validation
remain inputs to the same model, and their emitted records intentionally have
no intrinsic identity.

## Schema V1

`CanonicalAuthoredSemanticModelV1` has:

- schema version `1`;
- the source path from the parsed source product; and
- deterministic, deduplicated declarations ordered by semantic kind, subject,
  source range, and resolved identity.

Each declaration keeps source provenance (`start`, `end`, `line`, `column`).
Framework declarations also retain their TypeScript-resolved name, flags, and
declaration modules. The current vocabulary includes components, state,
actions, computed values, effects, slots, context tokens/providers/consumers,
forms, serialization, fields, validation, submission, resources, route
loaders, server actions, capabilities, TSX bindings, and TSX event references.

Candidates outside the complete source-AST extent are rejected. This makes the
serialized boundary safe for a future process transport without permitting
invented source locations.

## Deliberate non-goals

This slice does not discover legacy decorators or lower them into candidates.
That compatibility work belongs to the next slice, which may create these
records only after TypeScript identity classification. Existing legacy graph
and ASM products remain unchanged until that lowering is present.
