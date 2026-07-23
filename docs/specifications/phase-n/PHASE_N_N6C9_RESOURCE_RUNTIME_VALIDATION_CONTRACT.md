# Phase N N6-C9 resource runtime validation contract

The generated runtime now reads `presolve-resources-runtime` and fail-closes
boot on malformed Resource JSON, unsupported schema, duplicate declarations or
activations, missing exact endpoint location, or invalid activation lifecycle
linkage. Applications without a Resource artifact preserve existing boot
behavior.

Runtime validation does not import, invoke, or otherwise execute the endpoint.
It is a prerequisite for later activation and ensures malformed Resource
metadata cannot degrade into a generic client callback or package lookup.

Verification is `scripts/verify-n6c9-resource-runtime-validation.sh`.
