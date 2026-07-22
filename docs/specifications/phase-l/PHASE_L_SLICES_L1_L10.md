# PHASE_L_SLICES_L1_L10.md

Status: Authoritative Implementation Specification

Prerequisite:
PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md

---

# General Rules

Every slice shall:

- preserve Phase K semantics
- preserve deterministic compiler outputs
- preserve runtime behavior
- preserve diagnostics
- preserve schemas unless explicitly authorized
- compile cleanly
- pass all existing tests
- add verification for new platform contracts
- end with a clean repository
- produce AGENT_HANDOFF.md
- produce progress update
- commit before beginning the next slice

No slice may proceed if an authoritative contract is missing.

---

# L1 — Presolve Identity Transition

## Purpose

Transition the project from Presolve to Presolve.

This slice changes identity only.

No compiler semantics change.

---

## Required Work

Rename public identity:

Presolve

↓

Presolve

Update:

Repository metadata

CLI identity

Documentation

Package names

Namespaces

Build metadata

Version metadata

Legal notices

Copyright

License headers where applicable

README

Contribution documents

Examples

Website references

Brand assets

---

## CLI

Canonical executable:

presolve

No alternate executable becomes canonical.

Future aliases remain optional.

---

## Package Scope

Canonical package scope:

@presolve

Example:

@presolve/compiler

@presolve/runtime

@presolve/core

@presolve/cli

---

## Verification

Verify:

No remaining public Presolve references

Compiler builds

Tests pass

Generated artifacts unchanged

CLI launches successfully

---

## Completion

The compiler publicly identifies exclusively as Presolve.

Commit.

---

# L2 — Repository Constitution

## Purpose

Prepare the repository for public development.

---

## Required Work

Establish canonical directory layout.

Required directories:

/compiler

/runtime

/packages

/cli

/docs

/examples

/tests

/tools

/benchmarks

/scripts

Remove:

temporary scripts

migration helpers

experimental artifacts

obsolete fixtures

unused build products

Archive:

historical engineering material

phase notes

internal migration documents

---

## Documentation

Create permanent repository standards.

Directory conventions.

Naming conventions.

Contribution conventions.

---

## Verification

Repository builds.

Tests pass.

CI paths remain valid.

No orphaned references.

Commit.

---

# L3 — Compiler Platform Products

## Purpose

Introduce immutable platform products.

---

## Required Products

ProjectGraph

WorkspaceGraph

DependencyGraph

ArtifactGraph

CompilerSession

PersistentCompilerState

IncrementalCompilationPlan

BuildSchedule

BuildTrace

CompileCostReport

ToolingVerificationManifest

WorkspaceManifest

---

## Requirements

Products are immutable.

Versioned.

Serializable.

Deterministic.

Future tooling consumes these products.

Compiler remains sole producer.

---

## Verification

Serialization tests.

Schema validation.

Determinism tests.

Round-trip verification.

Commit.

---

# L4 — Persistent Compiler Service

## Purpose

Introduce the compiler service.

---

## Required Responsibilities

Persistent sessions.

Project reuse.

Workspace reuse.

Incremental scheduling.

Shared caches.

Compiler coordination.

---

## Service Rules

Never change compiler semantics.

Never bypass compiler.

Never modify compiler products.

Never independently analyze source.

---

## Products

CompilerServiceState

CompilerSession

PersistentCompilerState

---

## Verification

Session restart.

Session persistence.

Deterministic outputs.

Memory stability.

Commit.

---

# L5 — Incremental Compilation

## Purpose

Introduce deterministic incremental compilation.

---

## Required Work

Dependency invalidation.

Incremental planning.

Incremental scheduling.

Incremental verification.

Artifact reuse.

Minimal recompilation.

---

## Requirements

Outputs identical to clean builds.

No stale artifacts.

No hidden state.

No nondeterminism.

---

## Verification

Cold build comparison.

Incremental comparison.

Repeated rebuild comparison.

Random edit verification.

Commit.

---

# L6 — Persistent Cache

## Purpose

Introduce persistent compiler cache.

---

## Cache Responsibilities

Artifact cache.

Dependency cache.

Graph cache.

Workspace cache.

Build metadata.

---

## Cache Rules

Versioned.

Deterministic.

Self-validating.

Safe invalidation.

Portable.

---

## Required Schema

Cache Schema v1.

---

## Verification

Cache restore.

Cache invalidation.

Version mismatch.

Corruption recovery.

Commit.

---

# L7 — Workspace Architecture

## Purpose

Support multi-package Presolve workspaces.

---

## Required Features

Workspace manifests.

Package graph.

Dependency ordering.

Shared cache.

Shared compiler service.

Cross-package diagnostics.

Incremental scheduling.

---

## Required Product

WorkspaceGraph

WorkspaceManifest

---

## Verification

Multiple package fixture.

Dependency ordering.

Workspace rebuild.

Workspace cache.

Commit.

---

# L8 — Watch Mode

## Purpose

Persistent development workflow.

---

## Required Features

File watching.

Incremental rebuild.

Change batching.

Cancellation.

Restart.

Debounce.

---

## Requirements

Uses compiler service.

Uses incremental planner.

Uses persistent cache.

Never changes compiler semantics.

---

## Verification

Rapid edits.

Large edits.

Delete.

Rename.

Workspace edits.

Commit.

---

# L9 — CLI Platform

## Purpose

Define the permanent Presolve CLI.

---

## Required Commands

presolve create

presolve dev

presolve build

presolve watch

presolve check

presolve clean

presolve explain

presolve inspect

presolve graph

presolve trace

presolve profile

presolve benchmark

presolve cache

presolve doctor

presolve workspace

presolve version

---

## CLI Rules

All commands consume compiler products.

No command performs independent analysis.

Errors use canonical diagnostics.

Consistent formatting.

Deterministic output.

---

## Verification

Help output.

Exit codes.

Error handling.

Command consistency.

Documentation generation.

Commit.

---

# L10 — Tooling Schemas & Platform APIs

## Purpose

Freeze the platform interfaces used by all tooling.

---

## Required Schemas

Workspace Schema v1

Compiler Service Protocol v1

Cache Schema v1

Build Trace Schema v1

Compile Cost Report Schema v1

Artifact Graph Schema v1

---

## Required Guarantees

Backward compatibility.

Version negotiation.

Schema validation.

Forward compatibility planning.

Documentation.

---

## Verification

Schema validation.

Round-trip serialization.

Compatibility matrix.

Version mismatch behavior.

Commit.

---

# Phase L Midpoint Gate

L10 concludes the compiler platform implementation.

Before L11 may begin, verify:

✓ Presolve identity complete

✓ Repository stabilized

✓ Compiler service operational

✓ Persistent sessions operational

✓ Incremental compilation deterministic

✓ Cache deterministic

✓ Workspace deterministic

✓ CLI stabilized

✓ Platform schemas frozen

✓ All tests pass

✓ Clean repository

Only after successful verification may Phase L continue into tooling, developer experience, public documentation, repository publication, website readiness, and release engineering.
