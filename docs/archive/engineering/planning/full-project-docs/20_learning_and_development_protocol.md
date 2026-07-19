# Learning and Development Protocol

Status: **strict working policy**  
Audience: solo founder/developer or very small founding team  
Purpose: make manual learning and implementation sustainable without letting the project sprawl.

This project is too broad to build by intuition. The development process must force locality, proof, and documentation. Every feature must leave behind enough context that future work can resume without reconstructing the reasoning from memory.

## 1. Prime directive

Build one verified semantic slice at a time.

A semantic slice is a feature that proves a compiler behavior end-to-end:

```txt
source code
  -> parsed representation
  -> semantic graph node(s)
  -> generated artifact(s)
  -> manifest/explain output
  -> runtime/browser behavior where applicable
  -> fixture test
  -> documentation note
```

Do not build disconnected subsystems for long periods. For example, do not spend weeks building a full parser abstraction, runtime scheduler, router, or dev server before a real fixture proves that the compiler/runtime contract works.

## 2. Non-negotiable rules

### 2.1 Documentation rules

1. Every meaningful technical decision must be captured in either an ADR, RFC, design note, learning note, or fixture README.
2. If a decision affects user-facing syntax, generated output, compiler diagnostics, runtime contracts, or package layout, it requires an ADR or RFC.
3. If a topic required learning, create a learning note before or during implementation.
4. Documentation must include examples, not only prose.
5. Any new compiler behavior must include an `fw explain` expectation, even before `fw explain` is fully implemented.
6. Any new diagnostic must include a bad example, the expected message, and a corrected example.
7. Any new runtime behavior must include a browser-level fixture or a clearly stated reason why browser testing is not applicable yet.
8. Never document aspirational behavior as if it works. Mark it as `planned`, `prototype`, `implemented`, or `verified`.
9. Do not delete old reasoning silently. Supersede it with a note that links to the replacement.
10. Every public-facing statement must distinguish between product promise, current implementation, and future target.

### 2.2 Coding rules

1. Code must be boring unless the compiler model demands otherwise.
2. Prefer explicit data structures over clever control flow.
3. Prefer typed intermediate representations over strings, maps, or loosely shaped JSON.
4. Compiler phases must be separately testable.
5. Generated code must be snapshot-tested against fixtures.
6. Runtime code must remain small, dependency-light, and measurable.
7. Do not add a runtime feature to compensate for missing compiler knowledge without recording the debt.
8. Do not use `unsafe` Rust in MVP code unless an ADR approves it.
9. Do not allow TypeScript `any` in production packages unless a comment names the boundary and reason.
10. Do not introduce global mutable state in compiler crates unless it is isolated, tested, and documented.
11. Every public Rust function that belongs to a compiler phase should have either a doc comment or be module-private.
12. Every public TypeScript API must have a usage example or a test that acts as one.
13. Errors are product surface. Compiler and CLI errors must be written for a user, not for the implementer.
14. If the compiler cannot explain an optimization, the optimization is not ready.
15. If the runtime cannot validate the manifest shape, the manifest format is not ready.

### 2.3 Movement rules

You may not move to a new major area because the current one feels tedious. You may move only when one of these is true:

1. The current area meets its exit criteria.
2. The current area is blocked by a clearly documented dependency.
3. A time-boxed spike produced enough evidence to stop or pivot.
4. A bug or architecture flaw invalidates the current approach and an ADR records the change.

The default rule: finish the slice before changing areas.

## 3. Documentation system

Use four document types with strict meanings.

### 3.1 ADR: Architecture Decision Record

Use for decisions that constrain implementation.

Location:

```txt
adr/0001-decision-name.md
```

Required template:

```md
# ADR 0001: Title

Date:
Status: Proposed | Accepted | Superseded
Supersedes:
Superseded by:

## Context

What problem forced the decision?

## Decision

What are we doing?

## Consequences

What becomes easier?
What becomes harder?
What constraints follow?

## Alternatives considered

What did we reject and why?

## Verification

How will we know this decision worked?
```

ADR examples:

```txt
ADR: Use Oxc parser as initial TSX parser front-end
ADR: Represent template dynamics as binding graph nodes
ADR: Use JSON schema for compiler/runtime manifests
ADR: No global hydration baseline
ADR: Native custom-element output is an output target, not the entire mental model
```

