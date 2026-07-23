# Phase Q Q3 static request handoff

**Status:** compiler handoff complete; adapter adoption next.

`build_static_request_handoff_v1` maps each compiler-owned route to one GET
artifact path. It deliberately admits no authored request handlers, SSR,
streaming, loaders, server actions, sessions, or middleware. Hosts can serve
the resulting immutable artifacts without interpreting application source.
