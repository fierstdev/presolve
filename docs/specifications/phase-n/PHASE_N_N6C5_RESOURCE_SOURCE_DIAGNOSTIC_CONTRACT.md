# Phase N N6-C5 resource source diagnostic contract

N6-C5 makes the third-party Resource boundary explicit in compiler diagnostics.
Every retained Resource source fact receives `PSC1128` with field provenance:

- missing or unbound designator identifies the required imported package endpoint;
- a local/non-package binding identifies that a semantic-package resource export
  is required;
- a non-resource package kind identifies the actual declared kind; and
- a successfully resolved endpoint identifies its exact package, version, and
  export, then states that execution lowering is still unavailable.

`PSC1128` supplements, rather than weakens, the existing `PSC1046` source
rejection. A developer can therefore distinguish a package-contract problem
from a compiler capability boundary without any package implementation
inspection or runtime fallback.

Verification is `scripts/verify-n6c5-resource-source-diagnostics.sh`.
