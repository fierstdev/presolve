# Presolve beta handoff

## Current objective

Freeze and implement the first capability-specific Node executor contract for
compiler-issued route-loader and server-action handoffs. Preserve the existing
file-route graph, semantic-capability registry, environment ownership, and
deployment inventory as the only authorities; do not add a second router or
execute unclassified package source. Prove request decoding, exact endpoint
loading, cancellation, loader caching, FormData action input, JSON/redirect
responses, typed failure, and mixed static/node routes before widening the
surface. Then continue the remaining compatibility, diagnostics, performance,
and release-hardening gates.

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

Decorator-free terminal package Actions now cover the frozen primitive and
Promise completion surface:

- TypeScript authority schema v11 proves the exact named import, ordered
  `string`/`number`/`boolean`/`null` signature, synchronous non-Promise return
  or `Promise<void>` completion, and canonical DOM `AbortSignal` identity;
- authored-semantics schema v7 and package-invocation artifact schema v2 retain
  exact argument codecs, completion, injection, concurrency, cancellation, and
  restore-without-replay facts;
- the public `action` type hides the final compiler-owned signal from event
  callers while retaining its exact package handler signature;
- the Action event JSON boundary now emits canonical numbers as numbers rather
  than strings, closing a shared runtime transport defect;
- Promise invocations replace prior work per component instance, abort on
  structural teardown or `pagehide`, suppress stale settlement, preserve
  package failures, and never serialize or replay pending work;
- the package interoperability example proves every primitive codec plus
  fulfillment, rejection, replacement, pagehide cancellation, and compatible
  resume; and
- deterministic double builds retained identical invocation-artifact and
  registry hashes. The full workspace gate passes with 58 browser tests and
  662 compiler tests; the audited runtime baseline is 301,905 bytes and remains
  within its production budgets.

The canonical scaffold, publication, and documentation reconciliation is
implemented and locally proven:

- `create-presolve` now has interactive destination selection plus
  `--help`/`--version`/closed option handling and refuses every existing target;
- the generated application owns document metadata in `app/index.html`,
  shared composition in `app/app.tsx`, page landmarks in routes, byte-exact
  global CSS in `app/app.css`, and root-addressed files in `public/`;
- the starter is mobile-first, keyboard-visible, reduced-motion aware, and
  proves State, Action, and Computed updates in a real browser beneath a shared
  application shell;
- the generated README and public guides explain the complete ownership model,
  content-addressed CSS, public versus imported assets, Tailwind/PostCSS
  preprocessing, Vite's bounded bundling role, package Action classification,
  and fail-closed server boundaries;
- successful atomic application and file-route builds now retire obsolete
  exact Presolve release directories while retaining the active release,
  preserving the active pointer on failure, and leaving caller lookalikes
  untouched; and
- the Presolve.dev production-shaped Examples route exposed a framework bug:
  computed initialization ran every global evaluation for every component
  instance. Runtime execution now filters each evaluation to its compiler-owned
  component definition, so an app shell cannot request a route's computed slot.

The exact scaffold app-shell + route-computed browser regression passes with
State, Action, Computed, immutable CSS, metadata, landmarks, and zero runtime
diagnostics. Application publication, all 18 ergonomic-project tests,
production determinism, production budgets, strict touched-crate clippy,
TypeScript 7.0.2 compatibility, and all 90 public documentation files pass.
The audited runtime baseline is 302,247 bytes and remains within the committed
budgets.

`0.2.0-beta.18` and VS Code prerelease `0.2.18` are prepared in lockstep. The
complete uninterrupted release dry run passes at those versions: all Rust and
package tests, 59 real-browser cases, TypeScript compatibility, documentation,
clean-room installation from newly packed scaffold/packages, Rust packaging,
VSIX packaging, and deterministic hashes for all ten release packages. The
compiler-contract version change regenerated the canonical tooling query
snapshot and its validation identity.

Hosted CI `30513133319`, release dry run `30513133285`, and Publish beta
`30513847524` completed successfully for `0.2.0-beta.18`. The publish run
released all crates, npm packages, native CLI binaries, VS Code prerelease
`0.2.18`, and the non-draft GitHub prerelease.

Presolve.dev then installed the exact public beta.18 packages. Its mobile
browser acceptance probe found a second identity defect: the corrected initial
Computed loop selected evaluations by authored component name, but cold-boot
and resume component records retained the semantic component ID in their
`name` field. The runtime therefore skipped the correct route evaluations and
registered their bindings with empty caches, rendering `$ undefined` and
`Estimated accounts undefined` until an Action happened to run the
instance-qualified update path.

