# Pre-development readiness review

Status: **required before implementation begins**  
Audience: founders, technical leads, compiler/runtime engineers, maintainers  
Purpose: identify the decisions, constraints, and operating rules that should exist before the first serious development sprint.

## Executive answer

The product thesis is strong enough to begin, but the project is still at risk of beginning with too much architectural ambition and too few control surfaces.

The missing work is not another feature list. The missing work is mostly:

1. a development constitution,
2. a first-user wedge,
3. compatibility policy,
4. performance budgets,
5. compiler/runtime contract testing,
6. a security and supply-chain plan,
7. a standards and browser-support policy,
8. a governance model for syntax and semantics,
9. a release discipline,
10. a dogfood application that proves the system beyond counters.

Without those, the team can build many impressive parts that do not converge into a shippable framework.

## The most important pre-development decision

Decide what the first credible product is.

Do not begin by trying to build the full generational framework. Begin by building a narrow system that proves the central claim:

> The compiler understands a component deeply enough to ship less JavaScript, preserve HTML-first behavior, and explain every generated artifact.

The first product should probably be:

> **A TSX/html-template component compiler that emits SSR HTML, a small resumability manifest, lazy event chunks, and standards-native Web Components for a small but real app.**

Everything else should support or test that claim.

## Sprint zero deliverables

Before feature implementation, create the following repo artifacts.

```txt
adr/
  0001-product-constitution.md
  0002-monorepo.md
  0003-browser-support-policy.md
  0004-compiler-runtime-contracts.md
  0005-release-channels.md

rfcs/
  0001-component-authoring-model.md
  0002-reactivity-model.md
  0003-resource-action-model.md
  0004-resumability-model.md
  0005-accessibility-rule-taxonomy.md

schemas/
  component-manifest.schema.json
  chunk-manifest.schema.json
  diagnostics.schema.json
  explain-output.schema.json

fixtures/
  counter/
  form-action/
  resource-streaming/
  lazy-event/
  web-component-output/
  accessibility-failures/

examples/
  contact-form/
  todo-crud/
  product-page/
```

This prevents the compiler, runtime, CLI, and docs from evolving as disconnected opinions.

## 1. Product constitution

Write a short constitution before code.

It should define:

- what EdgeZero is,
- what it is not,
- which constraints beat convenience,
- which optimizations are allowed to change semantics,
- how much syntax magic is acceptable,
- when a feature is an escape hatch,
- what must remain inspectable.

Suggested constitution:

```txt
EdgeZero is a web authoring compiler with a framework surface.
HTML is the primary artifact.
JavaScript is loaded only when required by interaction, state, or runtime semantics.
The compiler must be able to explain generated behavior.
Accessibility violations that can be known statically are compiler diagnostics.
The runtime executes compiler decisions; it does not hide compiler ignorance.
Web Components are a native output target, not the only authoring model.
Performance cannot require routine user-managed memoization.
Resumability cannot require routine user ceremony.
Interop with the platform is a product requirement.
```

Also define anti-goals:

```txt
Not a React clone.
Not a JSX-only UI library.
Not a Web Components wrapper library.
Not a Rust-branded framework.
Not a client-SPA-first framework.
Not a framework that requires users to understand bundler internals.
Not a research compiler with no migration path.
```

## 2. First-user wedge

Pick one initial audience. Do not build equally for every audience at first.

Candidate wedges:

### Design-system and component-library teams

Pros:

- Web Component output matters.
- Standards-native distribution is attractive.
- Accessibility compiler is a strong differentiator.
- Interop is concrete.

Cons:

- They care deeply about styling, slots, parts, theming, Shadow DOM, and versioning.
- Resumability is less compelling than distribution quality.

### Content-heavy product teams

Pros:

- HTML-first delivery and low JavaScript are compelling.
- Astro-like use cases create an obvious benchmark.
- Forms, actions, resources, and streaming matter.

Cons:

- Web Component output may feel secondary.
- CMS/data integrations become important quickly.

### Full-stack application teams

Pros:

- Forms, resources, actions, routing, server/client split, and resumability all matter.
- The thesis is most complete here.

Cons:

- Surface area is large.
- Auth, deployment, database, caching, and errors become table stakes.

Recommendation:

> Start with **content-heavy and form-heavy product applications**, then use Web Component output as the interop and packaging story.

That wedge best exercises HTML-first delivery, forms, resources, lazy interaction, accessibility, and explainable compilation without forcing the team to solve every design-system edge case first.

## 3. Browser and standards support policy

The browser support policy must exist before syntax and runtime decisions harden.

Define:

- minimum browser set,
- whether support follows Web Platform Baseline,
- how polyfills are handled,
- how custom elements and declarative shadow DOM are gated,
- whether older browsers receive degraded HTML only,
- whether generated output uses modern syntax by default.

Suggested policy:

```txt
EdgeZero targets modern evergreen browsers by default.
The compiler may emit Baseline-wide platform features without polyfills.
Non-Baseline features require target-gated transforms or explicit user opt-in.
HTML fallback must remain useful when JavaScript is unavailable.
Old-browser support is a compilation target, not the default target.
```

This matters because features like custom elements, constructable stylesheets, import maps, streaming, modulepreload, declarative shadow DOM, and scheduler APIs change output strategy.

## 4. Accessibility standard and diagnostic policy

Accessibility cannot be hand-wavy. Choose a standard and a diagnostic contract.

Recommended baseline:

```txt
Compiler diagnostics should align with WCAG 2.2 AA where static or semantic analysis can detect a violation.
The compiler should distinguish definite errors, probable warnings, and advisory improvements.
Runtime-only accessibility issues should be exposed through devtools and test helpers.
```

Define diagnostic classes:

```txt
error       statically known invalid or inaccessible authoring
warning     likely issue, but context may prove valid
advisory    best-practice guidance
runtime     requires browser state or user interaction to verify
unsupported cannot be proven by compiler
```

Examples:

```txt
error: <img> missing alt unless marked decorative
error: form field has no associated label
error: clickable non-interactive element without keyboard path
warning: aria-label may duplicate visible label
warning: modal has no explicit focus return target
advisory: consider aria-live for async form status
runtime: focus trap verification requires interaction trace
```

The accessibility compiler should not pretend it can prove everything. It should be precise about what it knows.

## 5. Performance budgets

Define budgets before implementation; otherwise the compiler has no target.

Suggested MVP budgets:

```txt
Initial framework loader:     <= 1.5 kB brotli for simplest resumable page
Counter interaction chunk:    <= 1.5 kB brotli excluding user code
Form enhancement chunk:       <= 3.0 kB brotli excluding schema validator
Static page JS:               0 kB by default
Hydration-equivalent work:    none unless explicitly requested
DOM updates:                  direct binding patch, no VDOM diff baseline
```

Also track:

- HTML bytes,
- JS bytes by interaction,
- number of lazy chunks,
- parse/compile cost,
- time to first byte,
- time to interactive interaction,
- server render latency,
- edge cold start impact,
- compiler time per component,
- incremental rebuild latency.

The CLI should eventually expose this through `fw size`, but the budget should exist before `fw size` does.

## 6. Compiler/runtime contract strategy

This is one of the highest-risk areas.

The compiler and runtime must communicate through versioned manifests and schemas, not implicit assumptions.

Minimum contracts:

```txt
component manifest
  component id
  static DOM template id
  binding ids
  event ids
  resource ids
  action ids
  style ids
  accessibility metadata
  source map pointers

chunk manifest
  chunk id
  lazy load trigger
  imports
  captured values
  serialization requirements
  browser/server ownership

resume manifest
  resumable state cells
  continuation ids
  event delegation roots
  serialized resource snapshots

explain manifest
  source expression -> graph node -> generated output mapping
```

CI should run golden tests that verify:

- generated HTML,
- generated manifest,
- generated lazy chunks,
- generated diagnostics,
- generated source maps,
- runtime behavior in a browser.