### 3.2 RFC: Product or syntax proposal

Use for syntax, semantics, compiler behavior, or public API.

Location:

```txt
rfcs/0001-feature-name.md
```

Required template:

```md
# RFC 0001: Title

Status: Draft | Accepted | Rejected | Implemented
Owner:
Related ADRs:
Related fixtures:

## Summary

## Motivation

## User-facing syntax

## Semantics

## Compiler graph impact

Template graph:
Reactive graph:
Event graph:
Resource graph:
Accessibility graph:
Server/client graph:
Style graph:
Debug graph:

## Generated output

## Diagnostics

## `fw explain` output

## Runtime requirements

## Migration story

## Open questions
```

No user-facing feature may be implemented without at least a stub RFC.

### 3.3 Learning note

Use for new concepts you study.

Location:

```txt
docs/learning/<area>/<topic>.md
```

Required template:

```md
# Topic

Date:
Area:
Project relevance:
Primary resources:

## Concepts I need

## Concepts I do not need yet

## Summary in my own words

## Implementation implications for EdgeZero

## Questions remaining

## Toy experiment

## Decision or next step
```

The most important section is **Concepts I do not need yet**. It prevents research from becoming procrastination.

### 3.4 Fixture README

Every fixture directory must explain the behavior it proves.

Location:

```txt
fixtures/<area>/<fixture-name>/README.md
```

Required template:

```md
# Fixture name

## Purpose

## Source input

## Expected compiler graph

## Expected generated artifacts

## Expected diagnostics

## Expected runtime behavior

## Expected explain output

## Known limitations
```

## 4. Work item protocol

Every task should use this shape.

```md
# Work item: title

## Area

compiler | runtime | cli | docs | examples | packaging | testing | devtools

## Goal

One sentence.

## Non-goal

What this task will not solve.

## Learning needed

Specific concepts only.

## Fixture

Path to fixture or example.

## Implementation plan

Small steps.

## Tests required

Unit:
Snapshot:
Browser:
Size:
Diagnostic:

## Exit criteria

Checklist.
```

A work item is too large if the fixture cannot be described in one paragraph.

## 5. Git rules

### 5.1 Branching

Use short-lived branches.

```txt
main
feature/<area>-<short-name>
fix/<area>-<short-name>
docs/<topic>
spike/<question>
```

Rules:

1. `main` must always compile.
2. `main` must always pass the default test suite.
3. Spikes must not merge unless converted into production code or documented as evidence.
4. Branches should target one semantic slice.
5. Delete branches after merge.

### 5.2 Commit messages

Use conventional prefixes, but keep them project-specific.

```txt
docs: add learning note for custom elements lifecycle
adr: choose oxc as parser front-end
rfc: propose state field semantics
compiler: parse component decorator metadata
ir: add binding node representation
runtime: add delegated click dispatch
fixture: add counter lazy click fixture
test: snapshot explain output for counter
cli: add explain command skeleton
build: wire cargo and pnpm workspace checks
```

Rules:

1. One conceptual change per commit.
2. Do not mix unrelated docs cleanup with compiler behavior.
3. Code commits should include tests unless explicitly marked as scaffolding.
4. Learning commits are allowed and useful.
5. Avoid commit messages like `wip`, `stuff`, `fix`, or `changes`.

### 5.3 Pull request checklist

Even if working alone, open pull requests locally or in GitHub to force review structure.

Required PR checklist:

```md
## What changed?

## Why?

## What fixture proves it?

## What did I learn?

## What is intentionally not solved?

## Tests run

- [ ] cargo test
- [ ] cargo clippy
- [ ] pnpm test
- [ ] pnpm typecheck
- [ ] e2e/browser test if applicable

## Documentation updated

- [ ] ADR/RFC if needed
- [ ] learning note if needed
- [ ] fixture README if needed
- [ ] user-facing docs if behavior changed
```

## 6. Phase gates

The project should advance through gates. Do not skip gates.

### Gate 0: Operating base

Goal: establish the workspace, documentation rules, and first fixtures.

Required outputs:

```txt
Cargo workspace
pnpm workspace
justfile or xtask commands
ADR template
RFC template
learning-note template
fixture README template
first counter fixture stub
CI skeleton
```

Exit criteria:

```txt
repo can be cloned and checked with one command
Rust and TypeScript formatting are wired
empty compiler CLI can run
one fixture exists with expected output placeholders
```

