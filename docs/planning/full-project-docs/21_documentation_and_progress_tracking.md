# Documentation and Progress Tracking System

Status: **operating protocol**  
Audience: solo founder/developer, future contributors  
Purpose: define exactly what to use to document, plan, track, and review EdgeZero development.

## 1. Recommendation

Use this stack:

```txt
Source of truth:
  GitHub repository

Long-lived documentation:
  Markdown files in the repository

Local thinking and learning:
  Obsidian over the same Markdown files or a repo-adjacent vault

Task tracking:
  GitHub Issues + GitHub Projects

Major planning:
  GitHub Milestones + project roadmap view

Architecture decisions:
  ADR Markdown files committed to the repository

Language/product proposals:
  RFC Markdown files committed to the repository

Daily/weekly progress:
  Weekly log Markdown files committed to the repository

Public docs later:
  Starlight, VitePress, or Docusaurus only after the alpha surface stabilizes

Optional later:
  Linear only if GitHub Projects becomes too awkward for multi-person planning
```

Do **not** start with Jira, Notion-as-source-of-truth, scattered personal notes, or a separate product-management system. Those tools can be useful for teams, but they will create unnecessary duplication while the project is still a compiler/product research effort.

The correct model is:

```txt
Issues track work.
Docs explain knowledge.
ADRs record decisions.
RFCs propose semantics.
Fixtures prove behavior.
PRs connect all of them.
```

## 2. Single source of truth policy

The repository is the source of truth.

Allowed source-of-truth artifacts:

```txt
README.md
/docs/**/*.md
/adr/**/*.md
/rfcs/**/*.md
/notes/**/*.md
/fixtures/**/README.md
.github/ISSUE_TEMPLATE/**
GitHub Issues
GitHub Projects
Git tags and releases
```

Not source of truth:

```txt
private chat history
loose desktop notes
Notion pages
Discord messages
whiteboards
browser bookmarks
AI conversations
uncommitted Obsidian notes
```

External tools may help thinking, but anything that controls product direction must be converted into repo text or a GitHub issue.

## 3. Documentation surfaces

Use four documentation layers.

### 3.1 Product docs

Location:

```txt
/docs/product/
```

Purpose:

- positioning
- product principles
- audience
- competitive notes
- non-goals
- release criteria

Examples:

```txt
/docs/product/positioning.md
/docs/product/non-goals.md
/docs/product/public-alpha-gate.md
/docs/product/competitor-map.md
```

Rule:

Product docs explain **why this should exist** and **what it promises**.

### 3.2 Architecture docs

Location:

```txt
/docs/architecture/
```

Purpose:

- compiler pipeline
- semantic graph model
- runtime model
- resumability model
- server/client split
- output targets

Examples:

```txt
/docs/architecture/compiler-pipeline.md
/docs/architecture/semantic-graph.md
/docs/architecture/runtime-manifest.md
/docs/architecture/resumability.md
/docs/architecture/server-client-split.md
```

Rule:

Architecture docs describe current intended design. If the design changes, update the doc in the same PR that changes the code.

### 3.3 Reference docs

Location:

```txt
/docs/reference/
```

Purpose:

- exact syntax
- compiler diagnostics
- CLI commands
- manifest schemas
- runtime APIs

Examples:

```txt
/docs/reference/component-class.md
/docs/reference/state.md
/docs/reference/resource.md
/docs/reference/action.md
/docs/reference/forms.md
/docs/reference/diagnostics.md
/docs/reference/cli.md
```

Rule:

Reference docs must be precise enough that a test can be derived from them.

### 3.4 Learning notes

Location:

```txt
/notes/learning/
```

Purpose:

- record concepts learned while building
- capture links to official references
- summarize implementation implications
- separate learning from decisions

Examples:

```txt
/notes/learning/2026-07-compiler-spans.md
/notes/learning/2026-07-custom-elements-lifecycle.md
/notes/learning/2026-08-source-maps.md
/notes/learning/2026-08-accessible-name-computation.md
```

Rule:

