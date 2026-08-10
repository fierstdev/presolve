# Changelog

This repository follows Keep a Changelog-style entries. Changes are recorded
under `Unreleased` and grouped by Added, Changed, Fixed, Deprecated, Removed,
and Security as applicable. A release entry names the released version and
date only after the corresponding artifacts have been verified; this file does
not publish or imply a release.

## Unreleased

## 0.2.0-beta.27 - 2026-08-10

### Added

- Components can declare a canonical client/shared
  `Resource<Data, Error>` with `resource(handler)`. TypeScript authority proves
  the exact `resource` and `ResourceContext` symbols, Promise completion, and
  direct named package import before the compiler admits the endpoint.
- Project-local Vite bundles each selected Resource package module into a
  content-addressed public asset. The compiler publishes that exact location
  in Resource artifact schema v4; application callback source is not shipped
  or evaluated.

### Changed

- Canonical pure getters may derive from Resource `.data`, `.error`, or `.state`
  as well as State. Resource settlement invalidates the exact compiler-planned
  getter and DOM bindings, including in instance-qualified runtime plans.
- The Resource bundling workspace is replaced on every build under
  `.presolve/resource-build/`, so local builds retain only the current scratch
  publication rather than an accumulating build history.
- V2 TypeScript authority request/response schema v14 and authored semantics
  schema v9 retain exact general-Resource invocation and package evidence.

### Fixed

- Public guides and the capability registry now describe the canonical Form,
  Resource, route-loader, and Node server-action surfaces instead of stale
  pre-execution or compatibility-only constraints.

## 0.2.0-beta.26 - 2026-08-09

### Changed

- Computed schema v13, Effect schema v8, and Context schema v3 now preserve
  shared pure-program constant value kinds recursively. Numeric and string
  literals retain distinct compiler-issued representations without changing
  serialized State or resume contracts.

### Fixed

- Computed addition now executes authored TypeScript semantics for numeric
  literals. Expressions such as `this.count + 1` produce `1`, then `2`, rather
  than concatenating serialized number lexemes as `"01"`, then `"11"`.
- The canonical generated counter is covered through compiler artifact,
  production build, and browser interaction evidence so the shipped scaffold
  cannot regress independently of the computed runtime.

## 0.2.0-beta.25 - 2026-08-08

### Added

- Canonical `defineForm(...)` declarations can bind an exact imported Node
  server action with `(FormData, AbortSignal) -> Promise` semantics. The Node
  adapter bundles only compiler-proven exports and executes URL-encoded and
  multipart submissions with typed JSON, redirect, and failure results.
- Route components can declare a canonical server-backed
  `Resource<Data, Error>` with `loader(handler)`. TypeScript authority proves
  the exact Presolve intrinsic, route-parameter and abort types, Promise
  completion, and direct named package import before compiler lowering.
- The Node adapter executes compiler-issued route-loader plan schema v2 through
  a digest-verified server-only Vite registry. It provides strict route
  parameter decoding, exact data/error codecs, public/private/no-store caching,
  request and shutdown cancellation, and schema-v4 Resource bootstrap values.

### Changed

- Form submission hosts now enforce compiler-owned method, origin, media-type,
  body-size, duplicate-submission, request-disconnect, reset, and graceful
  shutdown behavior before invoking application server code.
- Node deployment plan schema v3 inventories both server-action and
  route-loader registries. Private loader responses partition authorization and
  cookies and emit `Vary: Authorization, Cookie`.
- Route-specific publication now excludes sibling routes' Resource
  declarations, allowing multiple dynamic loader routes to publish independent
  browser artifacts.

### Fixed

- Resource bootstrap injection recognizes both stable `runtime.js` and the
  compiler's production content-hashed runtime script. Dynamic loader routes no
  longer fail with a 500 only in production builds.

## 0.2.0-beta.24 - 2026-08-07

### Changed

- Compiler-owned JSON embedded in route documents now uses a compact transport
  encoding while the canonical pretty-printed artifact files and schemas remain
  unchanged. On the production-shaped Presolve.dev corpus this reduces route
  HTML from 33.7 MB to 13.6 MB and the complete publication from 82.3 MB to
  62.3 MB without removing any deployment artifact.