### Gate 1: Source ingestion

Goal: parse one simple component source file and preserve source spans.

Required learning:

```txt
Rust ownership basics
AST representation
source spans
error reporting basics
TSX parse structure
```

Exit criteria:

```txt
parse a component file
extract component name
extract render method or template
preserve line/column spans
emit structured parse diagnostics
snapshot parsed summary
```

Do not move to runtime work before this gate unless runtime exploration is a time-boxed spike.

### Gate 2: Template graph

Goal: represent static DOM and dynamic bindings.

Required learning:

```txt
HTML tree semantics
JSX expression boundaries
DOM text/attribute/property distinction
source maps
```

Exit criteria:

```txt
static elements represented in IR
dynamic text binding represented in IR
dynamic attribute binding represented in IR
invalid template cases produce diagnostics
fixture snapshots graph and explain output
```

### Gate 3: Reactive graph

Goal: connect state reads to bindings.

Required learning:

```txt
fine-grained reactivity
dependency graph construction
class fields and method capture analysis
assignment/update semantics
```

Exit criteria:

```txt
state field declared
binding reads state field
event handler writes state field
compiler emits dependency edge
explain output names dependency
invalid writes produce useful diagnostic
```

### Gate 4: Code generation without resumability

Goal: generate static HTML and minimal client patch code for one local interaction.

Required learning:

```txt
DOM patching
event delegation
code generation
module chunk boundaries
browser e2e testing
```

Exit criteria:

```txt
counter SSR HTML emitted
click handler updates exact text node
no VDOM runtime
browser test passes
size report exists
```

### Gate 5: Manifest contract

Goal: formalize compiler/runtime communication.

Required learning:

```txt
schema design
versioning
runtime validation
backward compatibility
```

Exit criteria:

```txt
component manifest schema exists
runtime validates manifest
fixtures snapshot manifests
schema changes require changelog note
```

### Gate 6: Lazy interaction and resumability prototype

Goal: event code loads only when the user interacts.

Required learning:

```txt
dynamic import
event serialization
closure capture constraints
state serialization
HTML as continuation format
```

Exit criteria:

```txt
initial page excludes handler body
click triggers lazy import
runtime resolves handler symbol
state resumes from serialized payload
explain output identifies lazy chunk
browser test verifies network/load behavior
```

### Gate 7: Forms/actions MVP

Goal: native form fallback plus enhanced submit.

Required learning:

```txt
HTML forms
FormData
server actions
CSRF basics
validation and error association
progressive enhancement
```

Exit criteria:

```txt
form posts without JS
form enhances with JS
server action boundary enforced
field error associates with input
accessibility diagnostics run
browser test covers no-JS and JS modes
```

### Gate 8: Accessibility compiler MVP

Goal: statically detect a small set of high-confidence accessibility errors.

Required learning:

```txt
accessible names
form labels
button names
ARIA validity
keyboard interaction basics
WCAG 2.2 AA structure
```

Exit criteria:

```txt
button accessible-name diagnostic
input label diagnostic
invalid aria attribute diagnostic
clickable div diagnostic
fixture includes bad and fixed examples
```

### Gate 9: Web Component output

Goal: package a component as a standards-native custom element.

Required learning:

```txt
custom element lifecycle
attributes vs properties
shadow DOM
slots
parts
constructable stylesheets or style injection tradeoffs
```

Exit criteria:

```txt
custom element registers
attributes map to props
slot behavior documented
generated package can be consumed in plain HTML
browser test passes in at least Chromium and WebKit
```

### Gate 10: Alpha readiness

Goal: prove a real small app.

Exit criteria:

```txt
one dogfood app uses routing, resource, action, form, lazy interaction, and a11y diagnostics
fw explain works on every dogfood component
fw size reports interaction chunks
package publishing dry-run succeeds
known limitations are documented
```

## 7. Area ownership rules

### 7.1 Compiler core

Allowed work:

```txt
parsing
semantic graph
IR
diagnostics
source maps
code generation
manifest generation
explain output
```

Rules:

1. Compiler phases must be pure where practical: input data in, structured output out.
2. Do not read the filesystem deep inside semantic passes. Pass resolved source units in explicitly.
3. Do not mix diagnostics with printing. Diagnostics are data; rendering is CLI responsibility.
4. Every IR node must have a reason to exist.
5. Every IR node that maps to source must carry span data.
6. Every generated artifact must be traceable to source.