This is more important than polished syntax in the first month.

## 7. Serialization and capability model

Resumability fails if serialization rules are vague.

Define early:

- what values can cross server/client boundaries,
- how closures are represented,
- whether class instances can be serialized,
- how resources are resumed,
- how non-serializable state fails,
- whether functions can be captured,
- how actions refer to server capabilities,
- how secrets are prevented from leaking into client bundles.

Suggested rule:

```txt
Only explicitly serializable values may cross the server/client boundary.
The compiler must reject accidental capture of server-only values into client-resumable code.
Every rejected capture must include the source path that caused the capture.
```

A vague serialization model will create Qwik-like ceremony or worse: security bugs.

## 8. Security model

Security needs a first-class document before implementation.

Minimum topics:

- server action authentication and authorization,
- CSRF handling for forms/actions,
- XSS and HTML escaping policy,
- CSP compatibility,
- secret leakage prevention,
- server/client import boundary enforcement,
- dependency provenance,
- build artifact integrity,
- package signing/provenance,
- source map publication policy,
- dev server exposure risks,
- SSR request isolation,
- multi-tenant edge runtime assumptions.

Suggested first security invariant:

```txt
The compiler must never emit server-only imports, secrets, environment variables, database clients, filesystem access, or privileged capabilities into browser bundles.
```

Suggested second invariant:

```txt
Server actions must require an explicit capability path and must have a CSRF-safe default transport.
```

The project should also decide early whether npm packages will use trusted publishing/provenance and whether Rust artifacts will be signed or accompanied by attestations.

## 9. Supply-chain and release trust

Because this project ships compilers, CLIs, npm packages, and native binaries, supply-chain trust is part of the product.

Before publishing anything public, define:

- who can publish,
- whether release is CI-only,
- whether long-lived npm tokens are prohibited,
- whether provenance is generated,
- whether binary checksums are published,
- whether release artifacts are reproducible,
- how security advisories are handled,
- how dependency updates are reviewed.

Suggested policy:

```txt
All public packages are published from CI.
Long-lived npm publish tokens are prohibited where trusted publishing is available.
Release artifacts include provenance or attestations where supported.
Native binaries publish checksums.
Security fixes receive coordinated advisories and patch releases.
```

## 10. Licensing and contribution policy

Decide these before outside contributors arrive.

Questions:

- MIT, Apache-2.0, dual MIT/Apache-2.0, or another license?
- Does the project require a CLA or Developer Certificate of Origin?
- Who owns trademarks?
- What names are protected?
- Can the compiler include third-party parser/runtime code?
- What license constraints apply to generated code?
- Can commercial users embed generated runtime output freely?

Recommendation:

```txt
Use Apache-2.0 OR MIT/Apache-2.0 dual licensing unless there is a specific business reason not to.
Avoid a CLA unless there is a real corporate/IP reason.
Use DCO if contribution sign-off is desired without CLA overhead.
Make generated output permissive for application use.
```

Name clearance should also happen before a public launch. Owning a domain is not the same as having trademark clearance.

## 11. Syntax governance

Syntax will be the most tempting place to overbuild.

Create a syntax RFC rule:

```txt
No new syntax lands without:
  1. semantic graph representation,
  2. compiler diagnostic behavior,
  3. generated output examples,
  4. explain output examples,
  5. migration story,
  6. escape hatch story.
```

This protects the project from becoming a collection of attractive but incompatible ideas.

For every proposed feature, require:

```txt
Can the compiler understand it?
Can the compiler explain it?
Can it degrade to HTML where possible?
Can it avoid client JS where possible?
Can it be debugged from source to DOM?
Can it interoperate with the platform?
```

If not, classify it as an escape hatch.

## 12. Error philosophy

The framework needs an explicit diagnostic style.

Bad diagnostic:

```txt
Cannot serialize closure.
```

Good diagnostic:

```txt
Cannot make `onClick` resumable because it captures `db`, which is server-only.

Source:
  src/routes/users.tsx:42:17

Capture path:
  onClick -> save -> updateUser -> db

Fix:
  Move `updateUser` into a server action:

  save = action(async form => {
    "server";
    await updateUser(form);
  });
```

