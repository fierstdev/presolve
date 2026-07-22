---
schema: presolve.launch-content
version: 1
presolve_version: 0.1.0-alpha
route: /architecture
---

# Architecture

The compiler owns authored-source interpretation, canonical identities,
diagnostics, and products. Consumers use emitted contracts rather than
reconstructing compiler state. The [compiler platform contract](../../docs/compiler-platform-contract.md)
and [runtime contract](../../docs/runtime-contract.md) define those boundaries.

Production and resumability remain explicit compiler/runtime products under the
[production optimization](../../docs/production-optimization-contract.md) and
[resumability](../../docs/resumability-contract.md) contracts.