### 7.2 Runtime

Allowed work:

```txt
signal engine
event delegation
DOM binding patching
lazy import resolver
manifest validation
optional custom-element upgrader
```

Rules:

1. Runtime does not discover application structure by scanning arbitrary DOM if the compiler can provide a manifest.
2. Runtime should not require component-wide hydration for local interaction.
3. Runtime must fail loudly on incompatible manifest versions.
4. Runtime must be size-tracked.
5. Runtime APIs are internal unless intentionally documented as public.

### 7.3 CLI

Allowed work:

```txt
fw dev
fw build
fw check
fw explain
fw size
fw a11y
fw trace
```

Rules:

1. CLI output must be stable enough for snapshot testing.
2. Machine-readable output should exist for core commands.
3. Human-readable output should be concise and actionable.
4. Every error should include file, span, cause, and repair suggestion where possible.

### 7.4 Examples

Allowed work:

```txt
counter
form
resource list
streaming dashboard
web component package
```

Rules:

1. Examples are tests, not marketing decorations.
2. Every example must be runnable from a clean checkout.
3. Examples must pin what behavior they prove.
4. Do not add examples for features that are not implemented unless clearly marked as design sketches.

## 8. Learning protocol

### 8.1 The 3-pass method

For every unfamiliar concept:

1. **Read for vocabulary.** Identify terms and mechanics. Do not implement yet.
2. **Build a toy experiment.** Keep it outside production code or inside `spikes/`.
3. **Apply to one project fixture.** Convert learning into compiler/runtime behavior.

A concept is not learned for this project until it changes a fixture, diagnostic, or design note.

### 8.2 Reading limits

Research must be time-boxed.

Default limits:

```txt
small concept: 60-90 minutes
medium concept: 1-2 days
large concept: 3-5 days, with a spike artifact
```

At the end of a time box, record:

```txt
what I understand
what I still do not understand
what assumption I will implement under
what fixture will validate or falsify the assumption
```

### 8.3 Resource ranking

Use resources in this order:

1. Normative specifications.
2. Official documentation.
3. Source code of mature projects.
4. Talks/articles by maintainers.
5. High-quality independent explanations.
6. Forum posts and Stack Overflow.
7. LLM explanations.

LLMs may help summarize or compare. They must not be the source of truth for standards, compiler APIs, security, or accessibility.

### 8.4 Learning note completion rule

A learning note is complete only when it says:

```txt
This is what EdgeZero will do now.
This is what EdgeZero will not do yet.
This fixture will prove it.
```

## 9. Resource map

This map is intentionally conservative. Start with these before chasing secondary material.

### 9.1 Rust implementation foundation

Use for compiler core, CLI internals, IR modeling, diagnostics, and performance-sensitive tooling.

Primary resources:

```txt
The Rust Programming Language
https://doc.rust-lang.org/book/

The Rust Reference
https://doc.rust-lang.org/reference/

The Cargo Book: Workspaces
https://doc.rust-lang.org/cargo/reference/workspaces.html

Rust Compiler Development Guide
https://rustc-dev-guide.rust-lang.org/
```

Required project output:

```txt
docs/learning/rust/ownership-error-handling-modules.md
docs/learning/rust/cargo-workspaces.md
ADR: Rust crate layout
```

Minimum knowledge before serious compiler work:

```txt
ownership and borrowing
lifetimes at a practical level
Result and error propagation
modules and visibility
traits and enums
workspace dependency management
testing and snapshot testing patterns
```

Do not study advanced async Rust, unsafe Rust, macro systems, or compiler internals beyond need unless a fixture requires them.

### 9.2 JavaScript, TypeScript, JSX, and TSX parsing

Use for authoring syntax, AST handling, source spans, and type-aware constraints.

Primary resources:

```txt
TypeScript Compiler API wiki
https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API

TypeScript Compiler API Book
https://typescriptcompilerapi.com/

Oxc documentation
https://oxc.rs/

Oxc parser usage
https://oxc.rs/docs/guide/usage/parser

SWC documentation
https://swc.rs/
```

Required project output:

```txt
docs/learning/compiler/tsx-ast-source-spans.md
ADR: Parser front-end choice
fixture: parse-simple-component
```

Minimum knowledge:

```txt
AST nodes
source spans
JSX vs TSX expression boundaries
type-only imports
class fields
methods and closures
decorators if used
```

### 9.3 Web platform and standards

Use for HTML-first rendering, Web Component output, form behavior, DOM patching, and progressive enhancement.

Primary resources:

```txt
WHATWG HTML Standard
https://html.spec.whatwg.org/

WHATWG Custom Elements section
https://html.spec.whatwg.org/multipage/custom-elements.html

MDN Web Components
https://developer.mozilla.org/en-US/docs/Web/API/Web_components

MDN templates and slots
https://developer.mozilla.org/en-US/docs/Web/API/Web_components/Using_templates_and_slots

MDN Server-sent events
https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events
```

Required project output:

```txt
docs/learning/web-platform/custom-elements-lifecycle.md
docs/learning/web-platform/forms-and-progressive-enhancement.md
docs/learning/web-platform/dom-patching.md
fixture: custom-element-basic
fixture: native-form-fallback
```

Minimum knowledge:

```txt
custom element lifecycle
attributes vs properties
shadow DOM boundaries
slots
HTML forms and FormData
DOM text node updates
attribute vs property updates
event propagation and delegation
```

### 9.4 Accessibility

Use for accessibility graph, diagnostics, and generated form semantics.

Primary resources:

```txt
WCAG 2.2
https://www.w3.org/TR/WCAG22/

Understanding WCAG 2.2
https://www.w3.org/WAI/WCAG22/Understanding/

ARIA Authoring Practices Guide
https://www.w3.org/WAI/ARIA/apg/

WCAG 2.2 Techniques
https://www.w3.org/WAI/WCAG22/Techniques/
```

Required project output:

```txt
docs/learning/accessibility/accessible-name.md
docs/learning/accessibility/form-labels-errors.md
docs/learning/accessibility/aria-diagnostics.md
fixture: a11y-button-name
fixture: a11y-input-label
fixture: a11y-invalid-aria
```

Minimum knowledge:

```txt
accessible name computation at a practical level
labels and form control association
ARIA roles, states, and properties
native semantic HTML before ARIA
keyboard interaction for custom controls
focus order basics
```

Rule: if you cannot explain why a diagnostic is valid according to a standard or WAI guidance, do not implement it as an error. Implement it as an experimental warning or do not implement it yet.

### 9.5 Fine-grained reactivity

Use for signal/state graph, binding updates, derived values, and effects.

Primary resources:

```txt
Solid fine-grained reactivity
https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity

Solid intro to reactivity
https://docs.solidjs.com/concepts/intro-to-reactivity

Svelte compiler docs
https://svelte.dev/docs/svelte/svelte-compiler

Svelte runes article
https://svelte.dev/blog/runes
```

Required project output:

```txt
docs/learning/reactivity/signal-graph.md
docs/learning/reactivity/binding-dependencies.md
fixture: state-to-text-binding
fixture: event-updates-binding
```

Minimum knowledge:

```txt
signals
subscribers/dependents
derived computations
effects vs render bindings
batched updates
disposal/lifecycle
```

Do not copy Solid’s public API by default. Study its mechanics, then design the class-field/template authoring layer around compiler visibility.

### 9.6 Resumability and lazy loading

Use for lazy event chunks, serialized continuation state, and HTML-first delivery.

Primary resources:

```txt
Qwik resumability concept docs
https://qwik.dev/docs/concepts/resumable/

Qwikloader docs
https://qwik.dev/docs/advanced/qwikloader/

MDN dynamic import
https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/import
```

Required project output:

```txt
docs/learning/resumability/event-serialization.md
docs/learning/resumability/closure-capture-rules.md
fixture: lazy-click-handler
fixture: resumable-counter
```

Minimum knowledge:

```txt
hydration vs resumability
event handler serialization
symbol resolution
lazy dynamic imports
closure capture constraints
serializable state
versioned manifests
```

Rule: no user-visible resumability marker should be added without an RFC explaining why the compiler cannot infer it.

### 9.7 Streaming and async rendering

Use for server-rendered async regions, resources, and progressive HTML.

Primary resources:

```txt
Marko HTML streaming explanation
https://markojs.com/docs/explanation/streaming

Marko core await tag reference
https://markojs.com/docs/reference/core-tag

WHATWG HTML server-sent events
https://html.spec.whatwg.org/multipage/server-sent-events.html

MDN Server-sent events
https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events
```

