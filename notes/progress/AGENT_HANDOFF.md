# Presolve beta handoff

## Current objective

Complete the canonical decorator-free V2 Forms gate from the supplied
`presolve-v2-beta-specification.zip`. The authoritative source form is
`defineForm({ serialization, fields, submit })` with nested `field(...)`
declarations and compiler-owned `bind:value`, `bind:checked`, and `bind:files`.

## Completed slice

Decorator-free Form source-authority foundation:

- public `defineForm`, typed `field`, Form state, submission, and value types;
- TypeScript-resolved `defineForm` recognition through V2 authority schema v5;
- canonical Form declarations in the unified authored semantic model;
- V2 graph projection into the existing compiler-owned Form entity; and
- focused compiler, CLI compilation, public TypeScript, and authority tests.

Legacy Form decorators remain compatibility-only and were not used as evidence.

## Next slice

Add a parser-derived, source-faithful static Form definition view and lower
nested `fields` entries into existing Form Field, serialization, validation,
and submission products. Preserve the frozen Form runtime identities and fail
closed for dynamic schemas, unsupported validation, files in resume data, and
malformed definitions.

## Verification

- `cargo test -p presolve-parser -p presolve-compiler form_definition -- --nocapture`
- `cargo test -p presolve-compiler file_route_assembly_projects_canonical_v2_form_into_existing_form_products -- --nocapture`
- `cargo test -p presolve-cli --no-run`
- `pnpm --filter @presolve/framework test`
- `pnpm --filter @presolve/typescript-authority test`
- `pnpm exec tsc -p tests/framework-public-api/tsconfig.json`

Beta readiness estimate after this slice: 79%.
