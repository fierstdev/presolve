# PRESOLVE_PACKAGE_AND_CLI_SPECIFICATION.md

Status: Authoritative Platform Specification

Phase: L

Prerequisites:

- PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md
- PHASE_L_SLICES_L1_L10.md
- PHASE_L_SLICES_L11_L20.md
- PHASE_L_VERIFICATION_AND_RELEASE.md

---

# Purpose

This document defines the permanent public package architecture,
CLI architecture,
configuration system,
workspace model,
tool boundaries,
and platform conventions for Presolve.

This document is constitutional.

No implementation may invent package responsibilities or CLI behavior beyond this specification.

---

# 1. Architectural Philosophy

Presolve is delivered as a cohesive compiler platform.

Every public package has a single responsibility.

Dependencies are strictly layered.

No package may bypass compiler products.

The compiler remains the single producer of platform knowledge.

---

# 2. Package Dependency Rules

Layer 1

Compiler Core

↓

Layer 2

Runtime

↓

Layer 3

Compiler Service

↓

Layer 4

CLI

↓

Layer 5

Developer Tooling

↓

Layer 6

Editor Integrations

↓

Layer 7

Example Applications

No reverse dependencies are permitted.

---

# 3. Public Package Architecture

The following packages constitute Presolve v0.x.

---

## @presolve/compiler

Purpose

Compiler implementation.

Responsibilities

Parser

Binder

Semantic Analysis

Optimization

Projection

Artifact Generation

Diagnostics

Schema Generation

Compiler Products

Public API

Compile()

Check()

Analyze()

No runtime behavior.

No editor logic.

No CLI logic.

---

## @presolve/runtime

Purpose

Generated application runtime.

Responsibilities

Component execution

State

Scheduling

Actions

Context

Slots

Lifecycle

No compiler logic.

No tooling.

---

## @presolve/core

Purpose

Shared platform contracts.

Responsibilities

Types

Identifiers

Schema contracts

Shared utilities

Platform constants

Compiler/runtime shared definitions.

---

## @presolve/compiler-service

Purpose

Persistent compiler host.

Responsibilities

Compiler daemon

Workspace management

Persistent sessions

Incremental scheduling

Cache coordination

Compiler product hosting

Never reparses independently.

---

## @presolve/cli

Purpose

Command-line interface.

Consumes compiler service.

Contains no compiler logic.

---

## @presolve/create

Purpose

Project scaffolding.

Responsibilities

Templates

Starter projects

Workspace creation

Configuration generation

No compiler behavior.

---

## @presolve/devtools

Purpose

Developer tooling.

Responsibilities

Explain

Inspect

Trace

Graph

Profile

Benchmark

Artifact explorer

Dependency explorer

Consumes compiler products only.

---

## @presolve/language-service

Purpose

IDE integration.

Responsibilities

Completion

Hover

Rename

Diagnostics

References

Semantic Tokens

Workspace awareness

Consumes compiler products only.

---

## @presolve/testing

Purpose

Testing utilities.

Responsibilities

Golden outputs

Fixtures

Workspace testing

Compiler assertions

Snapshot validation

---

## @presolve/vscode

Purpose

VS Code extension.

Depends exclusively on language-service.

Contains no compiler implementation.

---

# 4. Dependency Matrix

compiler

↓

runtime

↓

core

↓

compiler-service

↓

cli

↓

devtools

↓

language-service

↓

vscode

↓

examples

Compiler remains lowest-level authority.

---

# 5. CLI Philosophy

CLI is stable.

Human-friendly.

Machine-readable.

Deterministic.

Every command shall support:

Readable output

Structured JSON output where applicable

Stable exit codes

Consistent diagnostics

---

# 6. CLI Commands

---

## presolve create

Purpose

Create a new Presolve project.

Responsibilities

Generate project

Install templates

Initialize configuration

Workspace support

---

## presolve dev

Purpose

Development server.

Responsibilities

Compiler daemon

Watch mode

Incremental builds

Diagnostic streaming

---

## presolve build

Purpose

Production compilation.

Produces release artifacts.

---

## presolve watch

Purpose

Persistent incremental compilation.

---

## presolve check

Purpose

Compile without emitting production artifacts.

Diagnostics only.

---

## presolve clean

Purpose

Remove generated artifacts.

Clear cache.

---

## presolve explain

Purpose

