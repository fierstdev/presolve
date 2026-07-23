# Phase N N6-C3 resource artifact contract

N6-C3 serializes the internal Resource declaration and per-component-instance
activation products into `RuntimeResourceArtifact` schema version 1. Its
records retain canonical declaration/activation identities, typed data/error
boundaries, execution/retry/invalidation metadata, exact semantic-package
endpoint identity, cancellation policy, and lifecycle state/generation.

The artifact is built only from existing application semantic-model products.
It does not parse source, read package code, create a fetch wrapper, load a
runtime module, activate an endpoint, or change state. The builder fails closed
if a projected declaration lacks its resolved endpoint.

This remains a prerequisite artifact, not a published executable runtime
artifact. Resource source remains rejected by `PSC1046`; no CLI publication,
browser transport, cancellation delivery, snapshot codec, or resume restore is
available until later N6-C slices supply those products.

Verification is `scripts/verify-n6c3-resource-artifact.sh`.
