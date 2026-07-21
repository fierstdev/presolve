# Phase L revised delivery roadmap

**Status:** Authoritative sequencing amendment

**Date:** 2026-07-21

**Prerequisites:** the Phase K freeze; the Phase L constitution; the accepted
L1--L10-A commits; and every frozen Phase A--K contract.

This roadmap supersedes the sequencing in `PHASE_L_COMPLETION_EXECUTION_PLAN.md`
and the heading-level implementation order in `PHASE_L_SLICES_L11_L20.md`.
It does not change the Phase L constitution, package/CLI specification, or any
frozen compiler, runtime, diagnostic, artifact, or L3--L8 contract. A later
slice-specific implementation contract may add detail only when it is
consistent with this roadmap.

## 1. Decision

Phase K completed a deterministic compiler and runtime, not a blank platform.
L1--L10-A have already added a public identity, repository hygiene, canonical
platform/service/incremental/cache/workspace/watch products, a strict CLI
front end, and a transport-neutral schema registry. The remaining work is not
a linear collection of command names or public-facing documents. It is the
careful conversion of those products into supported, versioned public
surfaces.

The earlier L11--L20 headings remain the product destination, but are too
coarse to authorize implementation. In particular:

- `presolve.build-trace`, `presolve.compile-cost-report`, and
  `presolve.artifact-graph` are deliberately **reserved** by L10-A. A command
  cannot expose one before a canonical producer, serializer, fixture, and
  compatibility proof exist.
- The current L3--L8 products expose workspace, cache, scheduling, and session
  facts. They do not by themselves establish every editor query required for
  hover, completion, rename, references, or source mapping. An IDE must not
  recreate that knowledge by reparsing source.
- Phase K deliberately excludes wall-clock performance gates. Benchmark and
  profile output therefore cannot be canonical compiler truth. Deterministic
  structural cost reports and non-canonical sampled telemetry must remain
  separate.
- The L2 amendment correctly forbids speculative directories and package
  shells. New public packages, examples, web content, or release automation
  appear only with a concrete owner, artifact, test, and distribution use.

The governing rule is consequently:

> prove that a frozen product supplies the fact; if it does not, author the
> smallest new immutable product contract before implementing a consumer.

No tool, editor integration, documentation generator, or release workflow may
become a second compiler.

## 2. Current baseline and truthful support boundary

| Area | Accepted state | Public support at the next boundary |
| --- | --- | --- |
| Phase K | K0--K21 frozen | Language, diagnostics, generated artifacts, runtime, production policy, and Phase K budget baselines are compatibility inputs. |
| L1--L8 | Accepted | Presolve identity; L3 products; L4 durable host; L5 in-memory reuse; L6 complete-result cache; L7 explicit workspace coordination; L8 caller-driven watch orchestration. |
| L9 | Accepted foundation | Strict `presolve.json`, explicit build/check, cache/clean, single-project workspace, one-shot watch, deterministic help/version, and explicit reserved-command failure. Legacy compiler commands remain frozen compatibility paths, not new platform adapters. |
| L10-A | Accepted | Registry/negotiation v1 exposes existing products as `available` and future trace/cost/artifact products as `reserved`; it changes no L3--L8 bytes. |
| L10-B | Not started | Compatibility proof and midpoint audit. L11 must not begin first. |

`create`, `dev`, `inspect`, `trace`, `graph`, `profile`, `benchmark`, and
`doctor` are not public capabilities merely because their names appear in help.
Until their individual acceptance gates pass, their deterministic tooling-error
exit is the correct behavior. This is a support-status correction, not a
removal of the permanent command namespace.

## 3. Invariants for every remaining slice

1. **One authoritative producer.** A public result identifies its producer,
   schema/version, source product identities, and supported compatibility
   range. Consumers project; they do not derive replacement facts from source,
   names, DOM, paths, timestamps, or process-local state.
2. **No accidental persistence.** L4/L6 durable state remains source-free;
   L5 reuse remains session-local; L7 input remains caller-complete; L8 remains
   observer-driven. Tooling and editor products do not widen those boundaries.
3. **Canonical data versus observations.** Canonical JSON is newline-terminated,
   ordered, versioned, and fixture-backed. Wall-clock time, machine memory,
   process identifiers, and host paths are observations only: excluded from
   canonical bytes, compatibility identities, and deterministic goldens.
4. **Compatibility before availability.** A registry entry changes from
   `reserved` to `available` only in the same commit as its producer, strict
   reader, rejection behavior, fixtures, old-byte proof, and documentation.