Learning notes are not specifications. A learning note can influence an ADR or RFC, but it does not decide anything by itself.

## 4. Progress tracking surfaces

Use three tracking levels.

### 4.1 Milestones

Use GitHub Milestones for large phases.

Initial milestones:

```txt
M0 - Sprint Zero
M1 - Static Template Compiler
M2 - Dynamic Binding Slice
M3 - Lazy Event Slice
M4 - Native Form Action Slice
M5 - Web Component Output Slice
M6 - Public Alpha Candidate
```

A milestone is only complete when:

```txt
all required issues are closed
all associated fixtures pass
all docs are updated
known limitations are documented
release notes exist, even if no package is published
```

### 4.2 Epics

Use parent GitHub Issues as epics.

Epic title format:

```txt
[Epic] <major capability>
```

Examples:

```txt
[Epic] Static DOM graph
[Epic] Dynamic text binding
[Epic] Lazy event chunks
[Epic] Compiler diagnostics
[Epic] Native form fallback
```

Each epic must contain:

```txt
Goal
Non-goals
Required semantic model
Affected packages/crates
Fixtures required
Documentation required
Exit criteria
Child issues
```

### 4.3 Work issues

Every piece of implementation work must have an issue.

Issue title format:

```txt
[Area] Verb object
```

Examples:

```txt
[Compiler] Parse TSX element spans
[Graph] Emit static DOM nodes
[Runtime] Patch dynamic text binding
[CLI] Add explain command skeleton
[Docs] Document state field semantics
[Test] Add counter static HTML fixture
```

No issue, no work. Small typo fixes may bypass this rule, but any semantic, architectural, runtime, or testable change needs an issue.

## 5. GitHub Projects setup

Create one GitHub Project named:

```txt
EdgeZero Development
```

Use these views:

```txt
Board: Current Work
Table: Full Backlog
Roadmap: Milestones
Table: Learning Queue
Table: Blocked Work
```

Use these fields:

```txt
Status:
  Inbox
  Ready
  In Progress
  Blocked
  Review
  Done
  Deferred

Area:
  compiler
  runtime
  cli
  language-tools
  docs
  tests
  examples
  devtools
  release
  research

Type:
  feature
  bug
  refactor
  diagnostic
  fixture
  docs
  spike
  learning
  decision
  chore

Phase:
  M0
  M1
  M2
  M3
  M4
  M5
  M6

Risk:
  low
  medium
  high
  unknown

Confidence:
  known
  needs-research
  speculative

Priority:
  P0
  P1
  P2
  P3
```

Status rules:

```txt
Inbox:
  Not triaged.

Ready:
  Has clear acceptance criteria and no known blocker.

In Progress:
  Actively being worked on now.

Blocked:
  Has a stated blocker and a linked issue/note/ADR.

Review:
  PR exists or written design is ready for review.

Done:
  Merged, tested, documented, and linked.

Deferred:
  Valid idea, not part of current phase.
```

## 6. Labels

Create labels with strict meanings.

```txt
area:compiler
area:runtime
area:cli
area:docs
area:tests
area:examples
area:language-tools
area:devtools
area:release
area:research

kind:feature
kind:bug
kind:refactor
kind:diagnostic
kind:fixture
kind:docs
kind:spike
kind:learning
kind:decision
kind:chore

phase:m0
phase:m1
phase:m2
phase:m3
phase:m4
phase:m5
phase:m6

risk:low
risk:medium
risk:high
risk:unknown

status:blocked
status:needs-adr
status:needs-rfc
status:needs-fixture
status:needs-docs
status:needs-research
```

Label discipline matters. If labels become vague, the tracker stops being useful.

## 7. Required templates

### 7.1 Feature slice issue template