Cold-boot and resume allocation now retain `definition.name`, matching the
computed artifact's authoritative component owner while continuing to use the
semantic ID as the instance definition lookup key. A generator assertion
forbids the incorrect `name: instance.component` spelling, and the generated
starter browser probe verifies the authored route name. The production-shaped
Presolve.dev route built with this compiler now proves, at a 390px viewport:

- runtime ready with zero diagnostics;
- content-addressed CSS applied and no horizontal overflow;
- initial cart total `$ 24` and estimated accounts `1000`;
- Action/Computed updates to `$ 48` and `3000`;
- boolean Action state, structural pipeline tabs, and Preview/Code switching;
- native invalid submission blocking; and
- a valid Form submission updating State.

The same site proof exposed document-template placeholder capture. Sequential
`replacen()` calls inserted application HTML before selecting the runtime
placeholder, so documentation that displayed literal `{{ runtime }}` consumed
the runtime payload inside its code block and left the template location
unfilled. Document assembly now locates all three validated positions before
inserting any payload and projects them in source order. A compiler regression
preserves all three literal spellings inside application content while emitting
exactly one runtime manifest. The Presolve.dev project-structure artifact now
contains a compact 2,515-byte HTML sample with all three literals and no runtime
spill, rather than an 835,288-byte contaminated code sample.

## Next slice

Author the narrow Node executor contract missing from the existing loader,
server-action, and Node deployment handoffs, then implement one complete
vertical slice without changing those frozen compiler products. The generated
host must consume exact package coordinates and integrity facts from the
published plans, decode only admitted request inputs, retain abort and cache
ownership, return only admitted JSON/redirect/failure results, and continue to
serve compiler-classified static routes unchanged. Require focused compiler,
CLI, generated-host, deterministic-publication, and real-request evidence
before expanding the executor surface.

The complete beta.19 release dry run passes after the audited runtime baseline
decreased by six bytes from the identity spelling correction: strict formatting
and clippy, the Rust workspace with 663 compiler tests, all 59 browser cases,
package and TypeScript 7.0.2 tests, 90 documentation files, clean-room scaffold
installation, Rust/VSIX packaging, production budgets, and deterministic
release package hashes.

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
Hosted CI `30516684036`, release dry run `30516684048`, and Publish beta
`30517196543` completed successfully for beta.19. npm `beta` tags for the
framework, CLI, and creator resolve to `0.2.0-beta.19`; the GitHub release is a
non-draft prerelease and VS Code `0.2.19` was published. Full-beta completion is
now 49%.

The next framework gate closes the canonical `pnpm dev` gap. The CLI now
watches authored inputs, reruns compiler publication atomically, refreshes the
compiler-issued route manifest, hot-swaps CSS through stable development
coordinates without losing page state, and uses a safe full reload for edits
without narrower compatibility evidence. Failed rebuilds retain the last good
publication and display compiler stderr in an accessible external development
client; recovery reloads automatically. Compiler-owned hidden stages are
excluded from observation, preventing rebuild loops.

Focused loopback integration passes for initial serving, CSS update,
stylesheet bytes, semantic route reload, invalid-edit retention, diagnostic
publication, and recovery. Real Chrome passes the exact state-preservation
matrix: red to blue CSS with a preserved marker and
`/app.css?presolve-dev=1`, semantic content reload, retained last-good content
under `PSROUTE2002_FILE_ROUTE_SET_EMPTY`, and clean recovery. Strict touched
crate clippy, creator tests, and all 90 public documentation files pass before
the beta.20 full release gate.

The uninterrupted beta.20 release dry run is now green. It includes strict
formatting and clippy, all 59 browser cases, 663 compiler tests, the complete
Rust/package/TypeScript suites, 90 documentation files, clean-room scaffold
verification, Rust and VSIX packaging, deterministic production budgets, and
ten deterministic release artifacts. One earlier full run lost the V2 Form
Chrome probe to SIGKILL; the isolated test passed immediately and the
uninterrupted rerun passed the complete browser matrix. Full-beta completion is
50%.

## Current beta.23 publication boundary

`0.2.0-beta.23` is published and independently proven. Hosted main CI
`31075333292`, hosted release dry run `31075333304`, and Publish beta
`31144139320` passed. A fresh exact public creator invocation emitted beta.23
framework, CLI, and TypeScript-authority pins with the audited pnpm installer
policy, then passed install, check, build, and Cloudflare preparation.

Presolve.dev is committed at site commits `0bf5589` and `19ffb7e` and deployed
as Cloudflare Worker version `5eb64fc4-af8b-40f9-be96-24b4c9e5bcb7`, release
SHA-256 `7a0dde41e5b80a7df5cf61d3b89c5f3a9a35fd1a3995dad8873d28d3eeef1203`.
Production browser acceptance proves the canonical `/` URL, beta.23 title and
favicons, content-addressed CSS across reload, zero 390px overflow, opaque
close-on-navigation mobile menus, `Count: 0` to `Count: 1` with no runtime
diagnostics, live Computed/tabs/Form examples, and intact code samples with the
literal `{{ runtime }}` placeholder.