5. **Supported means tested.** A command/package/feature has a help, exit-code,
   human-output, JSON-output where applicable, negative, determinism, and
   documentation test before it leaves `reserved`.
6. **No phantom packages.** A package is introduced only with an owned API,
   dependency direction, test target, and distribution role. Empty directory
   structure is not progress.
7. **Slice discipline.** Each implementation slice has a focused verifier that
   joins `just check`, updates the progress log and handoff with exact results,
   commits atomically, and returns the worktree clean. Broader release gates
   run only when the touched boundary requires them.

## 4. Delivery spine

The following is the complete remaining Phase L sequence. “Contract” is an
authored, reviewed specification slice: it is not permission to fill a missing
product with inferred behavior.

### Gate 0 -- L10-B: compatibility and midpoint freeze

**Goal:** complete the schema-only midpoint without introducing a tooling
product.

1. Add registry request/response compatibility fixtures and rejection fixtures.
2. Prove every L3--L8 canonical fixture remains byte-identical before and
   after L10-A; prove reserved schemas are un-negotiable and have no decoder.
3. Audit imports, durable trees, cache keys, session files, and CLI command
   dispatch for accidental schema-registry coupling.
4. Record a midpoint inventory: available product, owner, canonical reader,
   public consumer, persistence class, and absent capability. Freeze it as the
   input to L11/L12.

**Exit:** L10 has no producer beyond its registry; `just check`, inherited
L3--L9 audits, the compatibility corpus, and the Phase K regression gate pass.
Stop before L11.

### Gate 1 -- L11-A through L11-C: product-backed developer tooling

**L11-A -- tooling capability contract.** Map each requested command to a
specific existing product and field. Separate “available now”, “requires a new
immutable product”, and “not an alpha capability”. Author the exact input
reference format, source-free provenance, JSON schema, text rendering rules,
and error taxonomy for projectors. This contract must state that a projector
never opens source files or invokes an alternate compile path.

**L11-B -- existing-product readers.** Implement read-only, negotiated readers
for the available L3--L8 products and explicit product references. Validate
schema/name/version/identity and reject unavailable, malformed, mismatched, or
unprovenanced input. No command is activated in this slice.

**L11-C -- inspect and graph.** Activate only views backed by L3 workspace
snapshot/graph, L4 session inspection, L5 incremental plan/report, L6 cache
inspection, L7 workspace plan, and L8 execution report. Graph rendering is a
deterministic projection (canonical JSON plus stable text/DOT if supplied),
not a graph reconstruction. Existing source-oriented legacy inspection remains
separate compatibility behavior until explicitly migrated.

**Exit:** `inspect` and the supported subset of `graph` have fixtures for every
available input, reverse-order determinism, malformed input, version rejection,
human/JSON parity, and exit code 6 for unsupported views.

### Gate 2 -- L11-D through L11-G: new tooling products and command activation

**L11-D -- trace and structural-cost contract.** Before code, define the
minimal immutable build-trace and compile-cost products. A trace may report
canonical phase/scheduling/publication events and product identities, never
source text, wall-clock timestamps, paths, or hidden retained state. A cost
report contains deterministic structural counts and the existing Phase K
artifact/cost facts. It must explicitly distinguish canonical cost from sampled
telemetry.

**L11-E -- artifact-graph contract.** Define the artifact graph as a compiler
or service-produced immutable product with exact source product provenance;
do not derive it by inspecting generated files. Decide whether a given fact is
already in Phase K reports or requires a narrow constitutional amendment.

**L11-F -- producers, registry, and readers.** Implement only the products
approved by L11-D/E. Each gets a canonical encoder/strict decoder, versioned
fixtures, validation, identity/provenance checks, reverse-input determinism,
L3--L8 byte-preservation proof, and an atomic L10 registry availability change.
The trace/cost/artifact products cannot be cached or persisted more broadly
than their producer contract permits.

**L11-G -- explain, trace, profile, benchmark, doctor.** Activate commands one
at a time as projectors of L11-F or existing immutable products. `explain`
reports only supplied compiler decisions. `doctor` reports deterministic
validation of supplied configuration/workspace/schema/cache facts and never
discovers a project. `profile` emits canonical structural cost; optional
sampled telemetry is clearly non-canonical and machine-labeled. `benchmark`
uses declared corpus/repetition/environment manifests; it never creates a
performance compatibility gate or claims equal timings across hosts.

**Exit:** every activated command has compatibility, negative, output, and
documentation evidence. Commands with no canonical product remain reserved;
the command registry and help text say so accurately.

