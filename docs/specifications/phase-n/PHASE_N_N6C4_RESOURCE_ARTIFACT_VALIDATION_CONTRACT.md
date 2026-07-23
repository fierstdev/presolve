# Phase N N6-C4 resource artifact validation contract

N6-C4 adds the fail-closed validation boundary for `RuntimeResourceArtifact`
schema version 1. A runtime consumer must reject an artifact with an unknown
schema, duplicate declaration or activation identity, incomplete package
endpoint coordinate, lifecycle state/generation mismatch, or an activation
that references no declaration.

Validation operates solely over the artifact. It neither evaluates package
JavaScript nor attempts a best-effort recovery, placeholder endpoint, implicit
network request, or altered lifecycle state. This gives later browser and
resume consumers one exact malformed-artifact failure authority.

The validated artifact remains internal while Resource source is rejected by
`PSC1046`. Endpoint transport, cancellation delivery, result serialization,
resume restoration, and executable browser proof are still missing.

Verification is `scripts/verify-n6c4-resource-artifact-validation.sh`.
