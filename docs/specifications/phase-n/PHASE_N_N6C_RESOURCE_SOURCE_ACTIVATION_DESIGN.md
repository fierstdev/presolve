# Phase N N6-C resource source and activation design

N6-C13 admits the first source-to-browser activation path: one source
declaration selects exactly one validated semantic-package resource endpoint,
lowers it through ResourceDeclaration and ResourceActivation products, embeds
the exact host-bound runtime artifact, and invokes its declared browser module.

The admitted source form supplies a stable field name, endpoint binding, and
serializable data/error boundary. The package contract supplies the execution
boundary, cancellation policy, and resume vocabulary. The compiler resolves
the package endpoint through the caller-supplied integrity-checked package
table; it must never inspect package implementation code.

Completion evidence for this slice includes a canonical resource declaration
identity, one activation per component instance, endpoint/activation and
cancellation artifact records, missing-runtime-map failure, malformed-artifact
failure, source diagnostics, and a browser proof using a deterministic test
endpoint. Resource inputs, invalidation, retry, result/error reads,
component-destruction cancellation, and snapshot/resume remain separate
admission work; they do not block this explicit activation-only source form.