### Gate 3 -- L12: language-service capability, not a second analyzer

**L12-A -- editor capability audit.** For hover, definition, references, rename,
diagnostics, symbols, semantic tokens, completion, signature help, and source
mapping, identify the exact compiler-owned fact, position model, identity, and
incremental invalidation source. The audit must prove whether current products
are sufficient. A missing fact is a blocker, not a license to scan or parse.

**L12-B -- query-product amendment and contract (conditional).** If the audit
finds a gap, author the smallest immutable compiler-produced query snapshot
and a constitutional amendment before implementation. It must have stable
source provenance and spans, schema/version/identity, canonical ordering,
privacy/persistence rules, query-specific diagnostic behavior, and Phase K
compatibility proof. If the amendment is not accepted, L12 stops here and the
affected editor feature remains unavailable for alpha.

**L12-C -- `@presolve/language-service` API.** Implement a read-only API over
approved products only. It validates a product reference then returns a
deterministic query response; it owns no parser, binder, semantic cache, or
independent diagnostics. Exercise cold/update/cross-package cases through
producer-provided incremental products, not fabricated source changes.

**L12-D -- LSP adapter.** Translate the approved language-service API to LSP
requests/responses without changing result ordering, ranges, error categories,
or cancellation semantics. Protocol framing, client capabilities, and
unsupported-method behavior receive their own contract and fixture suite.

**L12-E -- `@presolve/vscode`.** Introduce the extension only after the LSP
adapter is stable. It depends solely on the language-service/LSP package,
contains no language analysis, and is tested against a pinned editor fixture.

**Exit:** every advertised editor feature has compiler-parity fixtures, stable
unsupported behavior, source-order determinism, and no independent-analysis
imports. The extension is optional alpha distribution material, not a hidden
compiler dependency.

### Gate 4 -- L15 then L14: public test foundation before examples

The earlier plan placed examples before their public test harness. Reverse
that order.

**L15-A -- test-contract inventory.** Define package/test layers and map the
existing Rust fixtures, browser probes, L3--L11 product fixtures, and Phase K
production corpus into a public, non-duplicative matrix.

**L15-B -- `@presolve/testing`.** Introduce only the fixtures/utilities that
compile, validate canonical bytes, exercise CLI contracts, and run declared
browser/workspace cases. It may wrap compiler products; it may not implement
language semantics or weaken Phase K budgets.

**L15-C -- reproducibility lanes.** Split CI into deterministic contract lanes,
browser/runtime lanes, documented example lanes, and non-blocking benchmark
observation lanes. Each lane has pinned inputs, clear artifacts, and a local
reproduction command. Host timing never gates correctness.

**L14-A -- example contract.** Specify a small alpha corpus rather than ten
unverified applications: Counter, a component/Context/Slots example, a Forms
example, a workspace example, and a production/resume demonstration. Each
maps explicitly to frozen Phase H--K behavior and declares unsupported
features. Additional examples wait for post-alpha releases.

**L14-B -- canonical examples.** Add those examples one at a time with a
declared `presolve.json`, explicit source list/workspace input, build/check
fixtures, browser/runtime proof where applicable, expected product identities,
and documentation snippet tests. A scaffold template (`presolve create`) may
be extracted only after one canonical example is proven reproducible.

**Exit:** every shipped example is continuously built and tested with the same
public command path; no example depends on unpublished or reserved behavior.

### Gate 5 -- L13: documentation as a tested public interface

**L13-A -- information architecture.** Replace the archive-facing navigation
with a public docs index that links to frozen contracts but labels them as
reference material. Establish docs ownership, version policy, command/source
snippet format, and generated-reference boundaries.

**L13-B -- first-use path.** Document install, version, explicit configuration,
build/check, cache/clean, workspace, watch-once, diagnostics, and known
limitations using only accepted L9/L11 behavior.

**L13-C -- language and architecture reference.** Derive State, Actions,
Computed, Context, Components, Slots, Forms, resumability, production
optimization, runtime, platform, and cache/workspace pages from frozen Phase
H--K/L3--L8 contracts. Documentation summarizes rather than redefines them.

**L13-D -- generated command/schema reference.** Generate or fixture-validate
command help, exit codes, product schemas, compatibility tables, and supported
status from the same registry used by the CLI. No hand-maintained command list
may claim a reserved feature is available.

**Exit:** a link/snippet/command-validation job passes; examples, JSON schema
names, package names, and support statuses are all current.

### Gate 6 -- L16 and L17: public repository and reproducible release system

