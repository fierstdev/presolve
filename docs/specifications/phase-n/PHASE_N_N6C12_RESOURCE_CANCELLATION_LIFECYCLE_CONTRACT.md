# Phase N N6-C12 resource cancellation lifecycle contract

Every browser Resource activation owns one `AbortController`. The generated
runtime aborts each active controller at `pagehide`, and an aborted invocation
records `cancelled` rather than ready or failed. No application callback, DOM
lookup, or package-global cancellation authority participates in this path.

The policy is a lifecycle prerequisite only. It does not yet connect component
destruction, input invalidation, explicit retry, snapshot capture, or resume
restoration; Resource source remains rejected until those paths are lowered.

Verification is `scripts/verify-n6c12-resource-cancellation-lifecycle.sh`.
