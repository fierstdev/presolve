# PHASE_L_VERIFICATION_AND_RELEASE.md

Status: Authoritative Verification & Release Specification

Prerequisites:

- PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md
- PHASE_L_SLICES_L1_L10.md
- PHASE_L_SLICES_L11_L20.md

This document defines the mandatory verification protocol, release criteria, repository publication requirements, and Codex execution rules governing the completion of Phase L.

This document is constitutional.

No Phase L implementation is considered complete unless every requirement contained herein has been satisfied.

---

# 1. Philosophy

Phase K froze the compiler.

Phase L freezes the platform.

Verification is therefore significantly broader than previous phases.

The objective is not merely correctness.

The objective is confidence that Presolve is suitable for public adoption.

Verification therefore includes:

- compiler correctness
- platform correctness
- tooling correctness
- documentation correctness
- repository correctness
- release correctness

---

# 2. Verification Principles

Every verification shall demonstrate:

Correctness

Determinism

Reproducibility

Completeness

Stability

No verification may rely upon manual interpretation.

Verification shall be executable.

---

# 3. Compiler Verification

The compiler shall pass:

All existing compiler tests.

All parser tests.

All binder tests.

All semantic tests.

All optimization tests.

All runtime generation tests.

All projection tests.

All diagnostics tests.

No regressions permitted.

---

# 4. Runtime Verification

Verify:

Runtime behavior.

Scheduling.

State updates.

Actions.

Computed.

Context.

Slots.

Component lifecycle.

Generated runtime artifacts.

Behavior shall remain identical to Phase K.

---

# 5. Incremental Verification

Verify:

Cold build

↓

Incremental build

↓

Repeated incremental build

↓

Workspace incremental build

↓

Large workspace incremental build

Every produced artifact shall match a clean compilation.

---

# 6. Cache Verification

Verify:

Fresh cache

Warm cache

Deleted cache

Corrupted cache

Version mismatch

Cache migration

Partial cache reuse

Workspace cache reuse

No stale artifacts.

No hidden state.

---

# 7. Workspace Verification

Verify:

Single package

Multiple packages

Shared packages

Dependency ordering

Circular dependency diagnostics

Workspace rebuilds

Workspace cache reuse

Workspace graph serialization

---

# 8. Compiler Service Verification

Verify:

Daemon startup

Daemon shutdown

Session persistence

Workspace reuse

Incremental scheduling

Concurrent requests

Long-running sessions

Memory stability

Compiler service shall never modify compiler semantics.

---

# 9. CLI Verification

Every command shall verify:

Help output

Exit codes

Error formatting

Machine-readable output

Human-readable output

Configuration loading

Workspace loading

Incremental behavior

Commands include:

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

presolve benchmark

presolve profile

presolve doctor

presolve cache

presolve workspace

presolve version

---

# 10. Tool Verification

Verify:

Explain

Inspect

Graph

Trace

Benchmark

Profile

Doctor

Artifact Explorer

Dependency visualization

Compiler visualization

Every tool shall consume immutable compiler products exclusively.

---

# 11. Language Service Verification

Verify:

Hover

Completion

Rename

Go to Definition

References

Diagnostics

Workspace symbols

Semantic tokens

Cross-package navigation

Incremental updates

Language service results shall exactly mirror compiler knowledge.

---

# 12. Documentation Verification

Every documentation page shall verify:

Exists

Builds

Links correctly

References valid commands

References valid packages

Contains current examples

Contains current screenshots where applicable

No broken links.

No stale commands.

No undocumented public APIs.

---

# 13. Example Verification

Every example shall verify:

Build

Run

Test

Documentation

Current API usage

Workspace compatibility

Examples become part of CI.

---

# 14. Benchmark Verification

Verify benchmark suite:

Cold build

Incremental build

Workspace build

Cache reuse

Memory usage

Compilation throughput

Results shall be reproducible.

---

# 15. Package Verification

Verify every package:

Build

Version

Exports

Dependencies

Documentation

License

Publication metadata

Packages:

@presolve/compiler

@presolve/runtime

@presolve/core

@presolve/cli

@presolve/create

@presolve/testing

@presolve/devtools

@presolve/language-service

@presolve/vscode

---

# 16. Repository Verification

Repository shall verify:

Directory layout

README

License

Contributing

