# Phase Q Q4 deployable release handoff

**Status:** compiler handoff complete; adapter adoption next.

`build_deployable_release_manifest_v1` derives a provider-neutral release ID,
route-manifest digest, and exact artifact digest/size inventory from compiler
bytes. Deployment adapters may project this manifest but cannot rewrite it or
infer secrets, environment variables, hosting policy, or source membership.
