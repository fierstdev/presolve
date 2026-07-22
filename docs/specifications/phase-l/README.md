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
6. The [revised delivery roadmap](PHASE_L_REVISED_ROADMAP.md) supersedes the
   sequencing in the completion execution plan and the heading-level order in
   L11-L20. It retains every frozen contract and names the current L10-B gate.

The [L2 repository constitution amendment](PHASE_L_L2_REPOSITORY_CONSTITUTION_AMENDMENT.md)
supersedes only conflicting L2 wording in the L1-L10 slice specification. It
preserves current active paths and governs L2 repository classification,
archival, and hygiene work.

`./scripts/verify-phase-l-specifications.sh` verifies that this complete
authority set is present and indexed.

L3 — Compiler Platform Products is implementation-ready. Its frozen public
product and compatibility contract is [the compiler platform contract](../../compiler-platform-contract.md).
The implementation and exact schema fixtures are owned by `presolve-compiler`.

L4 — Compiler Service and Durable Sessions is implementation-ready under its
authoritative service contract. The public implementation contract is
[the compiler service contract](../../compiler-service-contract.md).

L9 is governed by the tracked [L9 recovery and implementation contract](PHASE_L_L9_RECOVERY_AND_IMPLEMENTATION_CONTRACT.md), which incorporates the authoritative L9-A.3 construction-based codec-proof correction without adding an L3 durable decoder. The full phase sequencing authority is the [Phase L completion execution plan](PHASE_L_COMPLETION_EXECUTION_PLAN.md).

L10 is governed by the authored [tooling-schema implementation contract](PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT.md). It freezes negotiation and registry behavior without changing frozen L3–L8 product bytes.

L11-A is governed by the [tooling capability and reader contract](PHASE_L_L11_TOOLING_CAPABILITY_CONTRACT.md). It maps the exact product-backed reader boundary before any developer-tool command can be activated.

L11-D is governed by the [trace and structural compile-cost contract](PHASE_L_L11_TRACE_AND_COST_CONTRACT.md). It defines source-free deterministic products while trace and cost schemas remain reserved.

L11-E is governed by the [production artifact-graph contract](PHASE_L_L11_ARTIFACT_GRAPH_CONTRACT.md). It freezes direct Phase K graph/artifact provenance while the artifact-graph schema remains reserved.

L11-F activates only the L11-D/E products through canonical encoders and strict supplied-byte readers. It transitions those three L10 schemas to available without activating a command or persistence path.

L12-A is governed by the [editor capability audit](PHASE_L_L12_EDITOR_CAPABILITY_AUDIT.md). It proves current products are insufficient for editor queries and blocks implementation pending an L12-B amendment.

L12-B is governed by the [query-snapshot constitutional amendment](PHASE_L_L12_QUERY_SNAPSHOT_AMENDMENT.md). It defines the source-free compiler product required before any language-service implementation.

L12-C activates only that transient compiler-produced product, its strict decoder, frozen source-free fixture, and L10 registry entry. It does not activate a language service, LSP, extension, source discovery, edits, or persistence.

The [L12-C language-service binding audit](PHASE_L_L12_LANGUAGE_SERVICE_BINDING_AUDIT.md) records the missing compiler-owned package binding. A binding contract must select the host authority before the roadmap's language-service API can begin.

L12-C-1 is governed by the [compiler-owned WASM language-service binding contract](PHASE_L_L12_WASM_BINDING_CONTRACT.md). It selects strict Rust decode-first WASM delivery and freezes the read-only request, response, error, cancellation, lifecycle, packaging, and fixture boundary before any binding implementation.

L12-C-2 activates only the crate-private Rust projection shared by the future WASM adapter. It strictly decodes one supplied product before interpreting a canonical request and emits only contract-defined records, errors, or unsupported results; it creates no external host surface.

L12-C-3 activates the compiler-owned `@presolve/compiler-wasm` build boundary. Its generated web artifact exposes only the Rust `query_snapshot_v1` projection, and its smoke test consumes the frozen product without any JavaScript product decoder or compiler path.

L12-C-3-B freezes the generated WASM response matrix by SHA-256 over exact canonical response bytes for every supported projection, its empty result, unsupported behavior, and every defined error category.

L12-C-4 activates the thin `@presolve/language-service` wrapper. It initializes only the compiler-owned WASM artifact, transfers caller-owned product bytes and canonical request envelopes, and returns the WASM response without a product decoder, compiler path, cache, or source API.

L12-D is governed by the [stateless LSP adapter contract](PHASE_L_L12_LSP_CONTRACT.md). It freezes framing, capability, error, cancellation, and fixture boundaries before any protocol code.

L12-D-2 activates the in-process `@presolve/lsp` dispatcher only. It maps the contracted JSON-RPC methods to language-service operations, preserves returned ordering/ranges/errors, and declines every other method without source or document state.

L12-D-3 freezes exact JSON-RPC response hashes for every mapping, unsupported behavior, invalid framing, and propagated query error.
