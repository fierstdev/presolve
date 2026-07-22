# Proposed Phase M roadmap: deterministic project initialization

**Status:** Proposal for owner acceptance; not implementation authority.

## Decision and scope

Phase M proposes the smallest useful expansion of the frozen alpha: activate
only `presolve create` as an explicit, deterministic local initializer. It
creates a caller-selected directory containing a finite, versioned Presolve
starter artifact. It does not activate `dev`, `benchmark`, or `doctor`, source
discovery, package-manager execution, network access, registry access,
telemetry, hosting, deployment, publication, signing, upload, editor writes,
or any language/compiler/runtime semantic change.

This proposal is intentionally narrower than a generic scaffolder. The command
must never infer a destination, inspect a parent project, overwrite existing
content, install dependencies, or select a template from ambient state.

## Compatibility and rollback

Activation changes the frozen reserved-command disposition for `create` only,
so acceptance requires a targeted amendment to the platform freeze, alpha
support matrix, CLI help/exit fixtures, public documentation, and release note.
The starter artifact has a versioned manifest and exact committed bytes. A
failed or cancelled create operation leaves no destination behind; a released
create version can be withdrawn by reverting to the last committed
matrix-compatible revision and restoring exit-6 behavior through an amendment.

## Slice sequence

### M0 — acceptance amendment and boundary contract

Owner accepts this proposal and records the single-command amendment: exact
arguments, exit codes, destination ownership, overwrite refusal, cancellation,
atomicity, starter versioning, exclusions, compatibility, and rollback. No
filesystem implementation occurs in this slice.

### M1 — starter artifact contract and canonical fixture

Define one starter profile with an explicit finite file list, UTF-8 bytes,
permissions policy, path-normalization rules, manifest schema, and SHA-256
fixture. The profile contains only existing supported alpha configuration and
example-shaped source; it cannot encode package installation or a remote URL.

### M2 — destination validation and planning

Implement a pure request-to-plan validator. It accepts one explicit relative or
absolute destination under caller authority, rejects empty/root/escaping/
ambiguous paths and pre-existing targets, and emits a deterministic creation
plan. It performs no writes, reads no ambient project metadata, and retains no
state.

### M3 — transactional local writer

Implement the plan executor using a sibling temporary directory, exact file
bytes, restrictive creation behavior, and atomic final rename where supported.
Every error cleans only its own temporary path and preserves pre-existing
content. Fixtures cover interruption, conflict, permissions denial, and
repeated invocation.

### M4 — `presolve create` command adapter

Expose only the contracted explicit command. Help, text/JSON success and error
envelopes, exit codes, and reserved-command behavior are fixture-backed. The
adapter delegates solely to M2/M3; it does not compile, discover, install,
start a server, or inspect the generated project.

### M5 — generated-project conformance proof

From a fresh temporary destination, prove starter manifest hashes, exact file
list, clean repeated conflict behavior, and caller-invoked existing `check` or
`build` using explicit inputs. This is a conformance proof, not an install or
development-server workflow.

### M6 — documentation, support, and rehearsal amendment

Amend public CLI reference, support matrix, examples guide, distribution and
clean-room rehearsal documentation. Add executable snippets for `create` and
state the remaining `dev`, `benchmark`, and `doctor` exit-6 exclusions.

### M7 — Phase M freeze

Freeze the starter schema/hash, command fixtures, conformance matrix, public
support row, rollback procedure, and exact amendment lineage. The Phase M
final gate runs all M0–M6 verifiers plus inherited Phase L evidence and a
clean-tree audit.

## Evidence matrix

| Concern | Required proof |
| --- | --- |
| No semantic expansion | Phase A–L frozen contract checks and a source-surface audit |
| Canonical starter | versioned manifest, exact byte/hash fixture, path-policy fixture |
| Safe writes | conflict, interruption, permission, cleanup, and atomicity tests |
| CLI boundary | help/exit/text/JSON fixtures and reserved-command matrix |
| Generated project | explicit existing check/build conformance from fresh output |
| Public compatibility | amended support/docs/rehearsal snippets and rollback test |

## Acceptance checklist

Before M0 starts, the owner must explicitly accept this roadmap, select the
starter profile and public command grammar, approve the frozen-command
amendment, and confirm that local filesystem creation is in scope. No slice
may begin from this proposal alone.
