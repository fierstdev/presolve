# Phase N N8-B compatibility and migration contract

**Status:** implementation authority.

N8-B publishes compatibility and rejected-syntax guidance from the canonical
semantic capability registry. It does not add a source migration transform,
legacy adapter, source rewriter, or JavaScript fallback.

## Product

```sh
presolve explain --capabilities --format migration
```

The compiler renders one deterministic guide with three sections:

1. **Compatibility policy** — the registry schema is the exact capability
   support boundary; admitted records are supported only under their listed
   source/type/dependency rules; deferred records have no compatibility path.
2. **Migration guide** — one ordered item per deferred record containing its
   ID, source form, and compiler-issued rejection reason. The reason is the
   only migration direction; the projection must not invent a source rewrite.
3. **Rejected syntax catalog** — the same deferred IDs in source order, one
   per line, with their class and reason.

JSON and the N8-A human matrix remain unchanged. This projection is a
developer-readable view and has no independent schema version.

## Authority and boundaries

The registry itself is the sole compatibility authority. The guide must not
parse an application, infer a compatibility result from package versions, list
framework-only declarations, reinterpret old artifacts, or imply that `opaque`
is already usable. A deferred record is rejected until a full semantic contract
moves it to admitted.

The guide is intentionally forward-looking only: it explains the current
compiler support boundary, not backward compatibility for a prerelease
compiler. No old source spelling, artifact schema, or generated runtime byte
is accepted solely because it appeared in an earlier development slice.

## Verification

N8-B proves byte determinism, source order, one migration and catalog entry per
deferred record, absence of an admitted record in either deferred section,
unchanged JSON/human output, CLI projection, and a focused verifier. The proof
consumes the compiler product; shell or documentation copies are not accepted
as authority.