The next hardening slice must remove the manual Cloudflare upload intervention.
The exact site publication is about 80 MB across 740 assets; Wrangler 4.119.0's
three concurrent bulk requests repeatedly failed with `EPIPE`, while a
local-only concurrency of one uploaded all 258 changed assets successfully.
Do not retain or document a patched dependency as the solution. Characterize
which generated products are genuinely browser-public, measure route-document
duplication, and implement a compiler/deploy-owned correction with deterministic
inventory, local deployment preparation, and release-sized Cloudflare proof.
Then continue the remaining authored beta gates and final compatibility,
determinism, diagnostics, artifact, and product hardening. Full-beta completion
is 73%.

## Beta.24 compact publication candidate

Document-embedded compiler JSON is now serialized as a compact browser
transport while the matching pretty-printed, digest-bound files remain byte
stable canonical artifacts. Unit and CLI publication tests parse both forms
and require exact JSON-value equality. Script-closing text remains escaped.

The production-shaped Presolve.dev build retains the complete 667-file
compiler publication but falls from 82,324,637 to 62,269,120 bytes. Route HTML
falls from 33,661,442 to 13,607,215 bytes; canonical JSON, JavaScript, CSS, and
runtime bytes are unchanged. A local browser proof passed the homepage counter,
computed example, tabs, and zero-diagnostic runtime with one-line embedded
manifests.

The uninterrupted beta.24 release dry run passes strict formatting and clippy,
all 59 real-browser cases, 664 compiler tests, the complete Rust/package/
TypeScript 7.0.2 suites, all 90 public documentation files, production
determinism and budgets, a real-lifecycle packed scaffold install/check/build/
Cloudflare preparation, crate and VSIX packaging, and all ten deterministic
release artifacts. The principal package hashes are:

- framework: `74b3d2b81d8c46b6ed30a491816464254a2ef108b725e426de539dce5627841c`;
- CLI: `62d7c45f65f7dfaa398b9ede659b0967ed8dfd49a3b9cb4c84ab20db7f2bb577`;
  and
- creator: `a1b18b44de2effa71d8c99965e8e85a7c5d409bc751016d9f7e3cecf7f55c4ab`.

The audited runtime remains 302,655 bytes. The action-counter HTML baseline is
26,136 bytes and component-structural HTML is 180,165 bytes. Hosted gates,
publication, exact public scaffold proof, Presolve.dev adoption, and the
unmodified Wrangler deployment remain required. Full-beta completion is 75%.

## Beta.24 public publication and deployment

Hosted main CI `31156361152`, hosted release dry run `31156361475`, and Publish
beta `31157001481` completed successfully. The publish run released all three
Rust crates, every npm/compiler/native-CLI package, VS Code prerelease `0.2.24`,
and the non-draft GitHub prerelease. The updated npm credential was exercised
successfully by the complete package publication.

An exact public `pnpm create presolve@0.2.0-beta.24 my-app` invocation emitted
beta.24 framework, CLI, and TypeScript-authority pins, installed from the public
registry, passed `presolve check`, built the production publication, and
prepared Cloudflare deployment. The documentation intentionally retains the
explicit beta version because pnpm 11 applies its one-day minimum package age
to unversioned creators. Release verification now also locks the creator's
generated pins and the recovery workflow default to the requested version;
future creator publications converge both npm `latest` and `beta` tags.

Presolve.dev commit `4428f22` adopts beta.24 and its explicit creator command.
All 38 source checks, production build, and Cloudflare preparation pass. The
publication retains 667 filesystem files and Wrangler's 740-path inventory at
about 62.6 MB, with 13,612,470 HTML bytes. Local and production browser proof
at 390 by 844 verifies immutable CSS, no horizontal overflow, title and
versioned favicons, the homepage counter, opaque close-on-navigation mobile
menus, cart/Computed updates, rollout `1000 -> 2000`, structural tabs,
Preview/Code switching, and a valid Form submission with zero diagnostics.

Official unmodified Wrangler 4.119.0 uploaded all 257 changed assets in one
ordinary deployment. Cloudflare Worker version
`7c57ccbf-1748-4d6c-8bfd-9a23eb27b9c5` serves release SHA-256
`4d967286cdb65961d14ce8a0d348f78dfb5388fa8dd29cd72fbe9aefe3d15d1b`;
the favicon, PNG mark, and immutable stylesheet all return HTTP 200. The
beta.23 manual-transfer blocker is closed. Full-beta completion is 77%.
