# Presolve beta handoff

## Current objective

Complete the canonical decorator-free V2 Forms gate from the supplied
`presolve-v2-beta-specification.zip`. The authoritative source form is
`defineForm({ serialization, fields, submit })` with nested `field(...)`
declarations and compiler-owned `bind:value`, `bind:checked`, and `bind:files`.

## Completed slices

Decorator-free Form source-authority foundation:

- public `defineForm`, typed `field`, Form state, submission, and value types;
- TypeScript-resolved `defineForm` recognition through V2 authority schema v5;
- canonical Form declarations in the unified authored semantic model;
- V2 graph projection into the existing compiler-owned Form entity; and
- focused compiler, CLI compilation, public TypeScript, and authority tests.

Legacy Form decorators remain compatibility-only and were not used as evidence.

Decorator-free Form Field authority and product projection:

- parser-owned static `defineForm` shape for serialization, nested fields,
  initial values, validation expressions, and inline submit shape;
- TypeScript-resolved canonical `field(...)` recognition through V2 authority
  schema v6, including alias-safe and lookalike-safe site classification;
- canonical nested Form Field declarations joined only below a canonical Form;
- projection into existing Form Field and serialization products, retaining
  nested paths and stable source provenance; and
- focused parser, compiler, authority, CLI compilation, and public TypeScript
  evidence.

Built-in validation helpers are now independently classified by TypeScript
identity through V2 authority schema v7. Aliases lower to their canonical rule
identity; local lookalikes remain inert. Proven rules enter the existing
validation graph with their original arguments and source provenance.

## Next slice

Lower inline submit into a native V2 submission/action product without adapting
it through legacy decorated-method semantics. Add Standard Schema validation,
then complete `bind:value`, `bind:checked`, and `bind:files` runtime/resume
browser proof, with files explicitly excluded from resumable payloads.

## Verification

- `cargo test -p presolve-parser -p presolve-compiler form_definition -- --nocapture`
- `cargo test -p presolve-compiler file_route_assembly_projects_canonical_v2_form_into_existing_form_products -- --nocapture`
- `cargo test -p presolve-cli --no-run`
- `pnpm --filter @presolve/framework test`
- `pnpm --filter @presolve/typescript-authority test`
- `pnpm exec tsc -p tests/framework-public-api/tsconfig.json`

Beta readiness estimate after the built-in validation gate: 84%.
