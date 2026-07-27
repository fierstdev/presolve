# Environment-read lowering contract

The named dotenv manifest is the only value authority for Presolve
environment reads. This contract admits a source form without permitting
ambient `process.env`, `import.meta.env`, or arbitrary global access.

## Source form

V2 source reads an environment value through the framework intrinsic:

```ts
import { environment } from "presolve";

const appName = environment.public("PRESOLVE_PUBLIC_APP_NAME");
```

The parser retains only the call site, its single string-literal argument, and
source span. TypeScript authority resolves the callee to the exported
`environment.public` identity. Neither parser spelling nor an imported local
name has framework meaning.

## Canonical join

Lowering joins three pre-existing facts:

1. the AST-selected call and literal-name span;
2. the exact resolved framework intrinsic identity; and
3. a caller-provided `EnvironmentInputManifestV1` whose `browserValues`
   contains the exact name.

It emits a versioned environment-read record with source provenance and a
literal browser value. A missing manifest, unresolved intrinsic, dynamic name,
or name absent from `browserValues` fails closed. Names listed only in
`serverValueNames` receive a browser-boundary diagnostic and never expose a
value.

## Boundaries

This product does not read dotenv files, process state, Vite environment
objects, or source text beyond the parser-selected literal. It does not
authorize server reads; server runtime configuration remains a separate
capability-specific product. Existing environment-ownership analysis consumes
the record rather than rediscovering names. Every admitted record is projected
as a browser/application-lifetime ownership node using its source-qualified
call identity; a lowering with diagnostics cannot be reclassified or
published.

## Acceptance

- Aliased imports resolve through TypeScript authority and produce the same
  canonical record as the direct import.
- A public value is projected deterministically with no server value in any
  browser artifact.
- Dynamic, unprefixed/server-owned, missing-manifest, and shadowed calls fail
  with stable diagnostics.
- A browser publication fixture proves the emitted public value and rejects
  an attempted server value.