- Application-publication tests prove the compact template-manifest value is
  exactly equivalent to the canonical digest-bound artifact, including safe
  embedded-script escaping. Real-browser proof covers cold boot, Actions,
  Computed updates, and structural tabs from the compact publication.

## 0.2.0-beta.23 - 2026-08-02

### Fixed

- Component-scoped Action updates now filter the application-wide computed plan
  by the active component before resolving instance-qualified computed slots.
  An interactive route no longer raises `PSR_INVALID_COMPONENT_ARTIFACT` merely
  because another route or unmounted component contributes a computed update
  batch to the same application publication.
- The real-browser instance-qualification proof now includes an unmounted
  component with its own State, Action, and computed plan and rejects any
  runtime error dispatched by an otherwise unrelated component action.

## 0.2.0-beta.22 - 2026-08-02

### Fixed

- Newly generated applications declare the audited `esbuild` and `workerd`
  installation scripts in `pnpm-workspace.yaml`, so pnpm 11's strict
  dependency-build policy admits the Vite and Cloudflare toolchains instead of
  blocking `pnpm check`, `pnpm build`, and deployment preparation.
- Clean-room scaffold verification now preserves the generated pnpm policy and
  installs with lifecycle enforcement enabled before check, build, and
  Cloudflare preparation. The release gate can no longer hide this failure with
  `--ignore-scripts`.

### Changed

- Immediate beta installation guidance uses the exact published creator
  version. npm's `latest` tag is still verified independently because pnpm 11's
  default one-day minimum package age intentionally delays unversioned
  resolution of a just-published release.

## 0.2.0-beta.21 - 2026-08-02

### Fixed

- The npm `latest` tag now names the current creator instead of the stale
  `0.2.0-beta.1` release. The beta.21 creator tarball generates exact,
  lockstep beta.21 framework, CLI, and TypeScript-authority dependencies.

### Changed

- GitHub Actions publishes `create-presolve` directly with the `latest` tag,
  avoiding a separate post-publication dist-tag mutation and its registry
  convergence race.
- The release workflow now verifies npm's `latest` tag and executes the exact
  published creator in a clean temporary directory before the VS Code extension
  and GitHub prerelease can publish. This keeps the proof deterministic while
  pnpm 11's default one-day minimum release age intentionally delays
  unversioned resolution of a newly published package.

## 0.2.0-beta.20 - 2026-07-29

### Added

- `presolve dev` now watches authored project inputs and republishes from the
  compiler. CSS-only edits hot-swap the rebuilt canonical stylesheet while
  preserving browser state, focus, scroll position, and the current document.
- Semantic edits rebuild the file-route manifest and use the fail-closed full
  reload boundary until a narrower compiler HMR product proves state
  compatibility.
- Failed development builds keep the last successful publication available and
  render the compiler diagnostic in an accessible browser alert. Correcting the
  source reloads the recovered publication automatically.

### Changed

- Development responses use `Cache-Control: no-store`, the injected development
  client is a CSP-compatible same-origin script, and compiler-owned publication
  stages are excluded from file observation.
- Scaffold and public styling documentation now trace the exact connection from
  component/route classes through `app/app.css`, the compiler-owned document
  head link, content-addressed production output, and development CSS hot swap.

## 0.2.0-beta.19 - 2026-07-29

### Fixed

- Cold-boot and resume component records now retain the authored component
  name used by compiler-owned Computed evaluations. Initial Computed bindings
  no longer render `undefined` until the first Action when an application shell
  and route component share one publication.
- Document-template placeholders are replaced only at their validated template
  positions. Application content can now display literal `{{ head }}`,
  `{{ app }}`, and `{{ runtime }}` examples without consuming compiler payloads
  or moving the runtime into a code block.

## 0.2.0-beta.18 - 2026-07-29

### Added

