# PHASE_L_SLICES_L11_L20.md

Status: Authoritative Implementation Specification

Prerequisite:

- PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md
- PHASE_L_SLICES_L1_L10.md

This document governs the second half of Phase L.

By entering L11 the compiler platform is considered complete.

The remaining work transforms the platform into a polished public product while preserving every constitutional guarantee established through Phase K and the first half of Phase L.

---

# General Rules

Every slice shall:

- preserve compiler semantics
- preserve runtime semantics
- preserve optimization behavior
- preserve diagnostics
- preserve deterministic outputs
- preserve schema compatibility

Every slice ends with:

- complete verification
- clean repository
- AGENT_HANDOFF.md
- progress update
- commit

---

# L11 — Developer Tooling

## Purpose

Implement the canonical developer tooling suite.

Developer tooling shall consume compiler products exclusively.

---

## Required Commands

presolve explain

presolve inspect

presolve trace

presolve graph

presolve benchmark

presolve profile

presolve doctor

---

## Explain

Explain compiler decisions.

Examples:

Dependency selection

Optimization reasoning

Template lowering

Component generation

Manifest generation

Incremental invalidation

---

## Inspect

Inspect immutable compiler products.

Examples:

Semantic graph

Artifact graph

Workspace graph

Dependency graph

Generated manifests

Runtime artifacts

---

## Trace

Produce deterministic compiler traces.

Examples:

Compiler phases

Build scheduling

Incremental rebuilds

Optimization timeline

Artifact generation

---

## Graph

Visualize compiler products.

Project graph

Workspace graph

Dependency graph

Artifact graph

Compilation plan

---

## Benchmark

Measure compiler performance.

Cold build

Incremental build

Workspace build

Cache reuse

Memory usage

Compilation throughput

---

## Profile

Produce compiler performance reports.

Required metrics:

Time

Memory

Artifact count

Node count

Graph size

Optimization cost

---

## Doctor

Validate project health.

Required checks:

Workspace validity

Configuration validity

Compiler version

Schema compatibility

Cache integrity

Dependency consistency

---

## Verification

Every command documented.

Deterministic output.

Machine-readable mode.

Human-readable mode.

Commit.

---

# L12 — IDE & Language Service

## Purpose

Provide first-class IDE integration.

---

## Required Features

Hover

Go to Definition

Find References

Rename

Diagnostics

Document Symbols

Workspace Symbols

Semantic Tokens

Completion

Signature Help

Source Mapping

---

## Rules

IDE never reparses language independently.

IDE consumes compiler products.

Compiler remains authoritative.

---

## Language Service

Introduce:

@presolve/language-service

---

## VS Code Extension

Introduce:

@presolve/vscode

---

## Verification

Large workspace validation.

Incremental edits.

Cross-package navigation.

Diagnostics parity.

Commit.

---

# L13 — Documentation

## Purpose

Replace internal engineering documentation with public documentation.

---

## Required Guides

Getting Started

Installation

Quick Start

Project Structure

Language Guide

Components

Templates

State

Actions

Computed

Context

Slots

Configuration

CLI

Diagnostics

Optimization

Architecture

Compiler Pipeline

Examples

Contributing

FAQ

Roadmap

Release Notes

---

## Requirements

Every public API documented.

Every CLI command documented.

Every package documented.

Every compiler product documented.

Architecture diagrams included where appropriate.

---

## Verification

Broken link detection.

Example validation.

Command validation.

Documentation completeness audit.

Commit.

---

# L14 — Canonical Examples

## Purpose

Establish reference applications.

---

## Required Examples

Counter

Todo

Dashboard

Shopping Cart

Markdown Editor

Data Grid

Component Library

Performance Demonstration

Workspace Example

Testing Example

---

## Requirements

Idiomatic Presolve.

Production quality.

Fully documented.

Continuously tested.

---

## Verification

Every example builds.

Every example passes tests.

Every example included in CI.

Commit.

---