```md
# Goal

What exact behavior should exist after this issue is complete?

# Non-goals

What is explicitly not included?

# Affected areas

- [ ] compiler
- [ ] runtime
- [ ] CLI
- [ ] docs
- [ ] tests
- [ ] examples

# Semantic model

What graph nodes, manifest fields, or runtime concepts are involved?

# Acceptance criteria

- [ ] source input exists
- [ ] generated output exists
- [ ] fixture test exists
- [ ] explain output exists if compiler-visible
- [ ] docs updated
- [ ] known limitations documented

# Resources

Official docs, specs, or prior project references.

# Links

ADR:
RFC:
Fixture:
PR:
```

### 7.2 Spike issue template

```md
# Question

What question must this spike answer?

# Time box

Maximum time allowed:

# Options to compare

1.
2.
3.

# Evidence required

- [ ] notes
- [ ] small prototype if useful
- [ ] recommendation
- [ ] follow-up issue or ADR if needed

# Explicit non-goal

This spike does not ship production behavior.
```

### 7.3 Learning issue template

```md
# Concept

What concept must be learned?

# Why it matters to EdgeZero

What implementation area depends on this?

# Primary resources

Use official docs/specs first.

# Output required

- [ ] learning note committed under /notes/learning
- [ ] implementation implications listed
- [ ] unanswered questions listed
- [ ] follow-up work linked
```

## 8. ADR rules

Use ADRs for decisions that affect repository structure, implementation strategy, dependencies, release policy, or engineering process.

Location:

```txt
/adr/
```

Filename format:

```txt
NNNN-short-title.md
```

Example:

```txt
0001-use-monorepo.md
0002-use-github-projects.md
0003-use-swc-for-initial-tsx-parsing.md
```

ADR template:

```md
# ADR NNNN: Title

Date: YYYY-MM-DD
Status: proposed | accepted | superseded | rejected

## Context

What situation forced this decision?

## Decision

What are we doing?

## Consequences

What becomes easier?
What becomes harder?
What risks remain?

## Alternatives considered

1.
2.
3.

## Review trigger

What future fact would cause this ADR to be revisited?
```

ADR rules:

```txt
One decision per ADR.
No ADR without alternatives.
No accepted ADR without consequences.
No silent reversals. Supersede the old ADR.
```

## 9. RFC rules

Use RFCs for user-facing semantics and compiler-visible language design.

Location:

```txt
/rfcs/
```

Filename format:

```txt
NNNN-feature-name.md
```

Example:

```txt
0001-class-components.md
0002-state-fields.md
0003-resource-primitive.md
0004-action-forms.md
```

RFC template:

```md
# RFC NNNN: Title

Status: draft | accepted | rejected | implemented

## Summary

What is being proposed?

## Motivation

Why does this exist?

## Syntax

What does the user write?

## Semantics

What does it mean?

## Compiler model

What graph nodes or manifest fields are produced?

## Runtime model

What runtime support is required?

## Diagnostics

What errors or warnings should exist?

## Explain output

What should `fw explain` show?

## Examples

Good examples and edge cases.

## Alternatives

What else was considered?

## Open questions

What remains unresolved?
```

RFC rules:

```txt
No user-facing syntax without an RFC.
No RFC without generated-output examples.
No RFC without diagnostics.
No RFC without explain-output implications.
```

## 10. Weekly progress logs

Use weekly logs, not daily logs. Daily logs create too much noise for a solo project.

Location:

```txt
/notes/progress/
```

Filename format:

```txt
YYYY-WW.md
```

Example:

```txt
2026-W28.md
```

Template:

```md
# Week YYYY-WW

## Focus

Main objective for the week.

## Completed

- 

## Learned

- 

## Decisions made

- ADR/RFC links only; do not bury decisions here.

## Blockers

- 

## Bugs or risks discovered

- 

## Next week

- 

## Metrics

Issues closed:
PRs merged:
Fixtures added:
Docs updated:
Known failing tests:
```

Rule:

Progress logs summarize work. They do not replace issues, ADRs, RFCs, or docs.

## 11. Pull request rules

Every non-trivial PR must link at least one issue.

PR template:

```md
# Summary

What changed?

# Linked issues

Closes #

# Change type

- [ ] compiler behavior
- [ ] runtime behavior
- [ ] docs only
- [ ] test only
- [ ] refactor
- [ ] spike

# Verification

- [ ] unit tests
- [ ] fixture tests
- [ ] browser/e2e tests if relevant
- [ ] docs updated
- [ ] explain output updated if relevant

# Screenshots/output

Paste generated output, CLI output, or failing/passing fixture summary.

# Risk

What could this break?
```

PR merge rule:

```txt
Do not merge behavior without tests.
Do not merge syntax without docs.
Do not merge compiler inference without explain output.
Do not merge architecture change without ADR/RFC link.
```

## 12. Documentation update rules

Use these rules during development:

```txt
If you learn something important, write a learning note.
If you decide something durable, write an ADR.
If you design user-facing semantics, write an RFC.
If you implement behavior, update reference docs.
If you add a fixture, add or update fixture README.
If you discover a limitation, document it immediately.
If a doc becomes wrong, fix it in the same PR as the code change.
```

Forbidden pattern:

```txt
I will document this later.
```

Replacement pattern:

```txt
Write the minimum accurate note now.
Improve it later if needed.
```

## 13. Fixture documentation rules

Every fixture directory must have a README.

Example:

```txt
fixtures/compiler/counter-static/README.md
```

Template:

```md
# Fixture: counter-static

## Purpose

What behavior does this fixture prove?

## Input

What source file is compiled?

## Expected output

What generated artifacts matter?

## Semantic graph expectations

What graph nodes must exist?

## Diagnostics

Expected warnings/errors, if any.

## Explain output

What should `fw explain` report?

## Related issues

- #
```

Rule:

A fixture without a README is not complete.

## 14. Learning queue

Track learning as first-class work.

Initial learning queue:

```txt
Rust ownership and error handling
Rust workspace and crate design
Parser architecture
TSX parsing strategies
AST spans and source maps
Intermediate representation design
Graph data structures
Compiler diagnostics
DOM tree construction
Custom Elements lifecycle
Shadow DOM and light DOM tradeoffs
Signals and fine-grained reactivity
Schedulers and microtasks
Event delegation
SSR and streaming HTML
Resumability models
Serialization constraints
Accessible name computation
ARIA validity rules
Form submission and progressive enhancement
HTTP caching and prefetching
CSP and Trusted Types
Package publishing and provenance
Browser e2e testing
```

For each learning issue, create a note with:

```txt
summary
terms
official resources
what EdgeZero needs
implementation implications
open questions
```

## 15. Resource system

Maintain one resource index:

```txt
/notes/resources/index.md
```

Structure:

```md
# Resource Index

## Rust

- Rust Book: https://doc.rust-lang.org/book/
- Rust Reference: https://doc.rust-lang.org/reference/
- Cargo Book: https://doc.rust-lang.org/cargo/

## TypeScript

- TypeScript Handbook: https://www.typescriptlang.org/docs/
- TypeScript Compiler API wiki: https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API

## Web Platform

- MDN Web Docs: https://developer.mozilla.org/
- WHATWG HTML: https://html.spec.whatwg.org/
- DOM Standard: https://dom.spec.whatwg.org/
- Custom Elements: https://html.spec.whatwg.org/multipage/custom-elements.html

## Accessibility

- WCAG 2.2: https://www.w3.org/TR/WCAG22/
- WAI-ARIA: https://www.w3.org/TR/wai-aria-1.2/
- Accessible Name and Description Computation: https://www.w3.org/TR/accname-1.2/

## Testing

- Playwright: https://playwright.dev/
- Web Platform Tests: https://web-platform-tests.org/
- Vitest: https://vitest.dev/

## Framework references

- React Compiler: https://react.dev/learn/react-compiler
- Svelte docs: https://svelte.dev/docs
- Solid docs: https://docs.solidjs.com/
- Qwik docs: https://qwik.dev/docs/
- Astro docs: https://docs.astro.build/
- Lit docs: https://lit.dev/docs/
- Marko docs: https://markojs.com/docs/
```

Resource rule:

```txt
Prefer official docs, specifications, source code, and design documents.
Use blog posts only as secondary interpretation.
Use social media as signal, not authority.
```

## 16. Obsidian usage

Use Obsidian as a Markdown editor and knowledge graph, not as a separate source of truth.

Recommended setup:

```txt
Open the repository root as an Obsidian vault
or
Open /notes as an Obsidian vault
```

Recommended folders:

```txt
/notes/learning/
/notes/progress/
/notes/resources/
/notes/spikes/
```

Recommended Obsidian usage:

```txt
link learning notes to ADRs and RFCs
link resources to implementation notes
use backlinks for concept discovery
keep all notes as plain Markdown
commit useful notes to Git
```

Avoid:

```txt
private-only notes that contain important decisions
Obsidian plugins that create non-portable state
using backlinks as a replacement for explicit docs
```

## 17. GitHub Projects usage

Use GitHub Projects for execution, not knowledge storage.

Good GitHub Project items:

```txt
implement dynamic text binding
write RFC for state fields
research source maps
add fixture for native form fallback
fix span diagnostic for invalid JSX
```

Bad GitHub Project items:

```txt
think about framework
learn compiler stuff
make it better
work on runtime
```

Every project item should be answerable:

```txt
What will exist when this is done?
How will it be verified?
Where will it be documented?
```

## 18. When to use Linear

Do not use Linear at the beginning.

Consider Linear later if:

```txt
you have multiple regular contributors
GitHub Projects becomes too slow for planning
roadmap management becomes distinct from engineering execution
you need stronger triage workflows
```

If Linear is adopted later:

```txt
GitHub remains source of truth for code, PRs, ADRs, RFCs, docs, and releases.
Linear may track product planning and prioritization.
Every Linear issue that affects code must link to GitHub issues or PRs.
```

## 19. Review cadence

Use this cadence:

```txt
Daily:
  choose one active issue
  write notes directly in the issue or learning note
  commit only coherent increments

End of work session:
  update issue status
  write what is blocked or next
  commit learning notes if useful

Weekly:
  update progress log
  review project board
  close stale speculative items
  promote learning into ADR/RFC/doc where needed

Monthly:
  review roadmap
  review risk register
  review docs for drift
  update public-alpha gate
```

## 20. Movement rules

You may move to a new area only when one of these is true:

```txt
current issue is done
current issue is blocked and the blocker is documented
current spike reached its time box and produced a recommendation
current work revealed a prerequisite issue that must happen first
```

You may not move because:

```txt
the current issue became tedious
a new framework idea seems more exciting
you want to avoid writing tests
you want to avoid documenting a decision
you want to skip learning the underlying concept
```

## 21. Minimal starting setup

Create these immediately in the real repository:

```txt
.github/
  ISSUE_TEMPLATE/
    feature-slice.yml
    spike.yml
    learning.yml
  pull_request_template.md

adr/
  0001-use-monorepo.md
  0002-use-github-projects-and-markdown-docs.md

rfcs/
  0001-class-component-authoring.md

notes/
  learning/
  progress/
  resources/
    index.md
  spikes/

docs/
  product/
  architecture/
  reference/
```

First GitHub milestones:

```txt
M0 - Sprint Zero
M1 - Static Template Compiler
M2 - Dynamic Binding Slice
M3 - Lazy Event Slice
M4 - Native Form Action Slice
M5 - Web Component Output Slice
M6 - Public Alpha Candidate
```

First GitHub issues:

```txt
[Docs] Create resource index
[ADR] Confirm documentation and progress tracking system
[Repo] Add issue and PR templates
[Compiler] Create parser spike
[Fixture] Create counter-static fixture skeleton
[Docs] Write fixture README template
[Learning] Learn AST spans and source maps
[Learning] Learn Custom Elements lifecycle
```

## 22. Final rule

The tracking system should make the project calmer, not heavier.

If a tool or process does not help answer one of these questions, remove it:

```txt
What am I working on now?
Why am I doing it?
What does done mean?
What did I decide?
What did I learn?
What proves this works?
What changed in the product model?
```
