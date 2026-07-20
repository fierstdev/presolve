# Phase L specifications

These documents govern Phase L of the Presolve platform. They are active
authoritative specifications and are not part of the engineering archive.

Authority is applied in this order:

1. Frozen Phase A-K language, compiler, runtime, artifact, schema,
   optimization, and diagnostic contracts.
2. [Phase L platform constitution](PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md).
3. [Presolve package and CLI specification](PRESOLVE_PACKAGE_AND_CLI_SPECIFICATION.md).
4. The active slice specification: [L1-L10](PHASE_L_SLICES_L1_L10.md) or
   [L11-L20](PHASE_L_SLICES_L11_L20.md).
5. [Phase L verification and release requirements](PHASE_L_VERIFICATION_AND_RELEASE.md).

The [L2 repository constitution amendment](PHASE_L_L2_REPOSITORY_CONSTITUTION_AMENDMENT.md)
supersedes only conflicting L2 wording in the L1-L10 slice specification. It
preserves current active paths and governs L2 repository classification,
archival, and hygiene work.

`./scripts/verify-phase-l-specifications.sh` verifies that this complete
authority set is present and indexed.

L3 — Compiler Platform Products is implementation-ready. Its frozen public
product and compatibility contract is [the compiler platform contract](../../compiler-platform-contract.md).
The implementation and exact schema fixtures are owned by `presolve-compiler`.