Every major feature should have example diagnostics before implementation.

## 13. Test strategy

Unit tests are not enough.

Required test layers:

```txt
parser tests
semantic graph tests
reactive graph tests
serialization tests
accessibility diagnostic tests
codegen snapshot tests
manifest schema tests
browser runtime tests
SSR tests
streaming tests
resumability tests
source map tests
CLI explain tests
package integration tests
fixture app e2e tests
```

Golden fixtures are essential because the product promise is generated behavior.

Each fixture should verify:

- source input,
- semantic graph,
- generated HTML,
- generated JS chunks,
- generated manifests,
- diagnostics,
- browser behavior,
- `fw explain` output.

## 14. Benchmark strategy

Do not benchmark only counters.

Use benchmark categories:

```txt
static marketing page
content page with one interactive form
dashboard with streaming resources
product page with image gallery and cart action
admin CRUD form
large component tree with many bindings
component library build output
```

Compare against:

- React/Next-style hydration baseline,
- Astro island baseline,
- Svelte baseline,
- Solid baseline,
- Qwik resumability baseline,
- Lit component package baseline.

Measure:

- JS shipped before interaction,
- JS loaded per interaction,
- interaction latency after lazy load,
- HTML size,
- build time,
- incremental rebuild time,
- memory use,
- edge/server render latency,
- source map usability,
- diagnostics quality.

Benchmarking should support claims, not drive fake micro-optimizations.

## 15. Devtools and inspectability path

The inspector is strategically important, but it should not be built as a browser extension first.

Recommended sequence:

```txt
1. `fw explain` text output
2. `fw explain --json`
3. local web inspector reading the explain manifest
4. browser overlay in dev server
5. browser extension only after stable graph contracts
```

This lets the compiler data model mature before investing in UI-heavy tooling.

## 16. Router and file conventions

Do not let routing conventions emerge accidentally.

Decide:

- route decorator vs filesystem routing vs both,
- nested layouts,
- route params typing,
- error boundaries,
- loading/streaming boundaries,
- metadata/head management,
- server-only routes,
- endpoint routes,
- static generation behavior,
- whether routes compile to Web Components or app-only components.

Recommendation:

```txt
Support explicit route declarations first.
Add filesystem routing later as a convention layer.
The compiler IR should not depend on filesystem routing.
```

This preserves portability for component libraries and nonstandard project layouts.

## 17. Styling model

Styling can derail framework architecture.

Decide early:

- scoped CSS mechanism,
- Shadow DOM default or opt-in,
- CSS Modules support,
- global CSS handling,
- design tokens,
- CSS custom properties,
- `::part` and `::theme` strategy,
- critical CSS extraction,
- style graph dead-code elimination,
- SSR style ordering,
- streaming style flushing,
- theming without hydration.

Recommendation:

```txt
Do not make Shadow DOM mandatory.
Support light DOM by default for application components.
Support Shadow DOM and parts for distributable Web Components.
Represent styles in the style graph either way.
```

Mandatory Shadow DOM would improve encapsulation but complicate app styling, forms, accessibility, SSR, and third-party CSS integration.

## 18. Package and API stability levels

Use explicit stability channels.

```txt
internal     no compatibility promise
experimental opt-in, may break in minors before 1.0
preview      intended design, may still change
stable       documented compatibility promise
```

Mark every package and API:

```txt
@edgezero/compiler-core     internal until 1.0
@edgezero/runtime           preview
@edgezero/cli               preview
@edgezero/vite              experimental
@edgezero/server-node       experimental
@edgezero/server-cloudflare experimental
@edgezero/devtools          experimental
```

Do not imply stability by publishing too many packages too early.

## 19. Migration and codemod policy

The project should assume syntax changes will happen.

Create `fw migrate` from the start, even if it only prints planned transforms.

Migration rules:

