# Presolve beta handoff

## Current objective

Broaden decorator-free V2 Action package invocation beyond the beta.17
zero-argument subset with compiler-transported serializable arguments and
Promise-aware completion. The amendment must define dependency, cancellation,
failure, teardown, and resume products without retaining or evaluating handler
source.

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
The full inherited workspace gate, including the expanded 58-browser-test
matrix and updated 297,509-byte production runtime baseline, passes.

Safari returning-client publication correction:

- canonical CSS now publishes at immutable `/app.<sha256>.css` and the retained
  `/app.css` compatibility coordinate;
- each route document executes `runtime.<sha256>.js` while the existing
  `runtime.js` artifact remains available to tooling and compatible hosts;
- the file-route manifest inventories and digest-verifies both immutable and
  compatibility artifacts; and
- a real WebKit iPhone-class probe proves styled output, no viewport overflow,
  and `Count: 0` to `Count: 1` interactivity against the generated application.

Decorator-free package interoperability now has its first executable vertical
slice:

- V2 TypeScript authority schema v10 proves the call-site alias and exact
  named-import declaration identity; same-spelled local functions remain inert;
- authored-semantics schema v6 carries only the proven terminal coordinate;
- ordinary package imports in V2 files are Vite inputs and no longer require a
  legacy semantic-package contract merely to exist;
- one synchronous, zero-argument, result-discarded package call can own a
  canonical Action endpoint without `@component`, `@action`, or `@opaque`;
- the compiler publishes `package-invocations.runtime.json`, and the ergonomic
  CLI Vite-bundles the exact export into
  `/presolve.package-invocations.js`;
- the browser validates the registry before boot, invokes the export once per
  event, records completion/failure evidence, and emits stable fatal/runtime
  diagnostics for missing or rejecting modules;
- a valid snapshot restores the Action event without replaying the external
  side effect, then remains interactive; and
- `examples/package-interop` is the release-grade authored and browser example.

The real-browser gate proves cold publication, compatible resume, exactly-once
execution across repeated events, a missing registry failure, and no diagnostic
or State corruption. Two complete builds produced identical registry,
invocation-artifact, and route-document hashes.

The `0.2.0-beta.17` public release and site adoption are complete:

- hosted `main` CI `30455449678` and release dry run `30455449652` passed;
- Publish beta run `30456828739` published all three crates, all npm packages
  and native CLIs, VS Code prerelease `0.2.17`, and the non-draft GitHub
  prerelease;
- independent npm queries resolved every public package's `beta` tag to
  `0.2.0-beta.17`;
- Presolve.dev installs the public beta.17 framework, CLI, and TypeScript
  authority packages and documents use-site package classification without
  decorators; and
- Cloudflare Worker version `758ba975-db0a-4772-9a7c-9225b76b20a1`
  (release SHA-256
  `e374169289c6402125b522df8a448d5ae474efd515df4493b7d621c660738a6e`)
  serves the beta.17 homepage and package guide. Production mobile proof
  covers content-addressed CSS, returning reload, logos, internal code
  scrolling, opaque menus, menu close-on-navigation, and `Count: 0` to
  `Count: 1` with no console diagnostics.

## Next slice

Amend the package-usage contract for compiler-transported serializable Action
arguments and Promise-aware completion, then implement it through the existing
TypeScript authority, canonical authored model, invocation artifact, Vite
registry, browser runtime, resume, diagnostics, example, and real-browser
proof. Do not infer arbitrary handler-source execution.

Then continue the remaining beta gates: broader Actions, Effect
execution/cleanup/resume, slot-projected structural hosts, Context,
completion-grade Forms/Resources/loaders/server actions/capabilities, and final
hardening.

Server executors remain outside the browser publication boundary until their
frozen handoff contracts gain an explicit executor product.

## Verification

- `pnpm --filter @presolve/typescript-authority test`
- `pnpm run test:types`
- `cargo test -p presolve-compiler file_route_assembly_ -- --nocapture`
- `cargo test -p presolve-compiler package_invocation -- --nocapture`
- `cargo test -p presolve-compiler runtime_codegen::tests::emits_runtime_manifest_bootstrap -- --nocapture`
- `cargo test -p presolve-cli package_invocation_bundle_specifiers -- --nocapture`
- `cargo test -p presolve-cli --test ergonomic_project -- --nocapture --test-threads=1`
- `cargo test -p presolve-cli --test application_publication -- --nocapture`
- `cargo test -p presolve-cli --test runtime_browser decorator_free_package_invocations_bundle_execute_resume_and_fail_closed_in_a_real_browser -- --nocapture --test-threads=1`
- `pnpm --dir examples/package-interop check`
- `pnpm --dir examples/package-interop build`
- deterministic double-build SHA-256 comparison of the registry, invocation
  artifact, and route document
- in-app browser verification of `0 -> 1 -> 2`, runtime-ready status, applied
  application CSS, and zero console errors
- `pnpm release:check`

The previous “100%” statements referred to completing individual beta.14-16
release trains and production regressions, not the final beta product scope.
The current full-beta completion estimate is 40% after beta.17 publication and
production documentation adoption.
