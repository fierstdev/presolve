# Phase N N6-C resource source and activation design

N6-C is not complete until one source declaration selects exactly one validated
semantic-package resource endpoint and lowers it through all existing
ResourceDeclaration and ResourceActivation products.

The source form must supply a stable name, endpoint binding, serializable data
and error boundary, explicit execution boundary, input dependency list,
cancellation policy, invalidation policy, and resume policy. The compiler must
resolve the package endpoint through the caller-supplied integrity-checked
package table; it must never inspect package implementation code.

Completion evidence must include a canonical resource declaration identity,
one activation per component instance, artifact records for endpoint/activation
and cancellation, resume snapshot acceptance or rejection, malformed artifact
failure, source diagnostics, and a browser proof using a deterministic test
endpoint. Until every product exists, resource source syntax remains rejected.