- every breaking syntax change needs a codemod or explicit non-codemodable explanation,
- docs must show before/after,
- diagnostics should point to migration commands,
- RFCs must include migration impact.

This avoids repeating the trust damage caused by ecosystem-wide syntax churn.

## 20. Adapter strategy

Adapters should be defined by capability, not by brand.

Adapter capabilities:

```txt
static output
node server
edge runtime
streaming response
server actions
file uploads
websocket/live channel
KV/cache
durable state
asset manifest
```

A Cloudflare adapter, for example, should be described as a set of supported capabilities, not just `@edgezero/cloudflare`.

This allows clear errors:

```txt
This route uses streaming server actions, but the selected adapter only supports static output.
```

## 21. Auth and session policy

Even if auth is not built immediately, the framework’s action and resource model must not make auth awkward.

Decide:

- how request context reaches resources/actions,
- how cookies are read/written,
- how sessions are typed,
- how server actions authorize mutations,
- how redirects work,
- how progressive enhancement preserves auth semantics,
- how CSRF protection composes with native forms.

Avoid baking in one auth provider. Provide primitives and examples.

## 22. Data cache and invalidation semantics

Resources and actions need a dependency model.

Questions:

- Are resources keyed explicitly or inferred?
- Can actions invalidate resources?
- Can invalidation be compiler-inferred from dependencies?
- How does optimistic state reconcile with server state?
- How are stale resources represented in the UI?
- How do streamed resources fail independently?

Initial rule:

```txt
Resources must have stable compiler-visible identities.
Actions may declare invalidation targets.
Compiler inference can suggest invalidation, but explicit declarations win.
```

Avoid building a magical cache with unclear invalidation. Cache bugs are trust killers.

## 23. Error, loading, and streaming semantics

Async UI needs a formal model.

Define:

- pending state,
- error boundaries,
- retry behavior,
- cancellation,
- partial stream failure,
- validation error transport,
- nested resource dependencies,
- whether fallback UI is required or inferred.

Recommended primitive set:

```txt
resource()
action()
<Await>
<ErrorBoundary>
<Pending>
<Retry>
```

But avoid forcing users into verbose wrappers for common cases.

## 24. Interop acceptance tests

Interop should be tested, not claimed.

Create fixtures for:

- consuming a vanilla custom element,
- emitting a custom element and using it from plain HTML,
- wrapping a React component,
- using a Lit component,
- using a vanilla DOM library,
- using Tailwind or another utility CSS workflow,
- importing an npm ESM library,
- using a server-only npm package without leaking it to client output.

This turns “interop” into a release gate.

## 25. Documentation strategy

Docs should be generated and tested against the compiler as early as possible.

Documentation types:

```txt
conceptual docs       explain the model
how-to docs           complete tasks
reference docs        exact API behavior
diagnostics docs      error causes and fixes
architecture docs     compiler/runtime internals
migration docs        version-to-version changes
```

Every public example should be compile-tested.

Do not publish aspirational examples that the compiler cannot run.

## 26. Naming, package names, and namespace control

Before public development:

- run trademark screening,
- reserve npm scope if available,
- reserve crates.io names if appropriate,
- reserve GitHub organization,
- reserve Discord/Matrix/community handles,
- decide whether the binary is `edgezero`, `ez`, or `fw`,
- decide whether docs use `EdgeZero`, `edgezero`, or `ez` consistently.

Recommendation:

```txt
Product: EdgeZero
CLI: ez
Compiler binary: ezc
NPM scope: @edgezero/*
Rust crates: edgezero-* or ez-* only after name diligence
```

Do not ship public packages before name diligence is complete.

## 27. Governance and decision process

Even a small founding team needs rules.

Minimum governance:

- one product owner for scope,
- one compiler owner for IR/semantic model,
- one runtime owner for browser behavior,
- one release owner,
- RFC required for user-facing semantics,
- ADR required for repository/process/tooling decisions,
- CODEOWNERS for review gates,
- deprecation policy before first public beta.

Without governance, every attractive idea becomes backlog debt.

## 28. Community expectations

If public, decide what kind of project it is.

