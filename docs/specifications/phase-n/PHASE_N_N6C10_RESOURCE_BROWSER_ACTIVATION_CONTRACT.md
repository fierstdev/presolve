# Phase N N6-C10 resource browser activation contract

The browser runtime activates only `Client` and `Shared` declarations from a
validated, host-bound Resource artifact. It dynamically imports the exact
artifact location, selects the contract-declared export, and calls it with an
`AbortSignal` plus a frozen empty input record. Each planned activation records
pending then ready, failed, or cancelled lifecycle state; endpoint failures are
compiler-runtime diagnostics. Results must be JSON-serializable.

`Server` declarations fail closed in the browser. There is no generic fetch,
implicit package lookup, arbitrary input evaluation, State mutation, render
dependency, cache, snapshot, or resume integration yet. N6-C13 supplies the
validated source-to-artifact publication path; the remaining capabilities need
their own semantic contracts.

Verification is `scripts/verify-n6c10-resource-browser-activation.sh`.