Required project output:

```txt
docs/learning/streaming/async-regions.md
docs/learning/streaming/resource-placeholders-errors.md
fixture: streamed-resource-region
```

Minimum knowledge:

```txt
HTML streaming
async placeholders
error boundaries
resource dependency ordering
flush timing
client patch or append behavior
```

### 9.8 Testing strategy

Use for compiler snapshots, runtime behavior, browser compatibility, and web-platform correctness.

Primary resources:

```txt
Playwright docs
https://playwright.dev/

Playwright browser docs
https://playwright.dev/docs/browsers

Vitest guide
https://vitest.dev/guide/

web-platform-tests docs
https://web-platform-tests.org/
```

Required project output:

```txt
docs/learning/testing/compiler-fixtures.md
docs/learning/testing/browser-e2e.md
ADR: fixture and snapshot testing strategy
```

Minimum knowledge:

```txt
unit tests
snapshot tests
browser e2e tests
cross-browser matrix
flakiness control
fixture organization
size regression testing
```

### 9.9 Rust-to-Node packaging and distribution

Use for CLI packaging, native compiler bindings, and possible browser/Wasm tooling.

Primary resources:

```txt
NAPI-RS docs
https://napi.rs/

wasm-bindgen guide
https://rustwasm.github.io/docs/wasm-bindgen/

MDN Rust to WebAssembly guide
https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm
```

Required project output:

```txt
docs/learning/packaging/rust-node-boundary.md
ADR: CLI/native binding distribution strategy
```

Minimum knowledge:

```txt
native npm package distribution
platform-specific binaries
Node-API basics
Wasm tradeoffs
install-time vs publish-time compilation
checksums and provenance
```

### 9.10 Monorepo and workspace management

Use for project structure, local commands, dependency boundaries, and CI.

Primary resources:

```txt
Cargo workspaces
https://doc.rust-lang.org/cargo/reference/workspaces.html

pnpm workspaces
https://pnpm.io/workspaces

pnpm workspace yaml
https://pnpm.io/pnpm-workspace_yaml
```

Required project output:

```txt
ADR: monorepo workspace layout
justfile or xtask command surface
CI path filter policy
```

Minimum knowledge:

```txt
workspace members
shared dependency versions
workspace commands
path-based CI
lockfile ownership
release lanes
```

## 10. Implementation order

Do not implement by subsystem. Implement by thin vertical slices.

### Slice 1: parse and explain a static component

```txt
Input: simple component with static JSX
Output: parsed component summary and explain skeleton
No runtime.
No reactivity.
No Web Component output.
```

Required docs:

```txt
learning note: TSX AST and spans
fixture README
ADR: parser front-end
```

### Slice 2: static HTML generation

```txt
Input: component render method with static DOM
Output: HTML string
No client JS.
```

Required docs:

```txt
learning note: JSX-to-HTML semantics
fixture README
```

### Slice 3: dynamic text binding

```txt
Input: state field used in text
Output: binding graph and generated text patch target
```

Required docs:

```txt
learning note: signal graph basics
RFC: state field semantics
fixture README
```

### Slice 4: click updates text

```txt
Input: button click mutates state
Output: lazy or eager minimal handler updates exact text node
```

Required docs:

```txt
learning note: event delegation
learning note: DOM text patching
fixture README
```

### Slice 5: lazy event chunk

```txt
Input: same counter
Output: handler is separate chunk loaded on click
```

Required docs:

```txt
learning note: dynamic import and symbol resolution
ADR: lazy chunk manifest format
fixture README
```

### Slice 6: native form fallback

```txt
Input: form action
Output: valid HTML form and server action placeholder
```

Required docs:

```txt
learning note: HTML forms and FormData
RFC: action semantics
fixture README
```

### Slice 7: accessibility diagnostic

```txt
Input: invalid button or unlabeled input
Output: compiler diagnostic with fix suggestion
```

Required docs:

```txt
learning note: accessible name or form labels
fixture README with bad/fixed examples
```

### Slice 8: custom element output

```txt
Input: component
Output: registered custom element package
```

Required docs:

```txt
learning note: custom element lifecycle
ADR: custom element output contract
fixture README
```

## 11. Stop rules

Stop and document before continuing when any of these happens:

