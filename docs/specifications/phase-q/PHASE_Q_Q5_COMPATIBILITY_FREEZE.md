# Phase Q Q5 metaframework compatibility freeze

**Status:** frozen.

Phase Q freezes the first Presolve metaframework as compiler-owned static
topology and handoff products:

* static `@route` identity, conflict diagnostics, hierarchy, and manifests;
* namespaced canonical route artifact inventories;
* static GET request-to-artifact ownership;
* provider-neutral deployable release digest inventory; and
* thin `@presolve/application` CLI invocation projectors.

No second router runtime, server runtime, artifact merger, source discovery,
or deployment adapter is part of this version. Dynamic route parameters,
layouts, SSR, loaders, server actions, sessions, middleware, secrets, and
provider-specific deployment remain separately versioned successor products.
