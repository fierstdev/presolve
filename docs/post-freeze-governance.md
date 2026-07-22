# Post-freeze governance

**Status:** L21 stewardship handoff. This is a non-feature governance record;
it authorizes no implementation, publication, deployment, signing, upload, or
release execution.

## Versioning and compatibility

Presolve remains a 0.x alpha. Public compatibility is the frozen support in
the [alpha support matrix](alpha-support-matrix.md) and the [platform freeze]
(platform-freeze-contract.md), not an inferred future promise. A compatibility
change requires an accepted amendment with its affected fixture bytes,
verifiers, documentation, and release note. Version labels never alter frozen
canonical bytes by themselves.

## Amendments

Only an explicit owner-approved amendment may change a frozen semantic,
identity, schema, diagnostic, durable byte, support row, or reserved-capability
disposition. The amendment must name its superseded authority, bounded surface,
fixtures, verifier, migration/rollback evidence, and effective version. The
archived evidence remains intact; no amendment silently rewrites history.

## Security and release authority

[SECURITY](../SECURITY.md) owns vulnerability intake. The distribution contract
and release dry run prove local private-package evidence only. Publishing,
hosting, signing, upload, registry credentials, and release execution require
separate authorized external release authority; this document grants none.

## Deprecation

A supported surface may be deprecated only through an owner-approved amendment
that updates the alpha support matrix, public documentation, compatibility and
rollback guidance, and exact fixture/verifier coverage. Deprecation must not
silently change accepted bytes, exit codes, schemas, or reserved exit-6
behavior.

## Next-roadmap intake

The next roadmap starts with an accepted owner proposal identifying scope,
authority, exclusions, compatibility impact, evidence, and rollback. No
implementation begins from an intake item. Reserved `create`, `dev`,
`benchmark`, and `doctor` remain unavailable unless a future accepted amendment
defines their separate product and verification authority.

## Freeze evidence

L21 relies on the L20 platform-freeze verifier, the recorded clean `just check`
matrix, and a clean committed tree. It does not replace those gates or expand
the frozen public platform.
