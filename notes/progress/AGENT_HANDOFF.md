# Presolve beta handoff

## Current objective

Continue from the published and production-verified `0.2.0-beta.16` baseline.
The content-addressed asset correction is live on Presolve.dev and has
returning-client WebKit proof.

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

Forms runtime artifact schema v6 publishes typed validation arguments plus the
exact imported submission capability registry.
Required, min/max, length, pattern, email, and compiler-bound cross-Field rules
fail closed and execute in the browser. The acceptance fixture also proves IME
composition suppression and Form input after snapshot resume.

Standard Schema authority and execution are now explicit:

- TypeScript semantic-authority schema v3 proves Standard Schema v1 protocol
  shape and V2 authoring schema v9 joins named imports to exact
  module/export/declaration identity;
- canonical authored semantics schema v5 carries that evidence without
  executing or serializing validator source;
- the V2 Form graph retains the coordinate on its validation candidate;
- ergonomic builds use the project's direct Vite dependency to bundle exact
  named exports into the publication inventory;
- Forms artifact schema v6 names the module and exact validator IDs; and
- cold/resume browser proof covers Promise scheduling, stale-result
  suppression, non-coercion, issue normalization, validation-aware submit, and
  post-resume interactivity.

The same proof corrected two hidden ownership gaps: Vite now preserves the
registry entry export, and canonical Form `bind:*` channels are excluded from
ordinary component-state binding registration.

Imported Form submission now has a closed authored and package contract:

- the parser retains one direct identifier call and identifier arguments as
  syntax facts without granting them framework meaning;
- the admitted source boundary is an async single-parameter `submit` whose
  complete body calls one named import with exact `value, signal` arguments;
- semantic-package kind `capability` is closed to the exact
  `(FormValue, AbortSignal) -> Promise<void>` signature, client execution,
  abort cancellation, Form-value input, void result, and cold fallback; and
- ambient calls, member/default/namespace imports, captures, reordered
  arguments, and arbitrary source execution remain excluded.

That contract is executable through Forms artifact schema v6. Ergonomic
publication bundles only the exact contract runtime module and export into
`presolve.form-submissions.js`; the browser validates and imports its closed
registry before initializing Forms. The runtime validates before invocation,
constructs the canonical nested value from compiler Field paths, suppresses
duplicates, owns one AbortController per accepted submission, and records
Completed, Failed, Cancelled, Invalid, or reset-to-Idle transitions.

The real-browser acceptance project proves deterministic double-build
publication, exact nested values including `File[]`, fulfillment, rejection,
reset-driven abort, duplicate suppression, cold boot, and resumed submission.
The full inherited workspace gate, including 57 browser tests and the updated
290343-byte production runtime baseline, passes.

Safari returning-client publication correction:

- canonical CSS now publishes at immutable `/app.<sha256>.css` and the retained
  `/app.css` compatibility coordinate;
- each route document executes `runtime.<sha256>.js` while the existing
  `runtime.js` artifact remains available to tooling and compatible hosts;
- the file-route manifest inventories and digest-verifies both immutable and
  compatibility artifacts; and
- a real WebKit iPhone-class probe proves styled output, no viewport overflow,
  and `Count: 0` to `Count: 1` interactivity against the generated application.

## Next slice

Implement the first executable slice of
`docs/v2/package-usage-classification-contract.md`: a decorator-free,
zero-argument terminal named-import call inside a V2 `action()` field, selected
by TypeScript authority and bundled through the project Vite installation.
Preserve use-site classification: discarded client terminal calls are bounded
capabilities, while values flowing into compiler-owned semantics still require
an explicit semantic contract.

Server executors remain outside this beta: the frozen loader/server-action
contracts complete those families at deterministic compiler handoff and
explicitly prohibit an inferred executor.

## Verification

- `cargo test -p presolve-parser -p presolve-compiler form_definition -- --nocapture`
- `cargo test -p presolve-compiler file_route_assembly_projects_canonical_v2_form_into_existing_form_products -- --nocapture`
- `cargo test -p presolve-cli --no-run`
- `pnpm --filter @presolve/framework test`
- `pnpm --filter @presolve/typescript-authority test`
- `pnpm exec tsc -p tests/framework-public-api/tsconfig.json`
- `cargo test -p presolve-cli --test runtime_browser decorator_free_v2_form_fields_bind_and_validate_in_a_real_browser -- --nocapture`
- `cargo test -p presolve-cli --test ergonomic_project -- --nocapture`
- `cargo test -p presolve-cli --test production_runtime_fixtures -- --nocapture`
- `cargo clippy -p presolve-cli --all-targets -- -D warnings`
- `node scripts/verify-release-version.mjs 0.2.0-beta.16`
- `pnpm release:check`

The complete 0.2.0-beta.16 local release dry run passes: formatting, strict
clippy, every Rust and package test, all 57 browser cases, TypeScript 7.0.2,
public TypeScript, 88 documentation files, a newly packed scaffold, crate and
VSIX packaging, and deterministic hashes for all ten release packages. Hosted
CI run `30418471399`, release dry-run `30418471420`, and Publish beta run
`30419010174` passed. Public npm, Cargo, Marketplace, and GitHub release checks
confirm beta.16.

Presolve.dev runs beta.16 as Cloudflare Worker version
`35705c75-0053-4392-a1d5-0e6b6e052030`. All 26 routes return HTTP 200. A
persistent WebKit profile seeded with beta.15 before deployment loaded the same
production URL after deployment without a cache clear, used immutable CSS and
runtime coordinates, remained styled without overflow, and updated the counter
from `Count: 0` to `Count: 1` with no console or request failures.

Beta completion: 100%.
