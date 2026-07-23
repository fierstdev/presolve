# Phase N N8-A capability matrix contract

**Status:** implementation authority.

N8-A turns the existing versioned semantic capability registry into a
developer-readable matrix without creating a second source analyzer,
framework-owned feature list, or runtime authority.

## Product

The canonical machine product remains:

```sh
presolve explain --capabilities --format json
```

N8-A additionally admits this deterministic human projection:

```sh
presolve explain --capabilities --format human
```

The compiler generates both products from the same ordered
`SemanticCapabilityRegistry`. The human projection is Markdown-compatible
plain text headed by the registry schema version and has one fixed-width row
per capability:

```text
id | class | status | source form | proof fixture
```

Deferred rows include their compiler-issued rejection reason in a following
indented line. Admitted rows do not invent a warning, migration, or fallback.

## Authority and compatibility

The human matrix is an inspection view, never a schema. JSON remains the
versioned automation interface and preserves its bytes and schema version.
The matrix must not parse application source, inspect generated JavaScript,
guess framework support, or derive a capability from TypeScript declarations.

Adding the human format does not admit a deferred capability. A record moves
to `admitted` only through the N0 full-path admission contract. A source that
is not represented by an admitted record remains compiler-rejected until a
separate semantic contract is completed.

## Verification

N8-A proves one compiler renderer, byte-determinism, every registry row once
in source order, the deferred-reason rendering rule, CLI JSON compatibility,
CLI human projection, and no source-file argument requirement. The focused
verification command must use those products directly rather than re-listing
capabilities in a shell script.
