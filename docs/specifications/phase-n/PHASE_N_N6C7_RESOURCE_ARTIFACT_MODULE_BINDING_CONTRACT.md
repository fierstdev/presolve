# Phase N N6-C7 resource artifact module binding contract

`build_runtime_resource_artifact_with_modules` is the executable-facing
Resource artifact builder. It requires an exact runtime-module-table entry for
every projected endpoint and writes that explicit location to the artifact.
An omitted, changed, or mismatched coordinate fails with a declaration-specific
build error; it cannot become an implicit package lookup.

The original internal artifact remains intentionally location-free for compiler
analysis and malformed-artifact testing. Only this host-bound variant may be
passed to the generated Resource runtime.

N6-C13 selects this builder from the canonical CLI once source resolution and
an explicit runtime mapping both succeed. Endpoint module loading and
page-teardown cancellation are admitted there; result decoding, source reads,
resume, and input invalidation remain required future work.

Verification is `scripts/verify-n6c7-resource-artifact-module-binding.sh`.