**L16 -- community readiness.** Add only legally and operationally complete
assets: license decision, changelog policy, contributing/security guidance,
code of conduct, issue/PR forms, support boundaries, and public README. Audit
for credentials, internal-only material, generated files, broken archive
links, and inaccurate support claims.

**L17-A -- package/distribution contract.** Before publishing, define which
packages have real artifacts, exports, dependency direction, synchronized
versioning, checksums/provenance, and install tests. Do not publish a package
solely because it appears in the constitution.

**L17-B -- CI and release automation.** Add reproducible build/test/docs/example
workflows, schema-compatibility gates, changelog/version validation, and a
release dry run that produces but does not publish artifacts. Secrets, GitHub
publication, registry publication, and signing are external-authority steps;
their workflow must fail closed when absent.

**Exit:** a clean checkout can reproduce every claimed release artifact and
its verification evidence without network-side publication.

### Gate 7 -- L18 and L19: launch content and alpha rehearsal

**L18 -- website content bundle.** Produce versioned, link-checked content for
presolve.dev (home, docs, examples, architecture, benchmarks methodology,
roadmap, GitHub, and a clearly non-functional playground placeholder). Deploy
only through separately authorized hosting work; Phase L owns content and
validation, not an assumed web platform.

**L19-A -- alpha manifest.** Freeze the alpha support matrix: available
commands/products/editor features/packages, known limitations, migration from
EdgeZero terminology, compatibility policy, contribution/support policy, and
rollback criteria.

**L19-B -- clean-room rehearsal.** From a fresh checkout and a clean install,
run the supported create path if it exists; otherwise run the documented
manual starter path. Build/check every alpha example, exercise cache/workspace/
watch/product tools, verify package metadata, and compare generated artifacts
to the frozen Phase K corpus. Record exact environment-independent evidence.

**Exit:** the alpha may be proposed for publication; actual publication still
requires the repository/registry/hosting authority named in L17/L18.

### Gate 8 -- L20: platform freeze

Run a final, fresh-checkout reproducibility matrix over frozen Phase A--K
fixtures; L3--L12 product/reader compatibility; CLI help/exit/JSON/human
fixtures; L5/L6/L7/L8 lifecycle cases; browser runtime; examples; docs links
and snippets; package dry-run; release dry-run; and the repository audit.
Compare all canonical outputs byte-for-byte to the committed baselines.

L20 also produces a public API/support table and removes only code proven
obsolete by the final audit. It must not use cleanup to rewrite historical
contracts or discard evidence. Phase L is complete only after this gate is
committed, the tree is clean, and every remaining `reserved` capability is
either delivered or explicitly removed by a constitutional amendment.

## 5. Verification ladder

| Change type | Required focused evidence | Required inherited evidence |
| --- | --- | --- |
| Registry/reader/product | strict schema rejection, canonical fixture bytes, provenance/identity checks, reverse-order determinism | L3--L10 audits and `just check` |
| CLI command | help, exit, stderr/stdout separation, human/JSON output, malformed input, reserved status | producing product verifier, L3--L11 audits, relevant browser proof |
| Language service/editor | compiler-parity corpus, range/order determinism, incremental/cross-package cases, unsupported protocol behavior | query-product verifier, L3--L12 audits, package checks |
| Example/docs | clean public command run, snippet/link/schema validation, declared limitation coverage | example/test lanes and docs validation |
| Release/website | fresh-checkout/reproducible artifact manifest, content link audit, dry-run failure-closed secrets test | full CI matrix and repository audit |

Every completed slice appends its exact commands and results to
`notes/progress/2026-W28.md` and `notes/progress/AGENT_HANDOFF.md`; no
subsequent slice begins on an unclean or unverified baseline.

## 6. Explicit stop rules

Stop and author or request an amendment when a desired capability would:

- require source parsing, binding, semantic reconstruction, source discovery,
  or a second diagnostics implementation outside the compiler;
- require a new canonical product not named by an accepted contract;
- make wall-clock or machine-specific telemetry part of a deterministic result;
- alter Phase K generated bytes, runtime behavior, diagnostics, or frozen
  compatibility products;
- widen L4/L6 persistence, L5 reuse, L7 caller-owned request authority, or L8
  observer/scheduler authority; or
- claim a public package, release, deployment, signing, or registry publication
  without a real artifact and the required external authority.

## 7. Immediate next action

Implement **L10-B only**. Its acceptance artifact is the midpoint capability
inventory described above. Do not activate a reserved command, create a tool
package, add an editor protocol, scaffold an example, or begin L11 until
L10-B passes its compatibility and Phase K regression gates.
