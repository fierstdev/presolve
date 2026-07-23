# Phase N N6-C1 resource endpoint resolution contract

N6-C1 retains the exact semantic-package endpoint selected by a non-executable
`@resource("localEndpoint")` source fact. It is a prerequisite for Resource
lowering, not Resource support.

The string designator names one local import binding. The compiler resolves
that binding only through the caller-supplied, integrity-checked semantic
package table. A successful resolution records the local name, package,
version, integrity, export, declared type signature, runtime module, resume
policy, and closed resource endpoint metadata. It never reads or infers the
package implementation.

Resolution outcomes are explicit and compiler-owned: missing designator,
unbound designator, non-package binding, non-resource package binding, or one
resolved resource endpoint. This makes a package mismatch inspectable without
turning an unresolved import into generic client code.

N6-C1 deliberately creates no ResourceDeclaration, ResourceActivation, IR,
artifact, runtime transport, cancellation protocol, snapshot, or framework API.
The existing `PSC1046` source rejection remains mandatory even when endpoint
resolution succeeds. The source form becomes available only after the full
N6-C design contract is represented through declaration identity, per-instance
activation, artifacts, cancellation, resume validation, deterministic endpoint
execution, diagnostics, and browser proof.

Verification is `scripts/verify-n6c1-resource-endpoint-resolution.sh`.