- Package Actions now forward exact `string`, `number`, `boolean`, and `null`
  arguments and support compiler-owned cancellable `Promise<void>` completion,
  replacement, teardown/pagehide abort, stale-settlement suppression, stable
  failure evidence, and restore without replay.
- `create-presolve` now generates the complete canonical document, application
  shell, global stylesheet, public asset, and interactive route structure with
  CLI help/version behavior, accessible mobile-first presentation, and
  comprehensive ownership and Vite guidance.
- Public project-structure, styling, Tailwind, Vite, asset, and third-party
  package guides define the exact automatic, adapter-owned, and unsupported
  boundaries.

### Changed

- Atomic application and file-route publication now retains only the active
  hidden release directory while preserving failed-build rollback and
  caller-owned lookalike directories.

### Fixed

- Instance-qualified Computed initialization now evaluates only programs owned
  by the active component definition. A shared application shell can no longer
  abort a route containing Computed values with
  `PSR_INVALID_COMPONENT_ARTIFACT`.

## 0.2.0-beta.17 - 2026-07-28

### Added

- Decorator-free Actions can invoke an exact named import from an ordinary
  installed package. The compiler proves import identity, Vite publishes a
  deterministic package-call registry, and the browser executes the admitted
  call after the compiler-owned state batch on both cold activation and
  structural resume.
- Package-call artifacts, runtime evidence, and stable diagnostics now
  distinguish compiler-owned semantic calls from packages used only as
  ordinary build inputs. Aliases retain authority, while local lookalikes,
  unproven calls, missing registries, and non-callable exports fail closed.
- The `examples/package-interop` application provides reproducible build and
  real-browser proof for third-party package interoperability.

## 0.2.0-beta.16 - 2026-07-28

### Fixed

- Conventional application documents now link content-addressed
  `app.<sha256>.css` and route-local `runtime.<sha256>.js` artifacts. Stable
  `/app.css` and `runtime.js` compatibility files remain in the compiler-issued
  inventory, while returning Safari clients can no longer pair current HTML
  with stale or previously failed external publication bytes.

## 0.2.0-beta.15 - 2026-07-28

### Fixed

- Conventional file-route publication now prunes sibling route roots before
  runtime products are derived. Component targets, bindings, events, state,
  and resume records remain local to the selected layout/route instance tree,
  preventing `PSR_INVALID_ORDINARY_TARGET` when another route owns
  interactivity.

## 0.2.0-beta.14 - 2026-07-28

### Added

- Canonical decorator-free `defineForm({ fields, submit })` authoring now
  lowers nested Fields, compiler-owned value/checked/file bindings, built-in
  and Standard Schema validation, native submission hosts, and stable Form
  resume products without legacy decorators.
- Imported async Form submission capabilities now resolve through exact
  semantic-package contracts, publish deterministic callable registries, and
  receive compiler-built nested values plus submission-owned AbortSignals.

### Changed

- Forms runtime artifact schema v6 records exact Standard Schema validators
  and integrity-bound submission capabilities. Browser execution validates
  before submit, suppresses duplicate calls, rejects stale async validation,
  preserves file fields as cold-only resume state, and reports explicit
  Completed, Failed, Cancelled, Invalid, and reset lifecycle outcomes.

## 0.2.0-beta.13 - 2026-07-28

### Fixed

- Resumable event activation now snapshots the exact target and event type
  before awaiting its lazy interaction chunk. Mobile Safari and other engines
  may release live event-path state after synchronous dispatch; deferred
  actions now retain compiler-authorized event authority and execute reliably.
- Canonical `app/app.css` links now carry the stylesheet's SHA-256 content
  identity. Browsers revalidate a distinct URL whenever global CSS changes,
  preventing stale or previously failed mobile caches from leaving otherwise
  valid application HTML unstyled after a release.

## 0.2.0-beta.12 - 2026-07-28

### Fixed

- Parenthesized JSX conditionals with an explicit `null` false branch now
  remain compiler-owned structural hosts. Decorator-free actions can
  materialize `condition ? (<Element />) : null` content in the browser
  instead of updating adjacent bindings while silently omitting the branch.

