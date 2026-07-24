# Presolve metaframework

The Presolve metaframework is the conventional application layer over the
compiler: project discovery, file routes, build/check commands, and provider
deployment handoffs. It does not add a second router, renderer, parser, or
artifact pipeline.

The public workflow is documented in [`../docs/metaframework.md`](../docs/metaframework.md).
The `@presolve/application` package contains the low-level application handoff
types used by integrations; application authors normally use the `presolve`
CLI instead.