# L15 — Testing Infrastructure

## Purpose

Public testing architecture.

---

## Introduce

@presolve/testing

---

## Required Features

Fixture utilities

Snapshot validation

Workspace fixtures

Compiler fixtures

Golden artifact validation

Performance fixtures

Regression fixtures

---

## CI

Examples tested.

Compiler tested.

Workspace tested.

CLI tested.

Language service tested.

---

## Verification

Regression suite.

Performance suite.

Golden output validation.

Commit.

---

# L16 — Repository & Community Readiness

## Purpose

Prepare the repository for public collaboration.

---

## Required Files

README.md

LICENSE

CHANGELOG.md

CONTRIBUTING.md

SECURITY.md

CODE_OF_CONDUCT.md

Issue Templates

Bug Template

Feature Template

PR Template

Discussion Templates

---

## README

Required sections:

Overview

Why Presolve

Installation

Quick Start

Examples

Architecture

Documentation

Contributing

License

---

## Verification

Repository audit.

Community health check.

Markdown validation.

Commit.

---

# L17 — CI/CD & Release Engineering

## Purpose

Establish permanent release automation.

---

## CI

Build

Test

Lint

Formatting

Examples

Benchmarks

Documentation

Workspace Validation

Schema Validation

---

## Release

Package publishing

GitHub Releases

Versioning

Changelog generation

Artifact publishing

---

## Verification

Release dry run.

CI reproducibility.

Version validation.

Commit.

---

# L18 — Website Readiness

## Purpose

Prepare presolve.dev.

---

## Required Sections

Home

Documentation

Architecture

Examples

Benchmarks

Blog

Roadmap

GitHub

Playground (placeholder permitted)

---

## Documentation Export

Automatically generated API docs.

CLI docs.

Compiler product docs.

Examples.

---

## Verification

Navigation audit.

Link audit.

Content audit.

Commit.

---

# L19 — Alpha Release Preparation

## Purpose

Prepare Presolve 0.1 Alpha.

---

## Required Work

Semantic versioning.

Package versions.

Release notes.

Migration notes.

Installation validation.

Package publishing validation.

Repository validation.

Website validation.

---

## Deliverables

Alpha checklist.

Known limitations.

Future roadmap.

Support policy.

Contribution guidance.

---

## Verification

Clean installation.

Fresh project creation.

Example validation.

Package validation.

Commit.

---

# L20 — Platform Freeze

## Purpose

Freeze the Presolve platform.

This is the final architectural checkpoint before product releases.

---

## Required Verification Matrix

Compiler determinism

Runtime determinism

Incremental determinism

Workspace determinism

Cache determinism

Compiler service determinism

CLI determinism

Tool determinism

Schema compatibility

Documentation completeness

Example completeness

CI reproducibility

Release reproducibility

Package reproducibility

---

## Repository Audit

No temporary artifacts.

No obsolete code.

No experimental implementations.

No dead packages.

No undocumented public APIs.

---

## Final Products

Compiler Platform

Developer Tooling

CLI

Language Service

Documentation

Examples

Website Content

Repository

Release Automation

CI/CD

---

## Completion Criteria

Phase L completes only when:

✓ Presolve identity fully established

✓ Compiler platform complete

✓ Developer tooling complete

✓ Language service complete

✓ Documentation complete

✓ Examples complete

✓ Repository publicly publishable

✓ Website content launch ready

✓ CI/CD operational

✓ Alpha release prepared

✓ All verification matrices pass

✓ Repository clean

✓ Final Platform Freeze committed

---

# Phase L Exit State

Upon completion of L20:

The project permanently transitions from an internal compiler effort to the public Presolve platform.

Future work shall proceed exclusively through semantic versioning and product releases.

No additional architectural phases are expected.

Compiler evolution shall occur through:

0.x Releases

↓

1.0

↓

Subsequent semantic versions

Phase L therefore represents the final architectural milestone of the Presolve compiler platform.