## 0.2.0-beta.11 - 2026-07-28

### Fixed

- File-scoped `presolve check <file> --format json` requests made inside a
  canonical application now use the same TypeScript-authority-backed,
  decorator-free project assembly as workspace checks and production builds.
  Editor diagnostics no longer redirect `extends Component`, V2 action fields,
  or Slot layouts through the legacy decorator graph.

## 0.2.0-beta.10 - 2026-07-28

### Added

- The VS Code extension now uses the project's installed Presolve compiler for
  exact on-save diagnostics, component CodeLens, source explanation, workspace
  checks, production builds, doctor output, and release-train status. It keeps
  TypeScript language behavior with the workspace TypeScript server and does
  not introduce a parallel TSX analyzer.

## 0.2.0-beta.9 - 2026-07-28

### Fixed

- Static JSX now preserves a single authored leading or trailing inline-space
  boundary between adjacent text and element nodes. This keeps compiler-emitted
  source examples valid and readable while retaining multiline formatting
  normalization.

## 0.2.0-beta.7 - 2026-07-27

### Fixed

- JSX text now decodes standard HTML character references before compiler
  publication escapes the resulting text. Literal code examples such as
  `&lt;button&gt;`, quoted attributes, braces, and other named or numeric entities
  render correctly in generated static HTML instead of appearing double-escaped.

## 0.2.0-beta.6 - 2026-07-27

### Fixed

- `presolve explain` now recognizes the canonical decorator-free
  `class … extends Component` form, reports it as a component, and no longer
  emits the legacy decorator-only `PS0100` warning.

## 0.2.0-beta.5 - 2026-07-27

### Fixed

- Unsupported-platform CLI diagnostics now report the installed Presolve beta
  version instead of the retired `0.1 alpha` label.
- The public capability matrix and guides now distinguish canonical
  decorator-free V2 source from retained alpha-compatibility decorators.

## 0.2.0-beta.4 - 2026-07-27

### Fixed

- Cloudflare Workers Static Assets deployments now consume compiler-internal
  canonicalization redirects for every authored route, including nested routes
  such as `/docs/`, without exposing `/routes/...` artifact paths.

## 0.2.0-beta.3 - 2026-07-27

### Added

- Canonical application files: `app/app.tsx`, automatic `app/app.css`, and a
  strict `app/index.html` document template with compiler-owned `head`, `app`,
  and `runtime` placeholders. The former `app/layout.tsx` and `styles/` paths
  remain beta compatibility inputs.

### Fixed

- Cloudflare Workers Static Assets deployments now consume internal asset
  canonicalization redirects inside the worker, so `/` remains `/` instead of
  exposing the internal `/routes/root/` artifact path.

## 0.2.0-beta.2 - 2026-07-27

### Fixed

- Decorator-free `Component` layouts can declare V2 slot fields with
  `children: SlotContent = slot()` through the published TypeScript authority
  bridge, without falling back to the legacy `@component()` diagnostic.
- `styles/` and `public/` are copied atomically into `dist/`, integrity-listed
  for deployment, and served by development and Node static hosts.

## 0.2.0-beta.1 - 2026-07-26

### Added

- Completion-grade V2 structural lifecycle, context, form, resource, route
  handoff, capability, and beta hardening evidence.
- The explicit closed beta Action surface, packed-scaffold verification, and
  tag-triggered beta publication workflow.

### Changed

- The compiler, framework, tooling, scaffold, native CLI packages, and VS Code
  extension now identify the `0.2.0-beta.1` compatibility train.

## 0.1.0-alpha.1 - 2026-07-23

### Added

- Public `presolve`, `@presolve/cli`, and `create-presolve` release surfaces.
- A TypeScript 7 starter project and installable `presolve-vscode` extension
  package.
- A tag-gated npm/VS Code Marketplace release workflow with native CLI package
  staging.
- Public alpha documentation for framework authoring, metaframework workflow,
  tooling, Cloudflare deployment, and release operations.

### Changed

- The compiler, framework, application workflow, tooling, and release train
  identify `0.1.0-alpha.1`.