Explain compiler reasoning.

Examples

Dependency selection

Optimization decisions

Template lowering

Manifest generation

---

## presolve inspect

Purpose

Inspect immutable compiler products.

---

## presolve graph

Purpose

Visualize platform graphs.

Project

Workspace

Dependency

Artifact

Compilation

---

## presolve trace

Purpose

Build timeline.

Compiler phases.

Incremental scheduling.

---

## presolve profile

Purpose

Compiler performance analysis.

---

## presolve benchmark

Purpose

Performance measurements.

---

## presolve doctor

Purpose

Validate project health.

Configuration

Workspace

Schemas

Cache

Compiler

Dependencies

---

## presolve cache

Purpose

Cache inspection.

Statistics.

Cleaning.

Verification.

---

## presolve workspace

Purpose

Workspace operations.

Package listing.

Dependency graph.

Workspace validation.

---

## presolve version

Purpose

Platform version information.

---

# 7. Exit Codes

0

Success

1

Compilation failure

2

Configuration error

3

Workspace error

4

Compiler internal error

5

Cache error

6

Tooling error

7

Unexpected platform error

Exit codes are permanent.

---

# 8. Configuration

Canonical file:

presolve.json

Future extensions:

presolve.workspace.json

Configuration is declarative.

No executable configuration.

---

# 9. Workspace Model

Workspace root

↓

Workspace manifest

↓

Package graph

↓

Projects

↓

Compiler products

Workspace behavior is deterministic.

---

# 10. Compiler Service Protocol

The compiler service exposes:

Workspace loading

Compilation

Incremental compilation

Diagnostics

Compiler products

Tracing

Profiling

No mutable compiler state.

---

# 11. Cache Architecture

Persistent cache stores:

Artifact Graph

Dependency Graph

Compiler Products

Workspace Graph

Compilation Plans

Cache shall never become authoritative.

Compiler remains authoritative.

---

# 12. Build Modes

Supported modes

Development

Production

Incremental

Workspace

Benchmark

Profile

Trace

Explain

Inspection

No additional modes without constitutional amendment.

---

# 13. Logging

Platform logging shall provide:

Human-readable mode.

Machine-readable mode.

Stable identifiers.

Timestamp support.

Deterministic ordering.

---

# 14. Diagnostics

CLI diagnostics use compiler diagnostics unchanged.

CLI may format.

CLI may group.

CLI may filter.

CLI may never reinterpret diagnostics.

---

# 15. JSON Output

Commands supporting structured output shall expose stable JSON schemas.

Examples

build

check

graph

inspect

benchmark

profile

doctor

Schemas are versioned.

---

# 16. Package Versioning

All public packages share identical versions.

Example

@presolve/compiler

0.1.0-alpha

↓

All packages

0.1.0-alpha

Independent versioning is prohibited.

---

# 17. Release Artifacts

Release includes:

Compiler

Runtime

CLI

Compiler Service

Language Service

VS Code Extension

Documentation

Examples

Benchmarks

Schemas

---

# 18. Repository Standards

Repository shall contain:

README

LICENSE

CHANGELOG

SECURITY

CONTRIBUTING

Examples

Benchmarks

Documentation

Roadmaps

Architecture

Historical Engineering Archive

---

# 19. Future Compatibility

New packages may be introduced.

Existing package responsibilities may not overlap.

Compiler authority shall remain singular.

Backward compatibility is preferred.

Breaking architectural changes require constitutional amendment.

---

# 20. Non-Goals

Presolve v0.x intentionally excludes:

Plugin API

Third-party compiler extensions

Runtime plugins

Alternative runtimes

Compiler forks

Alternative CLI implementations

Distributed compilation

Remote compiler services

Cloud build infrastructure

These may be considered after v1.0 through separate architectural proposals.

---

# 21. Success Criteria

This specification is complete when:

✓ Every package has a single authoritative responsibility.

✓ Package dependencies are unambiguous.

✓ CLI behavior is fully defined.

✓ Exit codes are standardized.

✓ Configuration is standardized.

✓ Workspace architecture is defined.

✓ Compiler service boundaries are fixed.

✓ Cache boundaries are fixed.

✓ JSON contracts are versioned.

✓ Codex can implement the entire public platform without inventing package architecture or CLI behavior.

This document is constitutional and shall govern all public-facing platform architecture for Presolve.
