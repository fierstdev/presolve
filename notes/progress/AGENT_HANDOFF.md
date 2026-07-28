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

Decorator-free `bind:value` and `bind:checked` expressions now resolve
`this.<form>.fields.<path>` directly through canonical Form and Form Field
identity. A real-browser acceptance project proves cold boot, input/change
updates, built-in validation, exact runtime artifacts, and stable resume with
control synchronization.

Inline `submit` now lowers natively into a compiler-owned V2 action endpoint,
submission plan, host record, serialization step, and browser execution. The
admitted first subset accepts a one-parameter sync or async callback containing
canonical State updates; unsupported statements and unowned State writes fail
closed with V2 diagnostics.

Decorator-free file controls now have an authority-backed platform-value path:

- V2 authority schema v8 classifies `field<File[]>` from the resolved generic
  signature and the configured DOM `File` identity;
- only `<input type="file" bind:files={...}>` and FormData serialization admit
  the value;
- runtime change, reset, required validation, and native file-control behavior
  have real-browser proof; and
- file value/tracking/validation slots are excluded from resume, while other
  fields resume and the Form deterministically revalidates after rebinding.

## Next slice

Add Standard Schema validation, then broaden native inline submit/action
execution to admitted imported capability calls with abort signals.

## Verification

- `cargo test -p presolve-parser -p presolve-compiler form_definition -- --nocapture`
- `cargo test -p presolve-compiler file_route_assembly_projects_canonical_v2_form_into_existing_form_products -- --nocapture`
- `cargo test -p presolve-cli --no-run`
- `pnpm --filter @presolve/framework test`
- `pnpm --filter @presolve/typescript-authority test`
- `pnpm exec tsc -p tests/framework-public-api/tsconfig.json`
- `cargo test -p presolve-cli --test runtime_browser decorator_free_v2_form_fields_bind_and_validate_in_a_real_browser -- --nocapture`

Beta readiness estimate after the file binding gate: 91%.