1. You are about to add a second way to represent the same semantic concept.
2. A runtime workaround is compensating for compiler uncertainty.
3. A test requires excessive mocking to pass.
4. A generated artifact cannot be traced back to source.
5. A diagnostic is technically correct but not explainable to a user.
6. A fixture becomes large enough that it proves multiple unrelated behaviors.
7. You are researching for more than the time box without producing a learning note.
8. You are changing syntax to make implementation easier rather than to make semantics clearer.
9. You cannot explain whether a feature is framework behavior, compiler behavior, runtime behavior, or platform behavior.
10. You are building a feature because a competitor has it, not because the product thesis requires it.

## 12. Daily working loop

Use this loop during manual development.

```txt
1. Pick one work item.
2. Identify the exact fixture it will prove.
3. Read only the resources needed for that fixture.
4. Write or update the learning note.
5. Write the expected fixture output before implementation where possible.
6. Implement the smallest compiler/runtime change.
7. Run tests.
8. Update explain output or diagnostic snapshots.
9. Record limitations.
10. Commit.
```

End each session with a short dev log.

Location:

```txt
docs/dev-log/YYYY-MM-DD.md
```

Template:

```md
# Dev log: YYYY-MM-DD

## Worked on

## Changed

## Learned

## Tests run

## Blockers

## Next exact task
```

The next task must be exact enough that work can resume without redesigning.

## 13. Weekly review loop

Once per week, answer these questions:

```txt
What semantic slice became more real?
What fixture became stronger?
What documentation became obsolete?
What runtime code grew because compiler knowledge was missing?
What concept did I misunderstand?
What should I stop doing next week?
```

Update the risk register if any risk became more likely.

## 14. Quality bars

### 14.1 Compiler quality bar

A compiler feature is not complete until:

```txt
it has a fixture
it has expected graph output
it has expected generated artifacts
it has diagnostics for at least one invalid case
it has explain output
it preserves source spans
it has a learning/design note if the concept was new
```

### 14.2 Runtime quality bar

A runtime feature is not complete until:

```txt
it is size-measured
it has a browser test if user-visible
it rejects incompatible manifest shape
it does not require global hydration unless explicitly marked
it documents failure behavior
```

### 14.3 Documentation quality bar

A document is not complete until:

```txt
it states status
it contains a concrete example
it says what is out of scope
it links to related fixtures or decisions
it names unresolved questions
```

## 15. First 30 days

### Week 1: operating base

Outputs:

```txt
repo skeleton
Cargo workspace
pnpm workspace
templates for ADR/RFC/learning/fixture/dev-log
CI skeleton
empty CLI
counter fixture stub
```

Learning:

```txt
Cargo workspace basics
pnpm workspace basics
Rust module/test basics
project command surface
```

### Week 2: source ingestion

Outputs:

```txt
parse static component
extract component metadata
snapshot parsed summary
source span preservation
basic parse diagnostic
```

Learning:

```txt
TSX AST
Oxc or SWC parser API
source spans
compiler diagnostics structure
```

### Week 3: template graph and static HTML

Outputs:

```txt
static DOM graph
static HTML generation
fixture explain skeleton
invalid template diagnostic
```

Learning:

```txt
JSX-to-DOM semantics
HTML element/attribute rules
source map basics
```

### Week 4: first dynamic binding

Outputs:

```txt
state field representation
dynamic text binding
binding graph snapshot
very small patch runtime spike
browser test skeleton
```

Learning:

```txt
fine-grained reactivity basics
DOM text patching
event delegation basics
Playwright basics
```

Do not attempt forms, streaming, router, auth, package publishing, devtools, or Web Component distribution in the first 30 days unless the first four weeks are complete.

## 16. Personal discipline rules

1. Keep a `PARKING_LOT.md` for good ideas that are not current.
2. Do not redesign the whole system after every new article.
3. When stuck, reduce the fixture, not the ambition of the product.
4. When overwhelmed, write the graph model for one example by hand.
5. When tempted to add syntax, first ask what graph node it creates.
6. When tempted to add runtime behavior, first ask why the compiler did not know enough.
7. When tempted to copy a framework, first identify the underlying invariant.
8. When a concept is hard, make a toy experiment and delete it after extracting the lesson.
9. Never let documentation drift more than one semantic slice behind implementation.
10. The project advances only when source code, generated output, tests, and docs advance together.
