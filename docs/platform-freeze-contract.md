# Presolve platform freeze contract

**Status:** L20 final platform-freeze candidate. This document becomes the
Phase L freeze evidence only when `verify-l20-platform-freeze.sh`, `just check`,
and a clean committed tree pass. It adds no language, compiler, runtime,
package, hosting, publication, signing, or registry behavior.

## Frozen public platform

| Surface | Frozen alpha support | Evidence |
| --- | --- | --- |
| Compiler and runtime | frozen Phase A--K products, diagnostics, generated artifacts, and browser/runtime contracts | [frozen contract map](frozen-contract-map.md), [production contract](production-optimization-contract.md) |
| Lifecycle | compiler service, incremental reuse, persistent cache, explicit workspace, and watch-once boundaries | [L3--L8 contract verifiers](../scripts/verify-l8-watch-contracts.sh) |
| CLI | accepted explicit L9 and product-only L11 command families, exact help/exit/JSON/human boundaries | [public CLI verifier](../scripts/verify-l13b-public-cli-docs.sh), [public surface verifier](../scripts/verify-l13d-public-surface-matrix.sh) |
| Products and schemas | all registry-approved v1 products under strict supplied-byte readers | [L10 verifier](../scripts/verify-l10-schema-contract.sh), [L11 product verifiers](../scripts/verify-l11f-tooling-products.sh) |
| Editor and packages | product-only compiler WASM, language-service, LSP, VSCode facade, testing utility, and runtime export | [L12 and L15 package verifiers](../scripts/verify-l12e2-vscode-facade.sh), [testing](../scripts/verify-l15b-testing-package.sh) |
| Examples and documentation | five explicit alpha examples, public references, link/snippet checks, and launch content | [example verifier](../scripts/verify-l14b-production-resume-example.sh), [docs verifier](../scripts/verify-l13a-public-docs-index.sh), [launch verifier](../scripts/verify-l18-launch-content.sh) |
| Repository and distribution | identity, community boundaries, offline package/release dry run, and clean-room alpha rehearsal | [identity](../scripts/verify-public-identity.sh), [release dry run](../scripts/verify-l17b-release-dry-run.sh), [rehearsal](../scripts/verify-l19b-clean-room-rehearsal.sh) |

The authoritative public API/support listing is the
[alpha support matrix](alpha-support-matrix.md). It fixes available command,
product, editor, and package scope plus compatibility, support, and rollback
policy without making any package publishable.

## Reserved-capability disposition

`create`, `dev`, `benchmark`, and `doctor` remain recognized exit-6 command
families. This freeze expressly excludes them from the Presolve alpha platform;
they are not deferred implementation promises, supported APIs, or publication
authority. Source discovery, scaffolding, server/telemetry/benchmark services,
hosting, deployment, registry publication, signing, upload, editor write
authority, document lifecycle, and source/compiler fallback are likewise
excluded. A future delivery requires a separately accepted amendment and its
own product, fixture, and verification authority.

## Final evidence matrix

The final freeze runner requires the authoritative L13--L21 sequence, public
identity, repository-layout, public surface, reproducibility-lane, launch,
alpha-support, clean-room, and distribution/release evidence to remain present
and internally consistent. `just check` then executes the complete inherited
matrix, including frozen fixture, lifecycle, browser, package, example, docs,
and audit lanes. Canonical fixtures remain committed bytes; host observations
and benchmark values never affect this gate.

No cleanup may rewrite archived evidence or frozen contract semantics. Phase L
is complete only after this contract and the final verification evidence are
committed, the worktree is clean, and the documented excluded capabilities
remain explicit.