Options:

```txt
closed design, open source drops
open development, closed roadmap
open RFCs, maintainer-decided
foundation-style governance
commercial core with OSS framework
```

Recommendation for early stage:

```txt
Open source code.
Maintainer-decided roadmap.
Public RFCs for major semantics.
No governance-by-popularity until architecture stabilizes.
```

## 29. Business and sustainability questions

Even if the framework is open source, sustainability matters.

Clarify:

- is this a VC-backed devtools company?
- consulting/adoption support?
- hosted compiler/observability product?
- enterprise accessibility/compliance tooling?
- cloud deployment platform?
- sponsorship/donation model?

Strategic risk:

> A framework can get adoption but still fail if nobody funds maintenance, docs, adapters, and support.

The strongest commercial wedge may be compiler observability, accessibility compliance, and enterprise migration tooling rather than hosting.

## 30. Kill criteria

Define what would prove the thesis wrong.

Examples:

```txt
If resumability requires visible user ceremony in common cases, reduce scope.
If generated output cannot beat existing frameworks on real apps, re-evaluate positioning.
If Web Component output forces unacceptable DX compromises, make it an output mode rather than the default mental model.
If compiler diagnostics are vague, pause feature work and improve explainability.
If the runtime grows beyond budget to patch compiler limitations, fix the compiler model.
```

Kill criteria protect the project from sunk-cost drift.

## Before coding: the 20 yes/no questions

Answer these in writing before implementation starts.

1. What is the first app we will dogfood?
2. What user wedge is the first public release for?
3. What is the initial browser support policy?
4. What is the initial accessibility standard?
5. What initial JS budget counts as success?
6. What runtime size budget counts as failure?
7. What values are serializable?
8. What values are forbidden in client-resumable code?
9. What is the manifest schema between compiler and runtime?
10. What is the first output target?
11. Is Shadow DOM default, opt-in, or target-dependent?
12. Is routing decorator-first, filesystem-first, or both?
13. What does a server action compile to?
14. How are CSRF and authorization handled?
15. What package names are reserved?
16. What license will the project use?
17. What APIs are internal vs experimental vs preview?
18. What benchmark apps will be used?
19. What is the RFC process?
20. What must be true before a public alpha?

## Recommended public alpha gate

Do not call it alpha until all of this works:

```txt
- compile at least three real examples
- emit SSR HTML
- emit lazy event chunks
- emit runtime manifest
- resume one interaction without full hydration
- run native form fallback without JS
- run enhanced form submit with JS
- produce accessibility diagnostics
- run `fw explain` on every example
- show JS-by-interaction size report
- package one component as a custom element
- pass browser e2e tests
- pass source map smoke tests
- publish packages from CI
- document known limitations honestly
```

## Recommended immediate sequence

### Phase 0: freeze the operating model

- write product constitution,
- write browser/accessibility/security policies,
- finalize repo skeleton,
- define manifest schemas,
- choose dogfood app.

### Phase 1: prove the compiler spine

- parse TSX/html templates,
- build semantic UI graph,
- emit static HTML,
- emit binding manifest,
- emit `fw explain` output,
- run golden tests.

### Phase 2: prove interaction without hydration

- add signal cells,
- add event graph,
- add lazy handler chunk,
- add tiny runtime loader,
- resume one stateful interaction.

### Phase 3: prove web-app usefulness

- forms,
- server action,
- native fallback,
- validation diagnostics,
- resource load,
- streaming boundary.

### Phase 4: prove interop

- custom element output,
- plain HTML consumption,
- Lit/React adapter experiments,
- CSS strategy,
- package build.

## Final assessment

You have not missed a better feature. The feature thesis is already broad enough.

The major missing piece is **project containment**:

```txt
What exactly proves the thesis?
What must not be built yet?
What are the compiler/runtime contracts?
What are the default budgets?
What are the policies for compatibility, security, accessibility, and release?
What fixture app tells us whether the system is real?
```

Answer those before beginning development. Then start small and make the compiler explain everything from day one.
