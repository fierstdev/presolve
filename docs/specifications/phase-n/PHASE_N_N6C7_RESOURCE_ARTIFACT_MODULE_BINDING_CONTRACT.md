# Phase N N6-C7 resource artifact module binding contract

`build_runtime_resource_artifact_with_modules` is the executable-facing
Resource artifact builder. It requires an exact runtime-module-table entry for
every projected endpoint and writes that explicit location to the artifact.
An omitted, changed, or mismatched coordinate fails with a declaration-specific
build error; it cannot become an implicit package lookup.

The original internal artifact remains intentionally location-free for compiler
analysis and malformed-artifact testing. Only this host-bound variant may be
passed to the future generated Resource runtime.

This does not execute a Resource or remove `PSC1046`. Endpoint module loading,
cancellation, result decoding, resume, and browser proof remain required.

Verification is `scripts/verify-n6c7-resource-artifact-module-binding.sh`.
