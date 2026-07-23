# Phase O O6 deployment and public-distribution intake

**Status:** complete as an intake decision; no deployment product is admitted.

Phase P's immutable release directory and atomic publication pointer are local
publication semantics only. They do not define package distribution, hosting,
environment variables, secrets, deployment targets, CDN behavior, asset
ownership, observability, or rollback policy.

## Decision

O6 does not add a provider adapter, deploy command, package publishing,
environment-variable interface, secret transport, generated configuration, or
hosting runtime. `@presolve/application` remains an invocation projector and
does not gain deployment authority.

A successor deployment contract must separately freeze:

* a deployable artifact inventory and content-addressing policy;
* target capability declarations and adapter compatibility;
* public versus secret configuration boundary;
* immutable release promotion, rollback, and retention behavior;
* asset URL, caching, integrity, and observability products; and
* deployment diagnostics and audit identity.

Until then, the Phase P publication pointer is the complete supported output
handoff. It is deliberately not a deployment API.
