# Historical pre-public alpha support matrix

> Historical engineering record. It describes the pre-public compiler-only
> release boundary and is retained for fixture provenance. For the public
> `0.1.0-alpha.1` surface, start with [Alpha status](alpha.md).

**Status:** L19-A frozen alpha support matrix. This is a repository-local
support declaration for `0.1.0-alpha`; it adds no command, product, package,
hosting, publication, or compatibility behavior. An entry is available only
when its cited verifier or fixture already proves it. Everything not listed as
available is unavailable or reserved.

## Terminology and compatibility

The product, executable, package scope, repository destination, and launch
content identity are **Presolve**, `presolve`, `@presolve`,
`github.com/fierstdev/presolve`, and `presolve.dev`. Active public material
does not accept a legacy product, executable, package, diagnostic, or runtime
marker spelling as a compatibility layer. The identity audit is
[`verify-public-identity.sh`](../scripts/verify-public-identity.sh).

This alpha supports only the exact schemas, command grammars, outputs, ranges,
orders, and package exports frozen by their cited contracts. It makes no
forward- or backward-compatibility promise beyond those records. An incompatible
change requires a contract amendment, a versioned product change, updated
fixtures, and the corresponding verifier; a roadmap heading cannot widen this
matrix.

## Available command families

| Surface | Alpha support boundary | Evidence |
| --- | --- | --- |
| `version`, `help` | deterministic public version/help output | [L13-B public CLI verifier](../scripts/verify-l13b-public-cli-docs.sh) |
| `build`, `check` | one explicit configuration and caller-supplied source membership | [L13-B public CLI verifier](../scripts/verify-l13b-public-cli-docs.sh) |
| `clean`, `cache inspect`, `cache verify`, `cache clean` | only the selected project's L6 cache entries | [L13-B public CLI verifier](../scripts/verify-l13b-public-cli-docs.sh) |
| `workspace` | one explicit single-project L7 projection | [L13-B public CLI verifier](../scripts/verify-l13b-public-cli-docs.sh) |
| `watch --once` | one caller-supplied complete replacement candidate; no daemon | [L13-B public CLI verifier](../scripts/verify-l13b-public-cli-docs.sh) |
| `explain` and `explain --inspect` | source summary and complete semantic inspection view, including selected entity and graph output | [explain integration fixture](../crates/presolve_cli/tests/explain.rs) |
| `inspect workspace-snapshot`, `inspect workspace-graph` | one named, already-valid L10 product; human or JSON only | [L11 command verifier](../scripts/verify-l11c-tooling-commands.sh) |
| `graph workspace`, `graph artifact` | one named, already-valid workspace or artifact graph; human, JSON, or DOT only | [L11 artifact-graph verifier](../scripts/verify-l11g-artifact-graph-command.sh) |
| `trace`, `profile` | one named, already-valid trace or compile-cost product; human or JSON only | [L11 trace and profile verifiers](../scripts/verify-l11g-trace-command.sh), [profile](../scripts/verify-l11g-profile-command.sh) |

`create`, `dev`, `benchmark`, and `doctor` are recognized **reserved** command
families and exit `6`; [the public-surface verifier](../scripts/verify-l13d-public-surface-matrix.sh)
checks that status. Source discovery, scaffolding, a server, telemetry, a
benchmark gate, deployment, and editor-write behavior are unavailable.

## Available compiler products

All current L10 registry entries are available at the cited v1 boundary:

`presolve.workspace-configuration`, `presolve.workspace-snapshot`,
`presolve.workspace-graph`, `presolve.compiler-service-protocol`,
`presolve.persistent-artifact-cache`, `presolve.cache-inspection-report.v1`,
`presolve.workspace-manifest`, `presolve.watch-session-configuration`,
`presolve.watch-change-batch`, `presolve.watch-execution-plan`,
`presolve.watch-event`, `presolve.watch-session-snapshot`,
`presolve.watch-execution-report`, `presolve.build-trace`,
`presolve.compile-cost-report`, `presolve.artifact-graph`, and
`presolve.query-snapshot`.

The [L10 schema verifier](../scripts/verify-l10-schema-contract.sh) and
[public-surface verifier](../scripts/verify-l13d-public-surface-matrix.sh)
check the registry, negotiation, and documented inventory. No caller may
decode, recreate, persist, or reinterpret a compiler product outside the
separately frozen reader boundary.

## Available editor and package surfaces

| Surface | Alpha support boundary | Evidence |
| --- | --- | --- |
| `@presolve/compiler-wasm` | compiler-owned WASM delivery of the supplied query snapshot projection | [L12-C-3 verifier](../scripts/verify-l12c3-wasm-binding.sh) |
| `@presolve/language-service` | stateless supplied-product definition, references, flat symbols, diagnostics, and position queries | [L12-C-4 verifier](../scripts/verify-l12c4-language-service.sh) |
| `@presolve/lsp` | in-process JSON-RPC mapping of those exact projections | [L12-D verifier](../scripts/verify-l12d2-lsp-adapter.sh) |
| `@presolve/vscode` | product-free facade over LSP for those exact projections | [L12-E verifier](../scripts/verify-l12e2-vscode-facade.sh) |
| `@presolve/testing` | canonical-byte equality and immutable declared-test metadata | [L15-B verifier](../scripts/verify-l15b-testing-package.sh) |
| `@presolve/runtime` | committed runtime package export used by frozen runtime artifacts | [public-surface verifier](../scripts/verify-l13d-public-surface-matrix.sh) |

These packages are available only inside the verified repository/package-smoke
boundary. Every manifest is private: there is no registry publication,
install-from-registry promise, signing, upload, or release artifact. The
[distribution contract](distribution-contract.md) records the dependency
direction and offline evidence. Hover, rename, completion, signature help,
semantic tokens, source/URI mapping, edits, code actions, workspace symbols,
document synchronization, transport, persistence, and any compiler/source
fallback remain unavailable.

## Support and rollback policy

Support is limited to the repository's documented contribution, security, and
support channels; it provides no service-level agreement, hosted operation, or
compatibility support outside this matrix. See [contributing](../CONTRIBUTING.md),
[security reporting](../SECURITY.md), and [support boundaries](../SUPPORT.md).

No L19 artifact is published. If a verified alpha surface regresses, maintainers
may revert to the last committed matrix-compatible revision and rerun the cited
verifier before accepting a replacement. A rollback cannot silently mutate a
frozen product, reinterpret durable bytes, restore a retired spelling, or claim
registry/hosting authority; any such change requires its own approved contract
and evidence.
