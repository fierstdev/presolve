# Phase L L13--L21 continuation contracts

**Status:** Authoritative continuation contract, accepted by the Phase L owner
on 2026-07-21. It turns the remaining roadmap gates into implementation-ready
slice authority while preserving every frozen Phase A--K and accepted L1--L12
contract.

## Universal boundary and sequence

No slice may parse, bind, analyze, or diagnose authored source outside the
compiler; reinterpret a compiler product; alter frozen Phase K behavior; retain
source/document/product state unless explicitly stated; or claim hosting,
signing, registry publication, or external release authority. Every slice is
verified, recorded, and committed independently.

Before L18, the owner-directed
[`Presolve identity migration contract`](../../presolve-identity-migration-contract.md)
supersedes the former retained-identity exceptions. It is a prerequisite
representation migration and does not enter the L18 launch-content slice.

The required order is L15, L14, L13, L16, L17, L18, L19, L20, then L21.
Each later gate is blocked until its predecessor's verifier passes. `just
check` remains inherited evidence and every new verifier joins it.

## L15 -- public test foundation

**L15-A:** create `docs/testing-contract.md`, mapping existing Rust/CLI/browser/
L3--L12/Phase-K fixtures to one public test purpose, exact local command,
environment, canonical assertion, and non-gating observation lane.

**L15-B:** create `@presolve/testing` with only fixture lookup, canonical-byte
comparison, and declared-command wrappers. It must not expose a compiler,
decoder, parser, source reader, cache, browser driver, or timing gate.

**L15-C:** add deterministic, browser/runtime, documented-example, and
non-gating observation lane manifests with pinned inputs and local commands.

## L14 -- canonical alpha examples

**L14-A:** create `docs/examples-contract.md` defining only Counter;
Components/Context/Slots; Forms; explicit workspace; and production/resume.
Each cites frozen evidence, explicit source/configuration authority, supported
commands, and unsupported features.

**L14-B:** add these examples serially. Every one has explicit input membership,
one public build/check fixture, existing browser proof where applicable, and no
create/scaffold command or reserved CLI behavior.

## L13 -- tested public documentation

**L13-A:** establish public docs index, ownership/version policy, reference-vs-
guide labels, archive labels, and a verifier-ready snippet format.

**L13-B:** document only accepted L9/L11 commands and limitations; every command
snippet is executed by a verifier.

**L13-C:** summarize frozen State, Actions, Computed, Context, Components,
Slots, Forms, resumability, production/runtime, service/cache/workspace, and
L12 editor boundaries by linking existing contracts, never redefining them.

**L13-D:** generate or fixture-validate command help, exits, available schemas,
package exports, and reserved status from real registry/package sources.

## L16 -- community readiness

Add complete LICENSE, CHANGELOG policy, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT,
support boundaries, issue/PR forms, and public README corrections. A repository
audit checks credentials, archive/generated labels, links, and support claims;
it does not publish anything.

## L17 -- reproducible distribution

**L17-A:** create `docs/distribution-contract.md` listing only real artifacts,
exports, dependency direction, versioning, checksums/provenance, and offline
install/package-dry-run evidence. All other packages are explicitly private.

**L17-B:** add fail-closed CI and local release dry run that builds/tests/packs
and emits a manifest only. It cannot publish, sign, upload, or require secrets.

## L18 -- launch content

Create repo-owned, versioned, link-checked public site content: home, docs,
architecture, examples, benchmark methodology, roadmap, repository link, and a
clearly non-functional playground placeholder. Deployment remains external.

## L19 -- alpha rehearsal

**L19-A:** freeze `docs/alpha-support-matrix.md`, citing verifiers for every
available command/product/editor/package and marking all else unavailable or
reserved; include compatibility, terminology, support, and rollback policy.

**L19-B:** add a clean-room rehearsal using the documented manual starter path
unless a separately proven create command exists. It checks examples, accepted
cache/workspace/watch/product commands, package metadata, and existing Phase K
artifacts without publishing.

## L20 -- platform freeze

Create `docs/platform-freeze-contract.md` and final verifier over frozen A--K
fixtures, L3--L12 compatibility, CLI/lifecycle/runtime browser cases, examples,
docs, package/release dry runs, and repository audit. It emits a public
API/support table. Phase L completes only when this gate, `just check`, and a
clean tree pass and every reserved capability is explicitly disposed.

## L21 -- post-freeze stewardship handoff

L21 is the owner-requested final non-feature slice. It creates
`docs/post-freeze-governance.md` for semantic-versioning, amendments,
security/release authority, deprecation, and next-roadmap intake. It authorizes
no implementation and verifies the L20 freeze evidence plus clean tree.

## Contract verification

The continuation verifier checks this order, every L13--L21 gate, no-semantics
boundary, L20 completion condition, and L21 non-feature handoff. Per-slice
verifiers supplement it rather than replacing it.