Security

Changelog

Issue templates

PR templates

GitHub Actions

Release workflow

No obsolete artifacts.

No temporary scripts.

No dead code.

---

# 17. Website Verification

Verify:

Navigation

Documentation links

Examples

Architecture

Roadmap

GitHub

Benchmarks

Blog

Playground placeholder

Every public page shall build successfully.

---

# 18. CI/CD Verification

CI verifies:

Compiler

Runtime

CLI

Language Service

Examples

Documentation

Benchmarks

Workspace

Schema validation

Release packaging

CI shall succeed on a clean clone.

---

# 19. Release Verification

Perform complete dry-run release.

Verify:

Package generation

Versioning

Release notes

Artifacts

GitHub Release generation

Installation

Fresh project creation

Example execution

No manual intervention.

---

# 20. Public Release Checklist

Before repository publication:

✓ Presolve identity complete

✓ Repository renamed

✓ CLI renamed

✓ Packages renamed

✓ Documentation complete

✓ Examples complete

✓ Website ready

✓ README polished

✓ LICENSE finalized

✓ CONTRIBUTING complete

✓ SECURITY complete

✓ CHANGELOG initialized

✓ CI operational

✓ Release automation operational

✓ Repository clean

---

# 21. Repository Publication

Repository:

github.com/fierstdev/presolve

Visibility:

Public

Repository shall contain no internal-only engineering artifacts except archived historical documentation intentionally preserved.

---

# 22. Alpha Release Checklist

Release:

Presolve 0.1 Alpha

Verify:

Installation

CLI

Examples

Documentation

Website

Packages

Language Service

Developer tooling

Compiler platform

Release notes

Known limitations

Future roadmap

---

# 23. Codex Execution Protocol

Codex shall execute Phase L exactly as follows.

For every slice:

1. Read the authoritative roadmap.

2. Implement only that slice.

3. Verify completely.

4. Run required tests.

5. Produce AGENT_HANDOFF.md.

6. Produce progress update.

7. Commit.

8. Stop.

Never merge slices.

Never speculate beyond constitutional documents.

Never weaken deterministic guarantees.

Never change compiler semantics.

Never introduce language features.

Never reinterpret compiler products.

---

# 24. Blocking Rules

Execution shall stop immediately if:

A constitutional contract is missing.

Compiler semantics would change.

Runtime semantics would change.

Tooling requires independent source analysis.

Determinism cannot be preserved.

A required schema is undefined.

Repository state becomes ambiguous.

Codex shall report the blocker rather than invent architecture.

---

# 25. Platform Freeze

Phase L concludes with the Platform Freeze.

The following become frozen:

Compiler Platform

Compiler Services

Workspace Architecture

Package Architecture

CLI Contracts

Tooling Contracts

Language Service

Documentation Architecture

Repository Architecture

Release Architecture

Future work proceeds under semantic versioning.

No further architectural phases are expected.

---

# 26. Codex Master Prompt

You are continuing the Presolve compiler platform.

Phase K has frozen the compiler.

Phase L freezes the platform.

Your authoritative specifications are:

- PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md
- PHASE_L_SLICES_L1_L10.md
- PHASE_L_SLICES_L11_L20.md
- PHASE_L_VERIFICATION_AND_RELEASE.md

These documents are constitutional.

Do not invent architecture.

Do not modify compiler semantics.

Do not modify runtime semantics.

Do not modify diagnostics.

Do not modify optimization behavior.

Do not reinterpret compiler products.

Compiler products are the sole source of truth for every platform subsystem.

Implement one slice at a time.

Verify completely.

Commit.

Produce AGENT_HANDOFF.md and a progress update.

Stop immediately upon encountering an architectural blocker.

The project is transitioning from Presolve to Presolve.

Repository:

github.com/fierstdev/presolve

CLI:

presolve

Package Scope:

@presolve

The goal of Phase L is to deliver a complete, deterministic, publicly releasable compiler platform suitable for the Presolve 0.1 Alpha release.

---

# 27. Phase Completion

Phase L is complete only when:

✓ Every constitutional document is satisfied.

✓ Every implementation slice is complete.

✓ Every verification matrix passes.

✓ The repository is publicly publishable.

✓ Presolve 0.1 Alpha can be released without additional architectural work.

Only then may Phase L be declared complete.
