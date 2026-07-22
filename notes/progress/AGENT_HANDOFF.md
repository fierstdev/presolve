Presolve Agent Handoff

Repository state

* Branch: main
* Latest completed slice: L17-B - Fail-Closed Release Dry Run
* Working tree: clean after the temporary-pack checksum-manifest dry run; all editor-package boundaries remain product-free.
* Date: 2026-07-21

Last completed slice

* Slice: L17-B - Fail-Closed Release Dry Run
* Result: adds local and CI offline install/check/temporary-pack evidence that emits only a checksum manifest to stdout and deletes temporary tarballs. It has no publish, signing, upload, network, or secret authority and fails on a dirty tree through inherited checks.
* Next: L18 may add repository-owned launch content only.

* Slice: L17-A - Distribution Contract
* Result: inventories only existing private package manifests, exports, dependency direction, provenance, and offline package-smoke evidence. It authorizes no publish, signing, upload, or release artifact.
* Next: L17-B may add a fail-closed local/CI dry run only.

* Slice: L16 - Community Readiness
* Result: adds repository-only license, changelog, contribution, security, conduct, support-boundary, and issue/PR material. The audit checks files, labels, links, credentials, and root ownership; it makes no hosting, publication, private-support, or SLA claim.
* Next: L17-A may define reproducible distribution facts from existing package manifests only.

* Slice: L13-D - Public Surface Matrix
* Result: validates public help, reserved exit-6 status, every available L10 schema, and every package export directly from the CLI, registry, and manifests. It introduces no new public surface.
* Next: L16 may add repository community readiness material only.

* Slice: L13-C - Frozen Contract Map
* Result: maps State, Actions, Computed, Context, Components, Slots, Forms, resumability, production/runtime, service/cache/workspace, and L12 editor boundaries to their existing authority documents only. It defines no new semantics.
* Next: L13-D may validate help, exits, available schemas, package exports, and reserved status from real sources.

* Slice: L13-B - Executed Public CLI Reference
* Result: documents only accepted L9 explicit-project and L11 strict named-product commands. Every marked command uses an explicit fixture and is executed; the L11 tests construct their required valid products before invoking each reader. Reserved commands remain exit-6 only.
* Next: L13-C may summarize frozen contracts by linking them without redefining semantics.

* Slice: L13-A - Public Documentation Index
* Result: adds the public reference/guide/archive index, compiler-ownership and frozen-version policy, and a marker grammar for executable command snippets. It introduces no command or product behavior.
* Next: L13-B may document only accepted L9/L11 commands and execute every marked command snippet.

* Slice: L14-B-5 - Production/Resume Canonical Example
* Result: adds the exact Phase K computed-diamond source and uses the established production build only. The proof asserts the emitted production/resume artifacts and invokes the existing CSP/malformed-boot browser probe; it adds no deployment, benchmark, or author-time edit behavior.
* Next: L13-A may establish the public documentation index and snippet format only.

* Slice: L14-B-4 - Explicit Workspace Canonical Example
* Result: adds one exact source mapping from the existing L9-F CLI fixture and invokes only `presolve workspace`. The proof asserts the existing workspace-result schema/status and does not discover packages, manifests, dependencies, or sources.
* Next: L14-B-5 may add the production/resume example only from existing Phase K artifact and browser evidence.

* Slice: L14-B-3 - Forms Canonical Example
* Result: adds the frozen declaration-only serialized Form, Field, compiler-owned submit action, and explicit host shape. Its public L9 check verifier cleans its generated local cache and makes no browser-submit or network claim.
* Next: L14-B-4 may add an explicit workspace example only from existing L7/L9-F fixture evidence.

* Slice: L14-B-1 - Counter Canonical Example
* Result: Counter now declares canonical `presolve.json` and is checked through the accepted explicit L9 command path with one caller-supplied `.tsx` mapping. No discovery, build output, scaffold, or unsupported behavior is added.
* Next: L14-B-2 may add Components/Context/Slots only after selecting exact existing fixture sources and explicit command proof.

* Slice: L14-A - Alpha Example Corpus Contract
* Result: defines the exact five-example alpha corpus and binds each to frozen evidence, explicit input authority, a public proof, and exclusions. The existing counter is not reclassified as proven until L14-B supplies its public fixture.
* Next: L14-B may add canonical examples serially, beginning with Counter only.

* Slice: L15-C - Reproducibility Lane Manifest
* Result: declares deterministic-contract, browser/runtime, package-smoke, deferred-example, and non-gating observation lanes with their existing local commands and artifacts. Host measurements are expressly non-correctness evidence.
* Next: L14-A canonical alpha example contract only.

* Slice: L15-B - Public Testing Utility
* Result: adds `@presolve/testing` with only canonical-byte equality and immutable declared-test metadata. It is deliberately unable to read fixtures, execute commands, access the compiler, start a browser, or establish performance criteria.
* Next: L15-C reproducibility lane manifest and local reproduction commands only.

* Slice: L15-A - Public Test Inventory
* Result: maps existing compiler, tooling, CLI, runtime/browser, editor-package, and repository fixtures to public purposes, exact local reproduction, and canonical assertions without adding test semantics.
* Next: L15-B may add only the thin `@presolve/testing` utility package.

* Slice: L13-L21 - Remaining Phase L Continuation Contracts
* Result: the Phase L owner accepted the authoritative ordered contracts for L15, L14, L13, L16, L17, L18, L19, L20, and L21. They define exact public-surface/release boundaries, evidence, and no-semantics restrictions while preserving all L1--L12 and Phase K contracts.
* Next: L15-A public test inventory only. It must precede every L14 example, L13 docs, distribution, release, or freeze slice.

* Slice: L12-E-2 - Pinned VSCode Facade
* Result: adds `@presolve/vscode` as a product-free facade depending exclusively on `@presolve/lsp`. Its pinned extension-shaped fixture proves definition and unsupported behavior through the completed LSP/language-service/WASM chain.
* Boundary: no VSCode API, document/workspace model, source/path/URI access, compiler/language-service direct import, product decoder, cache, persistence, transport, or edit capability was added.
* Next: Phase L completion requires the final cross-package/frozen-contract/release audit before declaring the phase complete.

* Slice: L12-E-1 - VSCode Extension Contract
* Result: freezes `@presolve/vscode` as a product-free client that depends exclusively on the completed LSP package. It may render only established LSP results and must not acquire source, compiler, URI/path, cache, persistence, document-model, transport, or edit authority.
* Next: L12-E-2 may implement only the pinned-fixture-backed extension facade and its dependency-surface audit.

* Slice: L12-D-3 - LSP Fixture Matrix
* Result: freezes exact JSON-RPC response hashes for all supported mappings, unsupported hover, invalid framing, and an underlying unknown-identity error. The fixture proof exercises the generated WASM through language-service and LSP without any alternative decoder or compiler path.
* Next: L12-E may now author the `@presolve/vscode` extension boundary contract before implementation.

* Slice: L12-D-2 - Stateless LSP Dispatcher
* Result: adds `@presolve/lsp` as an in-process JSON-RPC dispatcher over the language service. It supports exactly the contracted definition, references, flat symbols, diagnostics, and position mappings, preserves language-service errors, and returns a stable unsupported result for every unadvertised method.
* Boundary: no network/server transport, document lifecycle, URI/path translation, text input, product decoding, compiler invocation, cache, persistence, or extension dependency exists. The dispatcher accepts product bytes for each call and retains no state.
* Next: L12-D-3 must add the canonical protocol fixture matrix/capability proof before L12-E can start.

* Slice: L12-D-1 - LSP Adapter Contract
* Result: freezes a stateless, product-only LSP translation boundary over the completed language service. It maps only existing definition, references, flat symbols, diagnostics, and position projection; preserves range/order/errors; and declares stable unsupported behavior and caller-owned cancellation.
* Next: L12-D-2 may implement only that fixture-backed adapter. L12-E remains unstarted.

* Slice: L12-C-4 - Thin Language-Service Wrapper
* Result: introduces `@presolve/language-service` as a direct wrapper over `@presolve/compiler-wasm`. It accepts caller-owned product bytes per query, serializes the documented request envelope, and returns the generated WASM response. It retains no product/session state and does not implement product validation, semantic analysis, source access, compiler invocation, caching, or persistence.
* Verification: the package's workspace dependency is linked locally, its smoke proof initializes the compiled artifact and preserves both unsupported and position responses, and the audit rejects decoder/host imports from the wrapper source.
* Next: L12-D must first author the LSP framing/capability/error/cancellation contract and fixture plan. No LSP or VSCode code is authorized yet.

* Slice: L12-C-3-B - WASM Response Fixture Matrix
* Result: freezes SHA-256 commitments over full canonical WASM response bytes for position, definition, references, flat symbols, diagnostics, an empty position result, unsupported hover, invalid request/product, and every defined query error. The Node smoke test constructs only contract-defined caller inputs and compares bytes emitted by generated WASM to that matrix.
* Boundary: the fixture harness reads product IDs only to issue contract-defined source-unit/opaque-ID requests; it does not decode, validate, or project the product. It has no cache, persistence, source/document, compiler, LSP, or editor behavior.
* Next: L12-C-4 may begin the thin `@presolve/language-service` wrapper, which must retain this WASM byte authority and add no product decoder or analysis.

* Slice: L12-C-3-A - Compiler-owned WASM Binding Surface
* Result: enables the compiler crate's `cdylib` WASM target and exports exactly one `wasm-bindgen` function, `query_snapshot_v1`, which delegates directly to the crate-private strict-decode-first Rust projector. `@presolve/compiler-wasm` is a generated-artifact package with no handwritten semantic/product code.
* Proof: the build is pinned to the installed `wasm-bindgen` 0.2.108 generator, compiles `wasm32-unknown-unknown`, and a Node smoke test invokes the generated artifact using the frozen query product. The binding verifier rejects decoder/host imports in the package surface and inherits the Rust/product contract chain.
* Boundary: JavaScript owns only generated loading and caller-supplied byte transfer. It cannot decode or produce the product, invoke the compiler, read sources, retain state, update documents, persist/cache data, or introduce LSP/VSCode behavior.
* Next: L12-C-3-B must add the contract-required frozen canonical request/response/error fixture matrix and complete the binding-surface audit. Only after that proof may L12-C-4 begin a thin `@presolve/language-service` wrapper. L12-D/L12-E remain unstarted.

* Slice: L12-C-2 - Rust Query Projection Core
* Result: adds one crate-private `query_snapshot_v1` projector which first invokes `decode_tooling_query_snapshot_v1`, then returns only existing records/references/diagnostics in canonical response envelopes. It intentionally selects all position-containing records in product order, keeps document symbols flat, and never exposes a native public language-service API.
* Behavior: position, definition, references, document symbols, and diagnostics are contract-exact. Invalid products are rejected before requests; noncanonical/unknown requests, unknown source units/opaque IDs, and out-of-range offsets fail closed. All remaining L12 capabilities return the stable unsupported result without source access or analysis.
* Boundary: this core is an implementation detail for the selected compiler-owned WASM ABI. It owns no host, compiler invocation, product/session/cache state, path/URI/source input, persistence, updates, LSP, or VSCode behavior.
* Next: L12-C-3 may add only the WASM adapter and `@presolve/compiler-wasm` package/build boundary, together with frozen canonical request/response/error fixtures and a proof that JavaScript cannot bypass this Rust core.

* Slice: L12-C-1 - Compiler-owned WASM Language-Service Binding Contract
* Result: selects provisional `@presolve/compiler-wasm` as the sole browser/JavaScript delivery authority. Its single synchronous `query_snapshot_v1(product_bytes, request_bytes)` operation must invoke Rust `decode_tooling_query_snapshot_v1` first, then project only existing product facts. A future `@presolve/language-service` can only be a thin wrapper over that binding.
* Contract: freezes canonical request/response envelopes for position, definition, references, flat document symbols, and document diagnostics; it deliberately returns every matching position record rather than inventing a best-record heuristic. Unsupported capabilities return stable results; invalid product/request, unknown source unit, out-of-range offset, and unknown opaque identity fail closed with stable errors.
* Boundary: no JavaScript decoder, compiler invocation, source/path/URI/text input, filesystem/network/clock access, semantic cache, persistence, update API, LSP, or VSCode code is authorized. Byte ownership is caller-retained and every query is synchronous/stateless.
* Next: L12-C-2 may implement only the Rust query projection and compiler-owned WASM ABI/package required by this contract, with its frozen request/response/error fixtures and binding-surface audit. It may not introduce the language-service wrapper before that ABI proof completes.

* Slice: L12-C-0 - Language-Service Binding Prerequisite Audit
* Result: the roadmap's `@presolve/language-service` target has no compiler-owned WASM ABI, native addon, IPC protocol, or JavaScript binding for either transient `CommittedCompilation.query_snapshot` delivery or Rust strict decoding. The existing runtime package supplies none of those authorities.
* Boundary: a JavaScript package cannot proceed without a prohibited duplicate decoder, alternate compiler invocation, or invented transport/persistence path. LSP and VSCode remain unstarted.
* Required decision: select one explicit delivery authority—compiler-owned WASM ABI, compiler-owned native addon, or an amendment replacing the package target with a Rust-native API—then author its request/response, error/cancellation, lifetime, packaging, and fixture contract.

* Slice: L12-C - Query-Snapshot Product Gate
* Summary: activates `presolve.query-snapshot` v1 as a transient `CommittedCompilation` result produced only inside a successful explicit L3 compiler invocation. It serializes source-unit revisions/lengths, source-provenanced semantic ranges, resolved references with exported endpoints, and compiler diagnostics.
* Privacy and identity: raw `SemanticId` values and source paths never leave the compiler. Public `QuerySemanticId` values are domain-separated SHA-256 projections; provenance-free internal entities and references without two exported endpoints are omitted rather than given fallback locations.
* Validation: canonical JSON has a final newline, product identity is self-excluding SHA-256, strict decode checks canonical bytes/order/ranges/reference targets, and the frozen fixture proves no source path or authored component text. Reversed source enumeration produces byte-identical output.
* Registry and lifecycle: `presolve.query-snapshot` is now L10-available with its strict decoder, but remains caller-retained transient data. It is absent from L4--L8 persistence, cache keys/payloads, CLI readers/commands, and all language-service/editor surfaces.
* Next: L12-D may add only a language-service API that projects this validated supplied product; it may not reparse/analyze source, synthesize updates, add persistence, activate LSP, or create an editor extension.

* Slice: L12-B - Compiler Query-Snapshot Constitutional Amendment
* Summary: defines `presolve.query-snapshot` as one transient, compiler-produced, source-free product bound to a validated L3 workspace snapshot; it exposes only source-free `QuerySemanticId` values derived inside the compiler, existing kinds, provenance ranges, references, and diagnostics.
* Boundary: the client owns URI/path translation and supplies compiler-issued `SourceUnitId` plus UTF-8 byte offsets. The product fails closed for unknown, stale, ambiguous, or out-of-range queries and cannot persist in L4--L8, the CLI, language service, or an editor extension.
* Projection boundary: only semantic entities with compiler-established provenance and references whose endpoints are both exported may appear. Provenance-free internal/synthesized facts receive no fallback range or public identity.
* Capability boundary: future L12-C may support only position lookup, definition, references, document symbols, and compiler diagnostics. Hover, rename, completion, signature help, semantic tokens, source mapping, edits, and code actions remain unsupported until their facts are explicitly established.
* Next: L12-C must atomically add the producer, strict decoder, source-free fixtures, identity/provenance/reverse-order proof, and L10 registry amendment before any language-service API begins.

* Slice: L12-A - Editor Capability Audit
* Summary: proves L3--L11 products lack public immutable semantic range/position, query identity, edit authority, and invalidation facts required for every advertised editor capability.
* Boundary: L12-B authorizes only the minimal immutable compiler-produced query snapshot contract before any language-service implementation may begin.

* Slice: L11-G-C - Artifact-Graph Command Projection
* Summary: activates `presolve graph artifact` as an explicit validated artifact-graph projection in canonical JSON, deterministic human text, or deterministic DOT.
* Boundary: it reads one supplied product file only and never rebuilds graph topology from generated modules or a build directory.
* Verification: focused Phase K-derived CLI projection, strict CLI clippy, inherited L11-G-B/L11-F verification, and `just check` pass.

* Slice: L11-G-B - Structural-Profile Command Projection
* Summary: activates `presolve profile --schema presolve.compile-cost-report --product <file> [--format human|json]` as a projection of one explicitly named, strictly decoded L11-F compile-cost product.
* Boundary: profile renders only canonical structural counts/bytes. It never measures elapsed time, CPU, memory, or host telemetry, and it never invokes compilation, discovers a project, scans output, or persists profile state.
* Verification: focused CLI success evidence, strict CLI clippy, the L11-G-A/L11-F verifiers, and `just check` pass. Next L11-G command projections remain separate slices.

* Slice: L11-G-A - Build-Trace Command Projection
* Summary: activates `presolve trace --schema presolve.build-trace --product <file> [--format human|json]` as a projection of one explicitly named, strictly decoded L11-F build-trace product.
* Boundary: the command reads exactly one caller-supplied file and renders canonical JSON or deterministic human text. It never invokes compilation, discovers a project, scans generated output, or persists trace state; malformed or schema-mismatched bytes retain tooling exit code 6.
* Verification: focused CLI success/rejection evidence, strict CLI clippy, the L11-F producer/reader verifier, and `just check` pass. Next L11-G command projections remain separate slices.

* Slice: L11-F - Tooling Product Producers, Registry, and Readers
* Summary: activates canonical `presolve.build-trace`, `presolve.compile-cost-report`, and `presolve.artifact-graph` v1 encoders and strict caller-supplied-byte decoders, then transitions only those L10 schemas from `reserved` to `available`.
* Provenance: trace construction accepts only ordered source-free established facts; cost construction binds the existing same-build Phase K report pair; artifact-graph construction consumes a validated in-memory K7 graph and same-build validated K8 artifact without scanning a build directory or generated files.
* Validation: canonical JSON has a final newline, self-excluding SHA-256 product identities are recomputed on decode, schema/version/canonical-order validation is strict, and malformed/noncanonical product bytes map to the reserved L11 tooling-error ranges.
* Verification: focused Phase K-derived product round trips, strict reader tests, L10 negotiation tests, strict compiler clippy, the L10 compatibility audit, and `just check` pass. No CLI product command starts until L11-G.

* Slice: L11-E - Production Artifact-Graph Contract
* Summary: defines the future immutable, source-free `presolve.artifact-graph` v1 product without activating a producer, decoder, command, registry entry, or persistence path.
* Provenance: the product can be created only from one successful invocation's validated Phase K `ProductionChunkGraph` and its same-build `ProductionRuntimeArtifactV1`. It is forbidden to scan, parse, glob, hash, or reconstruct graph facts from generated files or a build directory.
* Contract: chunks, dependencies, and activations retain only frozen K7/K8 topology facts with canonical ordering; graph identity is self-excluding SHA-256 and the artifact checksum is the validated existing artifact checksum. No source, paths, module contents, timing, host, cache, report, or benchmark facts are included.
* Boundary: the schema stays L10 `reserved` until L11-F atomically implements it alongside L11-D trace/cost products, complete strict readers, fixtures, determinism/provenance proofs, and registry transitions.
* Verification: `./scripts/verify-l11e-artifact-graph-contract.sh`, `./scripts/verify-phase-l-specifications.sh`, and `git diff --check` pass. The verifier runs inherited L11-D/L11-C/L11-B/L11-A/L10/L3-L9 audits and is included in `just check`.
* L11-D correction: `RuntimeCostReportV1` has no policy field. The cost contract therefore binds the pair by same build ID and direct same-invocation provenance, with the frozen production policy carried solely by `OptimizationReportV1`; no Phase K bytes or product fields were changed.

* Slice: L11-D - Trace and Structural Compile-Cost Contract
* Summary: defines future immutable, source-free `presolve.build-trace` and `presolve.compile-cost-report` v1 products without activating a producer, decoder, command, registry entry, or persistence path.
* Contract: traces contain only established L3-L8/L4 publication facts in a fixed stage order; compile-cost reports project the existing paired Phase K optimization and runtime-cost reports. Neither product can contain source, paths, filenames, timestamps, durations, host measurements, or benchmark gates.
* Boundary: both schemas stay L10 `reserved` through L11-E. L11-F must deliver encoders, strict decoders, identity proofs, source-free fixtures, reverse-order determinism, compatibility handling, and the registry transition atomically.
* Verification: `./scripts/verify-l11d-trace-cost-contract.sh`, `./scripts/verify-phase-l-specifications.sh`, and `git diff --check` pass. The verifier runs the inherited L11-C/L11-B/L11-A/L10/L3-L9 audits and is included in `just check`.

Historical verification context

* Slice: L11-C - Workspace Tooling Projections
* Summary: adds a byte-only core reader for negotiated L3 workspace snapshot and graph documents. It delegates to the existing strict decoders, retains validated snapshot identity, and rejects reserved/unknown/unreadable schemas without filesystem, source, compiler-service, cache, workspace, or watch access.
* Key files: `crates/presolve_compiler/src/tooling_reader.rs`, `scripts/verify-l11b-tooling-product-readers.sh`, `justfile`, `2026-W28.md`
* Verification: three focused reader tests prove strict decoding, source exclusion, schema negotiation/rejection, unsupported-reader rejection, and reverse-order determinism. The L11-B verifier runs inherited L11-A/L10/L3-L9 audits, formatter, strict compiler clippy, and diff check; it is included in `just check`.
* Fixture correction: the historical L3 compatibility fixtures are intentionally structural zero-identity documents and not strict-decodable products. L11-B preserves them byte-for-byte and uses source-free canonical documents generated through the existing public L3 API solely in the focused reader proof.
* Proof: representative constructed Rust configurations first pass existing L3 validation, retain byte-identical L3 serializer fixtures, then CLI encode/decode to equal Rust values and equal existing L3 configuration identities. CLI fixture bytes are canonical and intentionally differ from L3 bytes. Twenty shuffled object-order byte inputs produce the same Rust configuration.
* Isolation: no public L3 decoder, durable migration, or cross-codec byte equality exists. The verifier rejects any newly introduced L3 configuration decoder and fails if L4/L6/L7 durable code imports the CLI codec; existing L4/L7 restart tests remain inherited evidence.
* Verification: `cargo test -p presolve-cli configuration_codec --lib -- --nocapture`, `cargo clippy -p presolve-cli --all-targets -- -D warnings`, `./scripts/verify-l9a1-configuration-codec-contracts.sh`, inherited `just check`, and `git diff --check` pass. The script is included in `just check`.
* Observer boundary: callers observe and read inputs, then submit complete exact replacement L7 workspace requests. The service does no filesystem watch/read/scan/poll/glob/manifest discovery and exposes no public watch CLI, dev server, HMR, browser refresh, or streaming transport.
* Scheduler: test and internal execution use explicit monotonic scheduler turns. Quiet and maximum deadlines are caller-clock values; zero debounce coalesces before the next turn. Pending input is one transient complete candidate only, replaced at highest accepted sequence; the source-free evidence union is retained for reporting.
* Publication/lifecycle: every execution delegates unchanged to `compile_workspace_v1`; L7 serial scheduling/publication, L5 ephemeral package reuse, and L6 complete-result cache behavior remain authoritative. Obsolete active success is discarded at the watch layer without rollback. Sessions are process-local and never restored; stop releases pending input.
* Verification: `cargo test -p presolve-compiler watch --lib`, `cargo test -p presolve-compiler l8_explicit --lib -- --nocapture`, `./scripts/verify-l8-watch-contracts.sh`, and inherited `just check` pass. The L8 script runs fake-clock/debounce/coalescing/supersession/journal/determinism (20 fresh runs), direct L7 delegation, L3-L7 audits, formatter, strict clippy, fixture/source-exclusion audit, and `git diff --check`.
* Workspace boundary: manifests and complete package requests are caller-owned. Explicit edges schedule serial package order only; they do not create cross-package semantics, discovery, artifact linking, or package-cache key changes. Package publication is atomic per package; workspace state publishes only after all packages succeed.
* Cache boundary: only canonical snapshot/graph/response metadata is persisted after L4/L5 publication. Cache payloads contain no authored source text, parser products, ASTs, request frames, or durable L5 baselines. A restart can hit a complete result but restores no parser reuse products.
* Cache authority: exact SHA-256 length-delimited keying binds compiler/service/schema/feature/platform/configuration/source-universe/mode/artifact/diagnostic/codec identities. Absent, disabled, corrupt, incompatible, or write-failed cache state falls through to L5 without changing compiler semantics.
* Verification: `./scripts/verify-l6-persistent-cache-contracts.sh` passes independently and is in `just check`; focused cases cover initialization, restart hit, corruption fallback, disabled cache, source sentinel exclusion, inspection, and owned cleanup. Stop at L7; no workspace discovery, watch mode, public cache CLI, or remote cache work has started.
* Reuse authority: only unchanged L3 parser products are eligible. A content edit invalidates its typed `SourceUnitId` plus the transitive reverse closure of canonical `WorkspaceGraph` compile edges. L3 validates every offered product's source unit, source revision, product key, and normalized path before use; parser/binder/semantic/artifact work remains L3-owned.
* Fallback authority: L3 v1 does not expose product-granular source-universe membership dependency edges, so additions/deletions/rename representation clean-fallback with `L5F009_SOURCE_UNIVERSE_MEMBERSHIP_UNMODELED`; configuration changes clean-fallback with `L5F002_CONFIGURATION_CHANGED`; malformed retained baselines clean-fallback with `L5F006_MALFORMED_BASELINE_GRAPH`.
* Persistence/lifecycle: the one L5 baseline is held only in `DurableSession` memory after L4 atomic publication and stores configuration/source fingerprints, snapshot/graph, and immutable normalized parser products—not authored source text. Service restart or a new session starts cold. Failed candidates and failed publication do not install a baseline.
* Equivalence: explicit test-only verification uses an isolated clean L3 `RequestedCompilationMode::Full` compile and compares canonical snapshot and graph bytes before publication. The focused deterministic sequence runs 20 fresh sessions and returns byte-identical plan/report output; no source text appears in the durable tree.
* Verification: `./scripts/verify-l5-incremental-contracts.sh` passes independently; it is in `just check`. It runs focused service tests, 20-run determinism, equivalence, source-persistence/no-discovery audits, formatter, strict compiler clippy, frozen L3/L4 audits, and diff check. L5 ends before L6 persistent-cache work.

Current phase boundary

* Slice: L12-C language-service binding contract.
* Status: blocked pending an explicit selection of compiler-owned WASM ABI, native addon, or an amendment to a Rust-native API. No compiler invocation, source discovery/reparse, update synthesis, duplicate decoder, persistence, LSP, or extension may begin first.
* Completed: Phase C1 through C35; Phase D1-A through D7-E; Phase E1 through E21; Phase F1 through F20; Phase G1 through G20; Phase H1 through H21; Phase I0 through I20; Phase J0 through J21; Phase K0 through K21.
* Remaining in Phase K: none.

Verification

* L4 verification: `./scripts/verify-l4-service-contracts.sh` checks service schema constants, canonical fixtures, framing, L3 decode boundary, temporary-write/atomic-rename publication, and the absence of network transport or source persistence. Focused service tests prove exact framing plus complete-candidate L3 compilation and durable canonical snapshot/graph publication without source text in durable commit metadata.

* L3 verification: `./scripts/verify-phase-l-specifications.sh` and `./scripts/verify-l3-platform-contracts.sh` pass. The L3 audit checks all five schema constants, canonical fixture presence and newline termination, no absolute-path/timestamp/cache-persistence/socket surface, and the existing parser/application-model authority boundary. Focused platform tests cover path normalization, identity stability, reversed source discovery, snapshot change classification, and cancellation rollback. `cargo fmt --all --check`, strict all-feature workspace clippy, `RUST_TEST_THREADS=1 cargo test --workspace --all-features`, independent `just check`, and `git diff --check` are the required final gate.

* L2 verification: `./scripts/verify-repository-layout.sh`, `./scripts/verify-phase-l-specifications.sh`, and `./scripts/verify-public-identity.sh` pass. The repository audit verifies the authoritative Phase L index, forbids speculative root `compiler/`, `runtime/`, and `cli/` directories, rejects archived schema/fixture directories and tracked generated or credential-like material, checks active control files, current root ownership, archive navigation, and no active automation path into the archive. Exact `cmp` checks prove all six supplied Phase L documents were tracked byte-for-byte. `pnpm -r check` and `pnpm -r test` pass; the existing JavaScript packages report their current placeholder checks/tests. `cargo test -p presolve-cli --test production_baseline` passes both generated-output baseline/determinism tests. `cargo fmt --all --check`, strict all-feature workspace clippy, `RUST_TEST_THREADS=1 cargo test --workspace --all-features`, independent `just check`, and `git diff --check` pass. No standalone schema validator or documentation-link framework exists; the repository audit is the established closest check for schema placement and archive/specification navigation.

* L1 verification: `cargo check --workspace`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `RUST_TEST_THREADS=1 cargo test --workspace --all-features`, independent `just check`, `./scripts/verify-public-identity.sh`, `cargo run -q -p presolve-cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json`, and `git diff --check` pass. The serialized workspace run passes 4 CLI units, 8 Component fixtures, 12 Context fixtures, 132 inspection/build tests, 2 production baseline tests, 2 production budget tests, 2 production runtime fixtures, 37 real-browser probes, 448 core tests, 13 parser units, and 26 parser integrations. The compiler builds and tests exclusively through the renamed `presolve-cli` package and `presolve` binary; frozen generated artifacts remain covered by the existing baseline and fixture assertions.

* K21 verification: the exact sequential gate passes `cargo fmt --all --check`, strict all-feature workspace clippy, `RUST_TEST_THREADS=1 cargo test --workspace --all-features` (686 tests: 4 CLI units, 8 Component fixtures, 12 Context fixtures, 132 inspection/build tests, 2 production baselines, 2 production budgets, 2 production runtime fixtures, 37 serialized browser probes, 448 core tests, 13 parser units, and 26 parser integrations), `just check`, and `git diff --check`. K21 also repairs exact resume Forms/Context/Slot execution exposed by the final browser gate, freezes the corrected K0 byte baseline, and records corpus/budget identities plus the 552-byte positive shared-candidate proof and 100-cycle 0/2 lifecycle registry result.

* J13 verification: hardened nested browser proof restores three distinct Context slots and exact default/outer/nearest Consumer selections without initial evaluator or Effect execution. A separate malformed R7 proof selects one clean cold fallback with no retained registry or reselection. All 410 core tests, 2 CLI units, 7 Component fixtures, 12 Context fixtures, 128 CLI inspection/build tests, all 33 sequential browser probes, strict all-target core/CLI clippy, formatting, generated-runtime syntax, and `git diff --check` pass.

* J12 verification: hardened repeated-instance browser proof restores State 7/11, recomputes exact Computed values 14/22 once each in canonical order, retains clean isolated caches, and runs zero initial Effects. The six-page fallback matrix adds invalid-codec atomic rejection. All 410 core tests, 2 CLI units, 7 Component fixtures, 12 Context fixtures, 128 CLI inspection/build tests, all 31 sequential browser probes, strict all-target core/CLI clippy, generated-JavaScript syntax, formatting, and `git diff --check` pass.

* J11 verification: hardened five-page real-browser matrix covers no-snapshot cold plus numeric action execution, accepted resume, build mismatch, runtime-protocol artifact rejection, malformed snapshot, empty rollback registries, exact activation lookup, double-bootstrap rejection, and suppression of authored cold initialization. The pass token does not occur literally in probe source. All 410 core tests, 2 CLI units, 7 Component fixtures, 12 Context fixtures, 128 CLI inspection/build tests, all 30 sequential real-browser probes, strict all-target core/CLI clippy, formatting, and `git diff --check` pass.

* J10 verification: 5 focused marker-plan/HTML tests, all 410 core tests, 2 CLI units, 7 Component fixtures, 12 Context fixtures, all 128 CLI inspection/build tests, all 29 sequential real-browser probes, strict all-target core/CLI clippy, formatting, and `git diff --check` pass. Coverage includes every marker kind, Form and ordinary events, static-only omission, missing/unstable/duplicate/wrong-kind/structural-pair failures, exact page/manifest agreement, source-reversal/path-independent build identity, and byte determinism.

* J9 verification: 8 focused manifest tests, all 405 core tests, 2 CLI units, 7 Component fixtures, 12 Context fixtures, all 128 CLI inspection/build tests, all 29 sequential real-browser probes, strict all-target core/CLI clippy, formatting, and `git diff --check` pass. Coverage includes v5/unknown-field rejection, every endpoint family, canonical snapshot/manifest examples, executable sensitivity, provenance and absolute-source-root independence, reverse-order determinism, `resume.runtime.json` emission, and exact page-embedded byte equality.

* J8 verification: `cargo test -p presolve_compiler --lib` passes all 400 core tests; strict all-target core clippy, formatting, and `git diff --check` pass. Five focused proofs cover all R0-R20 phases, retained/recomputable assignment, exact Form phase classes, dangling program references, wrong phases, duplicate writes, missing completion, parent-order rejection, reverse-input determinism, and `PSASM1359`-`PSASM1362`.

* J7 verification: `cargo test -p presolve_compiler --lib` passes all 395 core tests; strict all-target core clippy, formatting, and `git diff --check` pass. Nine focused proofs cover every value variant, canonical object/number encoding, negative zero/non-finite rejection, unsupported runtime shape rejection, retained-only program generation, equal-state byte equality, no mutation/timestamp, full quiescence, stable Form submission, malformed product rejection, reverse-input determinism, and `PSASM1355`-`PSASM1358`.

* J6 verification: `cargo test -p presolve_compiler --lib` passes all 385 core tests; strict all-target core clippy, formatting, and `git diff --check` pass. Focused proofs cover canonical object order, explicit nullable codecs, non-null union rejection, one schema per boundary, exact J2 slot reciprocity, reverse-input determinism, frozen Form runtime slot codecs, and `PSASM1349`-`PSASM1354`. The J7 entry audit additionally corrected schema collection order to preserve J3 parent-before-child ordering; all 7 focused schema tests and strict all-target core clippy pass after the correction.

* J5 verification: `cargo test -p presolve_compiler --lib` passes all 379 core tests; strict all-target core clippy, formatting, and `git diff --check` pass. Focused proofs cover one eager root, isolated interaction roots, no lazy dependencies/shared chunks, exact action isolation, reversed-input determinism, and `PSASM1343`-`PSASM1348`.

* J4 verification: `cargo test -p presolve_compiler --lib` passes all 376 core tests; strict all-target core clippy, formatting, and `git diff --check` pass. Focused proofs cover Eager/Interaction/None assignment, eager Forms runtime with interaction-scoped submit, zero Visible/Manual output, fixed precedence, and `PSASM1337`-`PSASM1342`.

* J3 verification: `cargo test -p presolve_compiler --lib` passes all 372 core tests; `cargo clippy -p presolve_compiler --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` pass. Focused proofs cover parent-before-child application/Component/structural/Form ownership, structural-template event ownership, ordinary and Form-submit activation references, upstream Component/liveness block preservation, reverse-input determinism, invalid parent reciprocity, and the complete `PSASM1328`-`PSASM1336` range.

* J2 verification: `cargo test -p presolve_compiler --lib` passes all 368 core tests; `cargo clippy -p presolve_compiler --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` pass. Focused proofs cover repeated exact State/Computed slots, one shared instance-qualified Context provider slot with exact State dependency evidence, all six Form v5 slot classes, deterministic transitive Computed evidence, input-order determinism, classification uniqueness, and dedicated policy/owner/boundary/proof integrity failures.

* J1-A verification: `just check` passes formatting, strict workspace clippy, 2 CLI units, 7 Component fixtures, 12 Context fixtures, 128 CLI inspection/build tests, 29 sequential real-browser probes, 364 core tests, 13 parser units, and 26 parser integrations. Focused browser proofs cover repeated State cold initialization, A-only action/binding updates, computed invalidation isolation, exact State-slot-only runtime keys, malformed/missing projection failure, Phase J component-v2 rejection, and retained legacy v2 cold compatibility.
* J1-A fixture repair: all template-manifest goldens now match the already-authorized v4 ordinary-instance contract, including canonical action `storage_id` operands; stale Phase G/H schema assertions now acknowledge template v4/component artifact v3.

* J1-C verification: `cargo test -p presolve_compiler --lib` (359), `cargo clippy -p presolve_compiler --all-targets -- -D warnings`, `cargo fmt --all --check`, `git diff --check`, and `cargo test -p presolve_cli --test runtime_browser` (27 real-browser probes) all pass. The component-runtime watchdog was a harness pipe backpressure failure, not a runtime-readiness regression: Chrome filled the piped DOM output while the runner waited to read it. The harness now drains both streams concurrently; the probe keeps its 20-second watchdog, removes parsed JSON metadata before DOM dumping, and reports terminal runtime errors immediately.

* J1-P implementation audit: ordinary targets/bindings/events are compiler projections of Phase H `ComponentInstanceId` and canonical template IDs. Runtime sees only exact marker indexes and `RuntimeExecutionContext`; it does not infer ownership from names, DOM ancestry, order, or counters. J1-P emits no J10 resume markers.
* J1-P verification: `cargo test -p presolve_compiler --lib`, `cargo clippy -p presolve_compiler --all-targets -- -D warnings`, `cargo fmt --all --check`, `git diff --check`, `just check`, and `pnpm test:e2e` completed for the bridge. The focused marker/registry/manifest tests cover repeated component instances, paired binding markers, v4/v3 pair enforcement, reciprocal Forms target records, deterministic projections, and absence of J10 markers.

* J1 `cargo test -p presolve_compiler resume_identity::tests::j1_resume_identities_are_typed_deterministic_and_instance_qualified`: pass
* J1 `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* J1 `cargo fmt --all --check`: pass
* J1 `git diff --check`: pass

* J0 `cargo test -p presolve_compiler j0_reserves_the_public_and_internal_resumability_ranges_without_products`: pass (1 focused entry-freeze test)
* J0 `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* J0 `cargo fmt --all --check`: pass
* J0 `git diff --check`: pass

* Phase I freeze repair `cargo test -p presolve_cli --bin presolve_cli i17_forms_inspection_projects_validation_rules_in_schema_v9`: pass (the committed Forms inspection now positively asserts `validation-rule` projection in ASM v9)
* Phase I freeze repair `cargo clippy -p presolve_cli --all-targets -- -D warnings`: pass
* Phase I freeze repair `cargo fmt --all --check`: pass
* Phase I freeze repair `git diff --check`: pass
* Phase I freeze repair `just check`: pass (strict workspace clippy; 2 CLI units; 7 Component fixtures; 12 Context fixtures; 128 CLI tests; 27 sequential real-browser tests; 350 core tests; 13 parser units; 26 parser integrations)

* I17 `cargo test -p presolve_compiler --lib`: pass (348 core tests, including canonical Forms inspection projection coverage)
* I17 `cargo test -p presolve_cli --test explain --test component_fixtures --test context_fixtures`: pass (128 inspection/build, 7 component, and 12 context tests)
* I17 `cargo clippy -p presolve_compiler -p presolve_cli --all-targets -- -D warnings`: pass
* I17 `cargo fmt --check`: pass
* I17 `git diff --check`: pass

* I15 `cargo test -p presolve_compiler --lib`: pass (345 core tests)
* I15 `cargo test -p presolve_cli`: pass (including Forms build/artifact/embed coverage and 26 browser tests)
* `cargo clippy -p presolve_compiler -p presolve_cli --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* `git diff --check`: pass

* I14 `cargo test -p presolve_compiler`: pass (343 core tests, including versioned instance-qualified registry coverage)
* `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I13 `cargo test -p presolve_compiler`: pass (342 core tests, including immutable Form IR optimization coverage)
* `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I12 `cargo test -p presolve_compiler`: pass (341 core tests, including instance-qualified Form IR coverage)
* `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I11 `cargo test -p presolve_compiler`: pass (340 core tests, including focused I11 reset planning)
* `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I10 `cargo test -p presolve_parser -p presolve_compiler`: pass (13 parser unit, 26 parser integration, 339 core tests including 2 focused I10 tests)
* `cargo clippy -p presolve_parser -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I9 `cargo test -p presolve_parser`: pass (13 parser unit, 26 parser integration tests)
* I9 `cargo test -p presolve_compiler`: pass (337 core tests, including 2 focused I9 tests)
* `cargo clippy -p presolve_parser -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I8 `cargo test -p presolve_compiler`: pass (334 core tests, including 4 focused I8 tests)
* `cargo clippy -p presolve_compiler --all-targets -- -D warnings`: pass
* `cargo fmt --check`: pass
* git diff --check: pass

* I7 `just check`: pass (formatting, strict workspace clippy, 331 core tests, 12 parser unit tests, 26 parser integration tests, 2 CLI unit tests, 7 Component fixture/freeze tests, 12 Context fixture/freeze tests, all 126 CLI inspection/build tests, and all 26 real-browser tests)
* `cargo test -p presolve_compiler form_validation_plan --lib`: pass (5 focused I7 planning tests)
* `cargo test -p presolve_compiler semantic_id::tests::derives_distinct_form_definition_instance_and_field_identities --lib`: pass (I7 identity extension)
* `cargo clippy -p presolve_compiler --all-targets --all-features -- -D warnings`: pass
* `cargo fmt --all --check`: pass
* git diff --check: pass

* I6 `just check`: pass (formatting, strict workspace clippy, 326 core tests, 12 parser unit tests, 26 parser integration tests, 2 CLI unit tests, 7 Component fixture/freeze tests, 12 Context fixture/freeze tests, all 126 CLI inspection/build tests, and all 26 real-browser tests)
* `cargo test -p presolve_compiler form_validation`: pass (7 focused I6 tests)
* `cargo test -p presolve_parser validation`: pass (2 focused parser-retention tests)
* `cargo clippy -p presolve_parser -p presolve_compiler -p presolve_cli --all-targets --all-features -- -D warnings`: pass
* `cargo fmt --all --check`: pass
* git diff --check: pass

* I0 entry `just check`: pass (workspace formatting, strict workspace clippy, 292 baseline core, 6 parser unit, 26 parser integration, 1 CLI unit, 7 component fixture/audit, 12 Context fixture/freeze, 126 CLI inspection/build, and 26 real-browser tests)
* `cargo test -p presolve_compiler`: pass (295)
* `cargo test -p presolve_compiler semantic_id::tests::derives_distinct_form_definition_instance_and_field_identities -- --nocapture`: pass (1)
* `cargo test -p presolve_cli --test component_fixtures phase_h_freezes_authorities_schemas_and_no_discovery_contract -- --nocapture`: pass (1)
* `cargo test -p presolve_cli --test component_fixtures -- --nocapture`: pass (7)
* `cargo clippy -p presolve_compiler -p presolve_cli --all-targets -- -D warnings`: pass
* `cargo fmt --all --check`: pass
* git diff --check: pass
* `cargo test -p presolve_compiler form_ownership::tests -- --nocapture`: pass (7)
* `cargo test -p presolve_parser -p presolve_compiler`: pass (10 parser unit, 26 parser integration, 319 core)
* `just check`: pass (formatting, strict workspace clippy, 319 core, 10 parser unit, 26 parser integration, 1 CLI unit, 7 component fixtures, 12 Context fixtures, 126 CLI inspection/build, and 26 real-browser tests)
* git diff --check: pass
* `cargo test -p presolve_parser -p presolve_compiler`: pass (10 parser unit, 26 parser integration, 312 core)
* `cargo test -p presolve_compiler form_binding::tests -- --nocapture`: pass (6)
* `cargo test -p presolve_cli --test explain`: pass (126)
* `cargo test -p presolve_cli --test component_fixtures -- --nocapture`: pass (7)
* `cargo clippy --workspace --all-targets -- -D warnings`: pass
* `cargo fmt --all --check`: pass
* git diff --check: pass
* `cargo test -p presolve_parser -p presolve_compiler`: pass (8 parser unit, 26 parser integration, 301 core)
* `cargo test -p presolve_compiler form::tests -- --nocapture`: pass (6)
* `cargo test -p presolve_cli --test explain`: pass (126)
* `cargo test -p presolve_cli --test component_fixtures phase_h_freezes_authorities_schemas_and_no_discovery_contract -- --nocapture`: pass (1)
* `cargo clippy --workspace --all-targets -- -D warnings`: pass
* `cargo test -p presolve_parser -p presolve_compiler`: pass (9 parser unit, 26 parser integration, 306 core)
* `cargo test -p presolve_compiler form_field::tests -- --nocapture`: pass (5)
* `cargo test -p presolve_cli --test explain`: pass (126)
* `cargo test -p presolve_cli --test component_fixtures phase_h_freezes_authorities_schemas_and_no_discovery_contract -- --nocapture`: pass (1)
* `cargo clippy --workspace --all-targets -- -D warnings`: pass
* `cargo fmt --all --check`: pass
* git diff --check: pass

Architecture decisions made

* Decision: J9 is the sole executable resume authority at schema v6 and preserves Phase I semantic meaning only through normalized records inside that manifest.
* Reason: Runtime validation and later marker/loader slices need one closed cross-referenced product rather than competing v5 planning and v6 execution structures.
* Tradeoff: v5 is rejected with no adapter. Anchor/event arrays are intentionally empty until J10, and no J11 runtime consumption begins early.

* Decision: `ResumeBuildId` uses SHA-256 lowercase hexadecimal over framed canonical executable inputs, with a fixed zero-sentinel manifest ID and canonical absolute-source-root normalization.
* Reason: The fingerprint must change for executable behavior while remaining independent of provenance, absolute paths, time, output directory, and build machine.
* Tradeoff: Build identity performs a second deterministic projection of existing artifacts during manifest construction; it does not make provenance or paths part of runtime authority.

* Decision: J0 reserves `PSC1096` through `PSC1111` in exact roadmap order and internal `PSASM1289` through `PSASM1384` inclusive (96 codes).
* Reason: Later Phase J validators and the J19 projector need monotonic, non-overlapping diagnostic space without prematurely creating identities, schemas, manifests, snapshots, chunks, or runtime behavior.
* Tradeoff: The reservations are inert metadata and a freeze test only; diagnostics remain unprojected until J19.

* Decision: J1-A owns the sole `StateInstanceSlotId` constructor from the exact typed pair `(ComponentInstanceId, IrStorageId)` and the immutable registry ordered by canonical instance then storage.
* Reason: declaration-level IR storage remains the correct program operand, but repeated component executions require exact mutable addresses before J2 can classify retained values.
* Tradeoff: component artifact v3 removes `instance_storage_prefix`; manifest v4 actions carry a canonical storage operand; the browser builds the closed pair-to-slot index only from serialized records. Manifest v3/component artifact v2 remains a cold-only legacy pair and cannot participate in Phase J resume products.

* Decision: J2 classifies the closed set of existing compiler-owned runtime slots into retained, recomputable, excluded, or blocked records and builds deterministic indexes by exact slot, owner, boundary candidate, and policy reason.
* Reason: capture and restore slices need one canonical liveness authority that preserves mutable State and Form values, proves pure eager Computed recomputation transitively, retains exact Context source dependencies, excludes Effect-body state, and fails closed for unsupported required values.
* Tradeoff: J2 creates no boundary graph or executable resume artifact. Instance-selected Context values combine the exact Phase H runtime binding with the canonical Context evaluation source entry; a shared provider remains one shared runtime slot even when multiple consumers read it.

* Decision: J3 retains structural-template Component instances as distinct Component boundaries parented by their exact structural-region boundary, while Interaction boundaries carry activation references and no ownership parent.
* Reason: Phase H ordinary events can belong to a structural-template instance, and the graph must give that event an exact owner without collapsing the Component into its region or discovering a parent from DOM ancestry. Form submits likewise need both their Component host and Form boundary references.
* Tradeoff: Structural-template boundaries currently own no J1-A/J1-C storage because those registries project executable planned instances only. J3 records the identity/parentage without inventing dynamic instance storage; later policy/program slices must preserve this boundary.

* Decision: J4 emits Eager for application infrastructure, immediate Forms runtime, and post-restore recomputation; Interaction only for exact event/submit activation roots; and None only where no independent executable work is present.
* Reason: This applies the frozen correctness precedence without cost heuristics and preserves earlier immediate browser behavior.
* Tradeoff: no earlier product authorizes Visible or Manual, so both sets are empty. Structural work is reached through exact interaction program closure rather than receiving a fabricated activation root.

* Decision: J5 duplicates required generated programs into each exact lazy root and permits no lazy-to-lazy dependency or shared lazy chunk.
* Reason: This is the frozen Phase J v1 isolation contract and keeps activation roots independently loadable without size heuristics.
* Tradeoff: deterministic duplication is accepted until Phase K; J5 plans module bytes/paths only and does not emit runnable chunks.

* Decision: J6 includes exactly J2 retained and recomputable slots in one schema per J3 boundary; J2 blocks remain schema blocks and excluded Effect scheduler metadata has no codec.
* Reason: Capture and restore generation need exact reciprocity with liveness and boundaries without turning blocked or intentionally excluded runtime metadata into snapshot values.
* Tradeoff: Form validation, aggregate validity, and submission state use frozen compiler-owned types matching the existing runtime representation; unsupported tuples, resources, and non-null unions fail closed until a later contract extends the codec vocabulary.

* Decision: J7 emits one closed read/encode/append triple per retained slot and omits recomputable slots from snapshot values.
* Reason: Snapshot capture must consume J2 liveness and J6 schemas exactly without replaying computation, walking arbitrary runtime objects, or serializing values that J8 will regenerate.
* Tradeoff: Snapshot model v1 is internal until J9. Form submission uses the existing runtime string states with an explicit stable-state allowlist; pending/unknown states and any non-quiescent runtime are rejected immediately.

* Decision: J8 assigns every retained/recomputable slot to the fixed R0-R20 sequence, decodes only retained values, and emits one recompute instruction per exact Computed cache pair.
* Reason: Resume boot must restore storage without replaying authored initialization and must establish deterministic readiness parent-before-child before any interaction can execute.
* Tradeoff: R16 DOM-binding and R17 Effect-subscription phases are present but carry no instructions until their later slices provide canonical anchors and runtime establishment authority; J8 does not fabricate those references.

* Decision: I7 creates one declaration-level `ValidationPlanId::for_form(FormId)` for every valid Form, including empty Forms, and one `FieldDependencyId::for_rule_and_source(ValidationRuleId, FieldId)` for each eligible I6 direct dependency edge.
* Reason: Form plans and dependency records need stable typed names independent of Rule counts, source order, Component instances, runtime registration, or DOM identity; future runtime planning can refer to a complete Form plan without using absence as policy.
* Tradeoff: Plans and dependencies are internal immutable products, not semantic owners or public schema entities. I7 creates no instance-qualified plan, runtime state, or artifact.

* Decision: I7 retains both Rule-to-source read facts and source-Field-to-Rule invalidation indexes, but schedules only direct dependencies after a committed abstract Field write. Change sets normalize changed Fields in Field-authored order, schedule Rules by target Field then rule order then Rule ID, deduplicate Rules/targets, and retain every triggering dependency ID.
* Reason: Validation reads Field values; a validation result is not a Field write. Direct-only scheduling prevents a cross-Field chain from becoming an invented Rule-to-Rule execution graph or transitive invalidation engine.
* Tradeoff: Unary Rules, target-Field writes, initial validation, submission, reset, dirty/touched transitions, values, browser events, and execution remain outside I7.

* Decision: I7 integrity validation derives internal `PSASM1242` through `PSASM1270` findings for plan/dependency identity, I5/I6 reciprocity, Form/Component/boundary/provenance/order/index consistency, duplicate/missing projections, direct-only leakage, and instance identity leakage. `PSASM1271` and `PSASM1272` detect stale retained validation and stale whole planning products in ASM validation.
* Reason: Later consumers must be able to reject malformed staged products deterministically without re-resolving validation syntax or repairing graph drift.
* Tradeoff: Blocked downstream records retain only malformed/stale canonical I6 evidence and receive no fabricated dependency identity; ordinary invalid authored rules remain solely in I6 candidate registries.

* Decision: The parser retains the outer `@validate` invocation shape and one normalized nested rule-expression fact, including direct call identity, ordered arguments, exact `this.<identifier>` designators, compiler constant expressions, literal strings, spans, and unsupported shapes. Canonical I6 lowering alone resolves target Fields, dependencies, rule kinds, normalized arguments, compatibility, duplicates, contradictions, cycles, and validity.
* Reason: Later diagnostics and planning must consume immutable normalized evidence without reparsing TypeScript, while the parser must not become a semantic validation authority.
* Tradeoff: The parser adds the pinned OXC ECMAScript regular-expression grammar as the pattern-syntax authority and retains module-local/imported value bindings only to reject authored shadows of compiler-owned rule names; it does not execute validators or resolve Forms/Fields.

* Decision: Every recognized placement receives `ValidationRuleCandidateId`; only a violation-free candidate receives `ValidationRuleId::for_field(FieldId, authored_validation_ordinal)`. The closed catalog is `required`, `min`, `max`, `minLength`, `maxLength`, `pattern`, `email`, `equals`, and `notEquals`, and every valid rule has the Client boundary.
* Reason: Candidate and executable identity domains must remain distinct, while valid identity depends only on the canonical target Field and authored decorator position rather than source allocation, map order, or runtime registration.
* Tradeoff: Invalid candidates retain partial Component/Form/Field/dependency/type/provenance evidence but never acquire a plausible valid rule identity. Public diagnostic codes and messages remain I18 work.

* Decision: Rule compatibility consumes I3's normalized canonical `SemanticType`, Phase C `is_assignable`, serialization compatibility, and the existing constant folder. Numeric and length arguments normalize before duplicate/contradiction analysis, and flags-free `pattern` arguments use the frontend's pinned ECMAScript regex parser.
* Reason: I6 must not introduce a Form-specific type parser, assignability engine, constant evaluator, serializer checker, or regular-expression dialect.
* Tradeoff: Unsupported/unresolved types, values outside the frozen constant subset, non-finite numbers, invalid lengths/patterns, shadowed rule names, cross-Form/cross-Component/self dependencies, and incompatible domains remain candidate violations with no executable rule.

* Decision: Duplicate normalized rule groups, contradictory min/max or length ranges, equals/notEquals pairs on the same dependency, and deterministic strongly connected dependency groups are invalidated wholesale with no source-order winner. Cycle-participating candidates remain retained but cannot enter executable membership.
* Reason: No runtime precedence, fallback, scheduling, or partial-cycle semantics are authorized. Deterministic group exclusion preserves all evidence without inventing execution behavior.
* Tradeoff: I6 records direct dependency facts and cycle products only. I7 must define any invalidation propagation, scheduling, derived update plans, or execution semantics before they can exist.

* Decision: `ValidationGraphId` derives from the same sorted Phase H build-root authority as I5. Graph nodes retain canonical Form, Field, and ValidationRule identities; `FormOwnsField` projects I5, `FieldOwnsRule` projects ASM ownership, and `RuleDependsOnField` projects canonical references.
* Reason: I5 remains the sole declaration-ownership authority, and the validation graph is an immutable typed projection/validator rather than a competing owner map or a syntax-derived graph.
* Tradeoff: Internal `PSASM1221` through `PSASM1239` cover validation-graph integrity, `PSASM1240` detects stale retained graph validation, and `PSASM1241` detects stale candidate/rule products. No public PSC diagnostic or public schema projection changes in I6.

* Decision: `FormOwnershipGraphId` derives from the sorted, deduplicated Phase H `ComponentRootId` set, while graph nodes retain existing Component, Form, Field, template-control, and Field-binding identities in a typed sum key.
* Reason: I5 identifies one application/build projection without manufacturing replacement semantic identities or depending on file order, spans, counts, runtime boot, or map insertion.
* Tradeoff: The graph is an internal product identity, not an ASM semantic entity and not a public serialized schema.

* Decision: Canonical ASM ownership now records each Field binding under its exact intrinsic template-control entity. The I5 projection consumes that owner together with the already-frozen `FieldBindingField` and `FieldBindingForm` references; Forms or Fields never own their template use sites.
* Reason: The canonical control is the narrowest authored use-site identity and already participates in the Template ownership tree. Direct control ownership makes the target relationship exact without duplicating a `TargetsControl` reference.
* Tradeoff: `FormFieldBinding.owner_template` remains the render-template metadata needed for authored ordering, while generic `owner_of(binding)` returns the exact control.

* Decision: I5 validates immutable graph structure and canonical-product reciprocity through internal `PSASM1203` through `PSASM1220` integrity facts. Validation is deterministic, retained on the graph, recomputed by ASM validation, and never repairs malformed input.
* Reason: Later compiler stages require trustworthy exact owners, endpoints, provenance, root reachability, acyclicity, component isolation, and ordering without returning to syntax or selecting fallback semantics.
* Tradeoff: These are internal integrity diagnostics only; I18 still owns public Form diagnostics and I6 owns value-validation language semantics.

* Decision: Component and Form instance identities are excluded from the declaration graph. Focused tests prove repeated component instances retain one declaration-level Form node while the reserved `(ComponentInstanceId, FormId)` constructor remains deterministic and distinct.
* Reason: Phase H instance topology remains frozen, and no authorized runtime-planning slice has projected declaration Forms into mutable executions.
* Tradeoff: Runtime Form/Field state cannot use declaration IDs directly; its qualification and creation authority remain deferred.

* Decision: I4 recognizes exactly one compiler-owned `field={this.<identifier>}` JSX attribute on intrinsic `input`, `textarea`, or `select` elements. The direct member resolves only among valid I3 Fields authored on the template's canonical component, and Form identity is copied only from `FormFieldEntity.owner_form`.
* Reason: The supplied contract makes the I3 declaration the sole Form-membership authority and prohibits HTML `name`, separate `form` attributes, template ancestry, Context, state/computed/method lookup, component adapters, imports, runtime instances, and DOM discovery.
* Tradeoff: Custom controls, authored `<form>` ownership, forwarding/adapters, file inputs, checkbox groups, and uncontrolled bindings remain absent.

* Decision: The normalized parser/render/template boundary retains raw immutable attribute facts, exact direct-this designators, literal values, expression/name/value spans, spreads, conflicts, and complete control children alongside backend-facing executable attributes. The compiler-only `field` attribute is deliberately excluded from executable Template attributes.
* Reason: I4 diagnostics and later planning must consume canonical candidate provenance without reparsing JSX, while no runtime or HTML path may treat the semantic binding marker as an ordinary dynamic DOM attribute.
* Tradeoff: Typed Template elements now carry an additional compiler-analysis attribute vector, but existing HTML, manifest, runtime, and browser contracts remain unchanged.

* Decision: Every recognized binding gets `FormFieldBindingCandidateId`; only a violation-free occurrence gets `FieldBindingId`, derived from its canonical control semantic entity plus `FieldId`. Valid bindings are Template-owned use sites with dedicated `FieldBindingField` and `FieldBindingForm` references; they never own the Form or Field.
* Reason: A Field declaration and each authored control occurrence are different identity domains. Candidate-only invalid identity preserves ambiguity, duplicate, partial resolution, compatibility, and provenance evidence without fabricating executable semantics.
* Tradeoff: Frozen semantic graph v5 and CLI ASM inspection v8 filter bindings and their references until a roadmap-owned versioned projection. Internal `PSASM1202` detects retained-product drift.

* Decision: Control channels and normalization are compiler-selected immutable metadata. Text/null, numeric/null, checkbox, radio, single-select, and multiple-select compatibility reuse canonical `SemanticType`, literal-value typing, and `is_assignable`; non-radio multiplicity considers otherwise-valid candidates only, while radio groups use exact `(ComponentId, FieldId)` ownership and static value identity.
* Reason: Later runtime execution must consume an exact channel and normalization policy without inspecting DOM element type or reconstructing group membership. Invalid controls must not poison unrelated valid bindings merely because partial Field resolution succeeded.
* Tradeoff: I4 records no read/write execution, event ordering, dirty/touched state, validation, submission, reset, serialization, IR, runtime artifact, or resumability behavior.

* Decision: I3 accepts only a directly authored canonical component instance field decorated by exactly one invoked `@field(this.<formName>)`. The designator must resolve to one valid, nonduplicate I2 `FormEntity` authored on the same component, and the authored identifier supplies the Field name.
* Reason: The supplied I3 contract freezes exact declaration-level Form ownership and prohibits default-Form, inheritance, composition, template-ancestry, import, runtime-instance, and DOM-derived resolution.
* Tradeoff: I3 introduces no template-control association, event behavior, validation, submission, serialization/reset plan, dirty/touched state, IR storage, runtime registry, or resume execution.

* Decision: Every recognized `@field` placement receives a source-qualified `FormFieldDeclarationCandidateId`, while only a candidate with no violations receives the existing Form-owned `FieldId` and a `FormFieldEntity`. Duplicate groups retain every candidate, invalidate the whole `(FormId, authored name)` key, and select no source-order winner.
* Reason: `FieldId` was introduced in I1 specifically as the canonical identity of a Field owned by an exact Form. Keeping invalid syntax candidate-only follows the frozen Phase H Slot model and prevents plausible partial paths from becoming semantic identity.
* Tradeoff: Invalid candidates may retain canonical component/Form evidence and complete provenance for I18, but cannot enter executable downstream Field products.

* Decision: I3 reuses Phase C semantic-type lowering, import/local alias resolution, normalization, inference, `is_assignable`, serialization compatibility, and immutable constant folding. A valid initial value is the existing compiler-owned recursive serializable value or an already-supported folded constant expression.
* Reason: Forms must consume canonical immutable authorities rather than create a Form-specific type parser, resolver, evaluator, assignability engine, or serializer checker.
* Tradeoff: Unsupported calls, state wrappers, resource/unknown/never/function-shaped types, unresolved aliases, and values outside the frozen constant subset remain invalid facts; I3 does not independently execute JavaScript.

* Decision: Valid Form Fields participate internally in `Component -> Form -> Field` ownership, canonical provenance, semantic typing, and explicit per-Form authored declaration order. Frozen semantic graph v5 and CLI ASM inspection v8 continue to filter the new entity kind.
* Reason: Later serialization, validation, submission, and reset slices require deterministic declaration order, while I3 explicitly forbids unrelated public inspection/schema changes.
* Tradeoff: Any public projection remains owned by its later roadmap slice and must version the affected schema explicitly.

* Decision: I2 lowers `@form()` only from a directly authored canonical component instance field whose authored identifier supplies the Form name. The field must be declaration-only, non-static, initialized by neither expression nor constructor, and decorated by exactly one invoked zero-argument `@form()`.
* Reason: The supplied I2 contract freezes declaration ownership, naming, multiplicity, arity, and marker semantics. `FormId` therefore derives only from the direct `ComponentId` plus authored field name, and every valid Form has the client execution boundary without implying a JavaScript `Form` object.
* Tradeoff: I2 creates no Form instances, Fields, bindings, validation, submission, serialization, reset, tracking, IR, runtime, inspection, public diagnostics, or resume products.

* Decision: Every recognized `@form` occurrence becomes an immutable `FormDeclarationCandidate`; independently true violations are retained in canonical order. Identity-capable invalid fields retain their deterministic `FormId`, while classes, methods, accessors, parameters, unsupported field names, declarations outside canonical components, and inherited non-declarations never receive fabricated Form identity.
* Reason: Later diagnostics must consume canonical retained facts and provenance without revisiting parser syntax. Duplicate declarations retain all candidates, mark the whole owner/name group invalid, and select no source-order winner.
* Tradeoff: Invalid candidates remain internal ASM inputs and cannot enter downstream executable Form products.

* Decision: The exact `Form` annotation resolves through a compiler-owned module type authority, not downstream text matching. Module-local type declarations and imports named `Form`, aliases, subclasses, unions, and generic applications remain invalid even when structurally compatible.
* Reason: `Form` is a nominal compile-time marker; user-authored lookalikes must never acquire canonical Form semantics.
* Tradeoff: I2 assumes the repository's established global built-in marker model. It does not add an importable runtime `Form` constructor or value.

* Decision: `FormEntity` participates in internal canonical ASM identity, ownership, provenance, and semantic typing, while frozen semantic graph v5 and CLI ASM inspection v8 explicitly filter it.
* Reason: I2 needs a first-class compiler-owned product, but the contract forbids unrelated public schema changes and runtime behavior.
* Tradeoff: A later roadmap-owned inspection/schema slice must version any public Form projection deliberately.

* Decision: Stop before I2 rather than assign `@form()` to a class, property, or method or choose a Form name source.
* Reason: The authoritative roadmap says only "Lower `@form()`". The existing parser retains decorators on all three declaration kinds, while frozen A-H products provide no Form-specific ownership rule. A canonical `FormId` requires both an owner and name, so any lowering choice would add language semantics not present in the roadmap.
* Tradeoff: I0 and I1 are complete and committed, but I2-I20 remain untouched. Continuation requires an amended authoritative contract stating the valid decorated declaration kind, canonical name source, component/Form multiplicity, argument/arity rules, and invalid-candidate retention needed for I18 diagnostics.

* Decision: I1 defines a Form independently of its executions, a Field under its exact Form, and a Form instance from the exact compiler-owned `ComponentInstanceId` plus `FormId`.
* Reason: One component definition may execute in multiple component instances; runtime-generated IDs or definition-ID reuse would collapse dirty, touched, validation, submission, serialization, and reset state across those executions.
* Tradeoff: I1 defines identity composition only. The syntax-owned Form name and the exact set of valid semantic owners remain I2/I3 lowering facts rather than assumptions in the ID layer.

* Decision: I1 consumes existing component-instance identities without constructing or rediscovering component instances. The H21 authority audit remains unchanged and passing.
* Reason: Phase H freezes `ComponentInstanceId` constructors to component-instance planning; Forms must qualify against that immutable product instead of creating parallel instance topology.
* Tradeoff: Form instances cannot exist before a canonical component instance exists, and I1 intentionally creates no fallback identity.

* Decision: I0 reserves `PSC1084` through `PSC1095` in one ordered `FORM_DIAGNOSTIC_RESERVATIONS` authority matching the roadmap's twelve diagnostic meanings.
* Reason: Phase I needs a deterministic range after frozen Phase H without allowing early slices to emit ad hoc diagnostics or duplicate code ownership.
* Tradeoff: The table reserves codes and meanings only. Identity requirements, provenance, suppression, deduplication, messages, and projection remain I18 work over canonical I1-I17 products.

* Decision: I0 retains every frozen Phase H schema exactly and introduces no Forms schema authority before its roadmap-owned slice.
* Reason: The entry audit must prove a clean architectural boundary rather than advance unrelated public products merely because Forms will eventually participate in runtime, resume, inspection, and check output.
* Tradeoff: Forms remain absent from semantic graph v5, resume manifest v4, ASM inspection v8, check JSON v4, template manifest v2, and every runtime artifact until the corresponding Phase I slice deliberately versions its boundary.

* Decision: H21 freezes one authority for each Component/Slot concern from H1 through H20 and makes the runtime's supported component artifact version derive from `RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION` rather than a second JavaScript literal. An executable audit test guards identity-constructor ownership, distinct definition/invocation/instance domains, repeated-instance separation, caller-owned Slot bindings, exact instance Context sources, actual public schemas, CLI projection, closed runtime order, parser-free diagnostics, and no authored-name runtime lookup.
* Reason: Phase H can be frozen only if later compiler, CLI, serializer, and runtime layers consume immutable canonical facts and schema authorities instead of reconstructing or rediscovering them.
* Tradeoff: The audit intentionally enforces architectural source boundaries as well as behavior. Moving an identity constructor or adding a runtime field with an authored-name-shaped identifier now requires an explicit authority/schema review and corresponding audit update.

* Decision: `docs/component-contract.md` is the public Phase H contract and records exact syntax, identity domains, ownership, instance Context selection, cold/action ordering, structural programs, schema versions, diagnostics, unsupported semantics, and no-runtime-discovery invariants. `docs/runtime-contract.md` now records template manifest v2 plus the narrow legacy-v1 compatibility rule and component runtime artifact v2.
* Reason: Consumers need the actual completed contract, including conditional legacy behavior, rather than roadmap target assumptions or stale schema-one runtime prose.
* Tradeoff: Phase H remains deliberately narrow: every unsupported feature listed in the component contract requires a later authoritative slice and deliberate identity/schema review.

* Decision: H20 organizes the authored regression matrix into five fixture families: declaration/import facts, composition and Slot facts, instance Context reprojection, runtime/structural/resume products, and one source file per reserved H19 diagnostic code. Compiler assertions stay in `component_fixtures`; browser-only behavior stays in `runtime_browser`.
* Reason: Every Phase H contract now has an authored, repeatable proof without duplicating semantic authority in fixtures or using browser probes for compiler-only products. Reversing multi-file inputs and rebuilding every serialized surface verifies canonical ordering rather than source discovery order.
* Tradeoff: The five H19 states that canonical valid construction prevents (`PSC1078`, `PSC1079`, `PSC1080`, `PSC1082`, and `PSC1083`) still use the H19-approved mutation of retained authoritative products after parsing their focused source fixture.

* Decision: instance-qualified Context selection supersedes declaration-only Phase G lifetime compatibility only inside H8 composition typing. The frozen Phase G lifetime record and `PSC1065` projection remain available and unchanged.
* Reason: An H6-selected ancestor Provider or root-qualified default outlives its exact Consumer instance even when the retained declaration graph cannot express the composed ancestry. Runtime-facing Phase H eligibility must use that exact instance fact without rewriting frozen Phase G evidence.
* Tradeoff: Valid composed Context fixtures may retain a declaration-level Phase G lifetime diagnostic while emitting no Phase H component diagnostic and producing compatible instance-qualified runtime records.

* Decision: duplicate Slot outlets emit one `PSC1076` per component/Slot-name group, with deterministic secondary outlet evidence, rather than one public diagnostic per duplicated outlet record.
* Reason: H19 freezes group-level deduplication and the H20 duplicate-outlet fixture exposed the per-record projector leak.
* Tradeoff: Independent duplicate outlet groups remain independent findings; only records in the same canonical owner/name group coalesce.

* Decision: H19 has one table-defined diagnostic contract and one canonical projector over H1-H17 ASM products. The projector adds only identities already owned by those products; invalid Slot candidates retain candidate/source evidence without a `SlotId`, and blocked invocations retain their canonical invocation identity without a fabricated target component identity. `PSASM1201` rejects any stored H19 diagnostic vector that differs from recomputation.
* Reason: Check, full ASM, selected-entity ASM, and explain can consume the same validated diagnostic vector and serializer without reparsing source, reconstructing component relationships, or allowing public surfaces to invent identities or labels.
* Tradeoff: The currently authored language subset naturally exercises declaration, invocation, cycle, Slot, and instance-Context failures. Independent H4/H7/H8 failure states used by `PSC1078`, `PSC1079`, `PSC1080`, `PSC1082`, and `PSC1083` are covered by canonical-product mutation tests because valid upstream construction normally prevents those retained states; no authored syntax or runtime fallback was invented to make them reachable.

* Decision: suppression is subject-qualified and precedence ordered: inheritance suppresses downstream findings only for that component; unresolved invocations and cycles suppress only the same invocation; invalid Slot findings suppress only the same invocation or binding; unavailable instance Context suppresses only the same instance; and planning failures suppress only the derivative lowering failure for that instance.
* Reason: This removes cascades caused by an already-reported authoritative failure while preserving diagnostics for unrelated components, invocations, bindings, regions, and instances in the same compilation.
* Tradeoff: Adding a future diagnostic authority requires an explicit precedence and typed subject key in the H19 table/projector rather than relying on message, source proximity, or source order.

* Decision: H13 builds a deterministic `RuntimeComponentRegistry` contract v1 from H10 and H12. It records only planned instances plus executable Slot/instance-Context binding records, all keyed by canonical IDs.
* Reason: The runtime receives exact compiler-selected topology, prefixes, bindings, and batches without any tag/Slot/Context/Provider name lookup, ancestry traversal, Provider selection, or source reconstruction.
* Tradeoff: H13 is metadata only: it neither serializes a public artifact nor allocates, initializes, renders, updates, or destroys components.

* Decision: H12 has an explicit immutable `OptimizedComponentIrReport`, but preserves the full H11 stream because every current operation is observable and H11 already excludes blocked/static-empty work.
* Reason: The optimizer boundary is canonical and testable without granting it authority to merge instances/bindings/Context slots, move caller-owned content, or change parent/child operation order.
* Tradeoff: H12 adds no lowered simplification, runtime behavior, or public schema. Internal `PSASM1200` rejects source/optimized projection drift.

* Decision: H11 lowers immutable H10 instance and Slot batches into a dedicated canonical `ComponentIrReport`. Create, initialize, and materialize operations preserve instance order; Slot binds retain exact caller-owned fragment and callee outlet placement. `DestroyComponentInstance` is a typed reserved operation and is not emitted before H16.
* Reason: Runtime-facing phases receive compiler-owned operation IDs and instance-qualified Context slots without declaration-slot aliases, parser walks, tag/slot-name lookup, DOM inspection, or generic rerendering.
* Tradeoff: H11 does not optimize, serialize, execute, or structurally update the report. Internal `PSASM1199` rejects IR drift; public inspection and schemas wait for their roadmap slices.

* Decision: H10 projects H4 instance topology through the existing `IrUpdateScheduler`, then schedules compatible H7 binding insertion only after exact caller/callee batches. Instance Context source readiness travels with the owning instance batch.
* Reason: The compiler retains stable initial ordering and prerequisites without a second scheduler, runtime traversal, DOM inspection, or executing structural updates early.
* Tradeoff: H10 is initial planning only. It adds no IR/runtime/schema or public diagnostic surface; `PSASM1198` detects retained-plan drift.

* Decision: H9 SCC analysis consumes only canonical resolved H2 definition edges and maps H4 `CompositionCycleBoundary` records to sorted cycle indexes. Slot ownership/content placement and unresolved invocation attempts never contribute edges.
* Reason: Static recursion is inspectable and finite without parser rewalk, runtime discovery, or accidental callee-to-caller edges from lexical Slot ownership.
* Tradeoff: H9 adds no runtime behavior or public diagnostic/schema projection. Internal `PSASM1197` rejects analysis drift; H4 remains the expansion-boundary authority.

* Decision: H8 records invocation, Slot binding, and instance Context compatibility without rewriting any H2, H6, or H7 identity or selection. All Phase H component boundaries are `Client`; unresolved targets retain no fabricated boundary.
* Reason: Later planning and diagnostics can consume one immutable eligibility product while exact `SlotContent`, caller scope, outlet ownership, cardinality, Context typing, serialization, boundary, and lifetime facts remain compiler-owned.
* Tradeoff: An incompatible Context Provider remains selected and identified. Unknown stays conservative, and no props, server/shared boundaries, public diagnostics, runtime behavior, or schema projection is added. Internal `PSASM1196` rejects product drift.

* Decision: H7 creates one callee-instance-qualified `SlotBindingId` per declared Slot, supplied unknown fragment, or otherwise empty blocked invocation boundary. Status is closed over `Bound`, `Empty`, `MissingOutlet`, `UnknownSlot`, `DuplicateContent`, `DuplicateOutlet`, `InvalidOwnership`, and `BlockedInvocation`.
* Reason: Later IR/runtime stages receive exact IDs and cardinality facts for every caller/callee relationship without matching slot names, inspecting templates, or inferring placement at runtime.
* Tradeoff: The binding record deliberately separates caller `content_owner_instance` from callee outlet placement and does not enter generic declaration ownership. H7 adds no props, forwarding, fallback/required Slots, runtime execution, public inspection shape, or user-facing diagnostics; internal `PSASM1195` rejects registry drift.

* Decision: H6 retains Phase G declaration-level Context facts and adds a parallel `InstanceContextRegistry` derived only from exact `ContextId` relations and canonical H5 self-to-root ancestry. `ProviderInstanceId`, `ConsumerInstanceId`, `ContextSourceInstanceId`, root default sources, and value slots are all instance qualified.
* Reason: Repeated component definitions can bind to different Provider instances while every later compiler/runtime product consumes an immutable selected source and never discovers Providers by names, parent traversal, DOM state, or runtime typing.
* Tradeoff: Invalid declarations without canonical Phase G identities remain excluded rather than receiving fabricated identities. Same-scope candidate ordering and ambiguity are represented, but H6 performs no runtime execution, slot binding, public inspection projection, or user-facing H19 diagnostics. Shared ASM validation uses internal `PSASM1194` for a noncanonical retained registry.

* Decision: H5 derives the executable instance scope graph exclusively from H4 `instances`; H4 `blocked` records never enter its nodes, parent map, child map, roots, or traversal queries. Node metadata retains definition, owner root, depth, structural region, and planned/template status.
* Reason: This graph is the sole future runtime ancestry authority while declaration ownership and Phase G `ComponentScopeGraph` remain unchanged. Independent deterministic validation exposes mutated/corrupt topology without inferring parents from templates, imports, DOM, or runtime traversal.
* Tradeoff: Canonical H5 construction is diagnostic-free. Shared ASM validation adds internal `PSASM1192`/`PSASM1193` integrity failures for a graph that diverges from H4 or violates endpoints, reciprocity, depth, roots, cycles, reachability, multi-parent, owner-root, or ordering invariants. No user-facing H19 diagnostic or public schema changes.

* Decision: H4 defines one compiler-owned `ComponentBuildRoot` per routed page when routes exist. Without routes, every valid definition with no incoming resolved invocation is a build entry; if a cycle-only graph has no such definition, the lexicographically first valid component is the single canonical fallback root.
* Reason: The repository has route records and build-all-template behavior but no explicit multi-root build identity. The narrow root product preserves routed pages and disconnected build entries without treating every library definition as a root, while the cycle fallback guarantees a finite plan and an explicit cycle boundary.
* Tradeoff: Root and child `ComponentInstanceId` values encode the root plus the full canonical invocation path. Conditional/keyed-list uses are one `StructuralTemplate` record rather than eager branch/item instances. Resolved recursion stops at a typed cycle boundary; unresolved, dynamic, ambiguous/non-component, and invalid targets are blocked records and never executable instances. H4 performs no scope-graph validation or execution.

* Decision: H3 consumes only immutable template, invocation, and Slot products. Exact direct-child `<template slot="name">` wrappers are compile-time-only named fragments; all other `<template>` elements remain ordinary default content while any dynamic, malformed, or nested Slot-wrapper intent is retained as a blocked fact. Direct non-wrapper children form one `children` fragment, and empty invocations form no incoming fragment.
* Reason: Caller-owned synthetic fragment roots and ordered canonical template-entity roots preserve lexical/semantic ownership separately from later callee placement. Grouping by invocation and requested Slot guarantees one fragment per invocation/Slot while retaining duplicate provenance for H19.
* Tradeoff: Callee `<slot>` elements are compiler directives, not host DOM semantics. Outlets retain missing declarations, duplicate placement, dynamic names, invalid attributes, and unsupported fallback content as typed violations. H3 performs no fragment-to-outlet binding, wrapper DOM removal, instance planning, runtime rendering, or diagnostics projection.

* Decision: H2 recognizes only an exact identifier whose first character is ASCII uppercase as a statically resolvable component invocation. JSX member or namespaced targets whose final segment is PascalCase are retained as `UnsupportedDynamicTarget`; lowercase intrinsic elements are not invocation candidates.
* Reason: Canonical template traversal supplies the template entity and source position, while local component definitions and the existing import binding table supply the only target-resolution authority. Repeated uses therefore retain distinct caller/template/path identities without DOM indexes, runtime names, or source-order target selection.
* Tradeoff: A resolved target participates in the invocation identity; blocked identities use an explicit status-qualified authored symbol. H2 adds no instance, slot fragment/outlet, runtime relation, or diagnostic projection. Canonical invocation entities remain absent from semantic graph v5 and ASM inspection v6 until H18.

* Decision: H1 accepts only a zero-argument, non-static, declaration-only definite-assignment component field of exact built-in type `SlotContent`. The field name is the slot name; `children` is the default slot and every other valid name is named. Every authored candidate receives a source-qualified `SlotDeclarationCandidateId`, while only a candidate with no violation receives a component-qualified `SlotId`.
* Reason: This keeps valid semantic identity separate from retained invalid syntax and supplies H19 with exact arity, declaration-kind, type, static, initializer, definite-assignment, duplicate, and conflicting-decorator facts without reparsing source or fabricating identities.
* Tradeoff: `SlotContent` is nominal, client-boundary, exact-assignable, and nonserializable. H1 adds generic in-memory ASM lookup and ownership only. The frozen semantic graph v5 and ASM inspection v6 omit Slot entities until the roadmap-owned H18 projection; invocation, outlet, content binding, runtime, and diagnostics remain absent.

* Decision: G20 freezes one compile-time Context-designator resolver shared by G2 Provider and G3 Consumer lowering. G4 remains the only Provider visibility/selection authority; every later product only consumes its retained result.
* Reason: Removing the duplicated raw path/import matching prevents Provider and Consumer identity resolution from drifting while preserving all existing local and imported designator behavior.
* Tradeoff: The shared resolver establishes only `ContextId`. It does not select a Provider, add scope edges, interpret runtime names, or broaden the language.

* Decision: declaration-expression typing is limited to canonical State, valid Context, and valid Provider owners. Retained invalid Provider candidates may keep authored/expression evidence, but receive no type assignment whose origin pretends that an executable Provider entity exists.
* Reason: This makes deliberately invalid G18 fixtures valid immutable compiler products and removes `PSASM1105` without accepting fabricated invalid semantic origins.
* Tradeoff: No valid Provider typing changes. Invalid candidates remain excluded from G4-G17, IR, runtime artifacts, and resume state.

* Decision: Context diagnostic secondary type labels validate against the exact canonical Provider or Context declared-type provenance already used by G18 projection. Check JSON now names its frozen v3 constant rather than embedding an unowned literal.
* Reason: Canonical `PSC1059`–`PSC1064` evidence must pass shared ASM validation, while mutated/noncanonical provenance must continue to fail. Naming the version makes the serialized freeze explicit without changing output.
* Tradeoff: No diagnostic code, identity, message, label, suppression, ordering, or serialized shape changes.

* Decision: G19 is fixture expansion only. It adds no semantic product, language form, Provider selection, type/lifetime rule, lowering behavior, artifact field, runtime operation, or schema version.
* Reason: The matrix now proves the frozen G1-G18 products from authored declarations through real-browser execution, including compiler-generated `ContextValueSlotId` bindings, exact action-batch updates, blocked-source exclusion, stable resume identities, and byte-deterministic public outputs.
* Tradeoff: Deliberately invalid/diagnostic fixtures expose existing ASM-validation interactions for G20 to audit, specifically type-evidence secondary provenance and unresolved expression origins on invalid Provider candidates. G19 records coverage but does not repair or redefine those contracts.

* Decision: G18 uses retained declaration candidates and canonical G4/G5/G8/G9/G10 records as its only diagnostic sources. The shared `ComponentDiagnostic` is extended with optional typed candidate, Context, Provider, and Consumer identities; no Context-specific diagnostic envelope exists.
* Reason: Complete canonical identities, narrow source-specific provenance, deterministic secondary evidence, and validation can therefore cross every compiler inspection surface without source rewalk, missing-product inference, Provider reselection, dependency reconstruction, or runtime logic.
* Tradeoff: Check JSON advances to v3 and ASM inspection to v6 because their structured shape changed. Other public schemas and all Context selection, typing, lifetime, planning, lowering, artifact, and runtime contracts remain unchanged.

* Decision: G1 Context syntax is a zero-argument `@context()` decorated non-static component field with an explicit declared type. It lowers to a distinct component-qualified `ContextId` and remains separate from the authored field identity.
* Reason: One exact field form makes name, ownership, declared type, provenance, and later Provider/Consumer targets compiler-owned facts without accepting runtime constructors, decorator naming, or implicit globals.
* Tradeoff: Only literal defaults are retained as Context-owned shared expression roots. They are not state, providers, runtime storage, or reactive edges; invalid/missing-type/argument/static/nonliteral forms create no valid Context entity and no new user-facing diagnostic catalog.

* Decision needed before G2: define the authored Provider declaration syntax, its Context target reference, provided-value expression contract, and full declaration provenance anchor.
* Reason: The G2 roadmap identifies required Provider products but supplies no source construct that creates one. Choosing a decorator, JSX wrapper, component field, method, implicit default, or name-based lookup would invent language semantics and determine the Context-to-Provider relationship, expression ownership, and later Consumer resolution behavior.
* Tradeoff: G1 remains complete and committed. No Provider entity, Context reference, value expression, ownership inference, consumer resolution, execution, runtime artifact, or runtime lookup has been added.

* Decision: G2 Provider syntax is a non-static `@provide(ComponentSymbol.contextField)` field with one exact static designator, an explicit type, and one initializer in the existing canonical expression subset.
* Reason: Resolving the symbol/designator through local component facts and the existing import binding table yields an immutable Provider-to-Context relation without evaluating decorator arguments, using strings, reflecting runtime classes, or searching a component tree.
* Tradeoff: Provider values are expression roots only. They create no State/Computed entities, reactive edges, visibility facts, runtime slots, or execution behavior; unsupported forms create no valid Provider, while same-component duplicate targets retain one deterministic duplicate declaration fact.

* Decision needed before G3: define the authored Consumer declaration syntax, its canonical Context designator/reference form, requested-type contract, owning scope, and provenance anchor.
* Reason: The G3 roadmap identifies Consumer products but supplies no source construct that creates a Consumer. Choosing a decorator, field helper, parameter form, template read, method call, or implicit Context name would invent language semantics and predetermine later resolution/type behavior.
* Tradeoff: G2 remains complete and committed. No Consumer entity, Context read, Provider selection, visibility analysis, runtime binding, or runtime lookup has been added.

* Decision: G3 Consumer syntax is a non-static, declaration-only definite-assignment `@consume(ComponentSymbol.contextField)` field with one exact static designator and an explicit requested type.
* Reason: The compiler resolves the designator only to an immutable canonical Context identity through local component facts or the existing import binding table. Consumer identity remains component-qualified by the local binding field, while Context ownership remains unchanged.
* Tradeoff: Consumers record only `Resolved(ContextId)` or `Unresolved` Context identity state. G3 emits no Provider relation, visibility/nearest-provider analysis, default fallback selection, type compatibility result, runtime slot, scheduling product, IR, or runtime lookup.

* Decision needed before G4: define the canonical component-composition/ancestry product that establishes Provider visibility; the deterministic selection rule when several visible Providers target one Context; whether same-component Providers participate; and whether, when, and how a Context default becomes an explicit fallback result.
* Reason: The G4 roadmap says only to resolve every Consumer to exactly one Provider or retain an unresolved fact. The existing compiler has component/module ownership but no canonical component-composition ancestry or Provider visibility semantics. Selecting globally, by source order, import order, lexical nesting, or runtime-tree traversal would invent language semantics and violate the compiler-only authority invariant.
* Tradeoff: G3 remains complete and committed. No Consumer-to-Provider relation, nearest-provider result, Context default fallback, ownership-graph reconstruction, runtime binding, or runtime lookup has been added.

* Decision: G4 uses one compiler-owned `ComponentScopeGraph` as the sole Provider-visibility input. Phase G populates it reflexively only; future compiler-owned composition lowering may add validated parent edges without changing the nearest-scope algorithm.
* Reason: This gives Consumers deterministic binding results without inferring ancestry from imports, source order, lexical nesting, templates, or runtime parent traversal. Explicit Providers take precedence by nearest scope; canonical Context defaults are distinct fallback results rather than hidden Providers.
* Tradeoff: Without a canonical composition edge, cross-component Providers are intentionally invisible. G4 records immutable resolution facts and `resolves-to-provider` edges only; it adds no type compatibility, runtime slot, scheduling, IR, execution, or runtime discovery behavior.

* Decision needed before G5: define Provider value-type inference/evaluation scope; the directed compatibility rules among Context declared type, Provider declared and value types, Consumer requested type, and Context default; serialization and execution-boundary compatibility; canonical compatibility-result identity/status; and how unresolved, ambiguous, invalid, and default resolutions participate.
* Reason: The roadmap lists the three type participants but does not specify assignability direction, structural/nominal behavior, unknown/alias handling, whether a Provider declaration type, value type, or both govern compatibility, or which existing semantic type facts become authoritative. Selecting those rules would invent language semantics and prematurely determine G18 diagnostics and later runtime eligibility.
* Tradeoff: G4 remains complete and committed. No Provider/Consumer compatibility, inferred Provider value type, default compatibility, boundary/serialization result, Context type diagnostic, or runtime type behavior has been added.

* Decision: A Context declared type is the canonical channel contract. G5 projects the existing type model into immutable records and evaluates Provider values through `value -> Provider declaration -> Context declaration -> Consumer request`; Context defaults independently flow to the Context declaration. Unknown type facts are conservative, and G4 resolution is never reselected.
* Reason: Existing compiler type, serialization, and boundary products already provide the canonical facts necessary for a complete directed compatibility result. Retaining every intermediate relation lets later diagnostics and runtime lowering consume one compiler-owned answer without evaluating source again or querying Providers at runtime.
* Tradeoff: G5 records compatibility only. It adds no new type language, diagnostic catalog, Provider visibility rule, source reconstruction, dependency graph, schedule, IR, artifact, runtime slot, or execution behavior.

* Decision needed before G6: define the canonical ownership-graph product: its node and edge domains/directions; whether and how component ancestry relates to the existing `ComponentScopeGraph`; which Context/Provider/Consumer/default/unresolved/ambiguous facts appear; graph invariants, queries, and export/schema requirements; and whether this is a projection or a new authoritative composition product.
* Reason: The G6 roadmap names owners and ancestry but does not define a graph contract. The compiler already has entity ownership and a G4 visibility scope graph, but choosing an ownership topology or treating scope edges as ownership would invent the semantics that G7 dependency projection, G8 lifetime analysis, and G9 ordering must consume.
* Tradeoff: G5 remains complete and committed. No G6 ownership graph, inferred ancestry, Provider visibility change, dependency graph, runtime traversal, runtime ownership lookup, or component-tree reconstruction has been added.

* Decision: G6 is one derived `ContextOwnershipGraph`, distinct from `ComponentScopeGraph` and `ContextResolution`. Its only edges are Component-to-Context, Component-to-Provider, Component-to-Consumer, and Context-to-default-expression; typed IDs and retained inverse indexes make all reads compiler-owned and deterministic.
* Reason: Entity owners and default expression roots already establish canonical semantic ownership. Projecting them once preserves a lifetime-analysis input without turning ownership into composition, visibility, binding selection, or data dependency authority.
* Tradeoff: Context ownership is independent of G4 resolution and G5 compatibility. G6 neither copies scope topology nor adds Provider-to-Context, Consumer-to-Context/Provider, component-to-component, Provider-value-expression, or runtime edges; it makes no public inspection schema change.

* Decision needed before G7: define the canonical Context dependency-graph product: its typed node and edge domains/directions; whether Provider value and Context-default expression references are direct-only or transitive; how explicit Provider/default/unresolved/ambiguous/invalid Consumer resolutions contribute; handling of unknown or incompatible G5 bindings; graph ordering, invariants, queries, validation, and export/schema requirements.
* Reason: G7 says to project Provider/Consumer relations and ownership but does not define data-dependency topology. Treating semantic request/provide/ownership relations, expression nodes, or G4 candidate evidence as dependencies would invent the facts needed by G8 lifetime and G9 evaluation planning.
* Tradeoff: G6 remains complete and committed. No Context dependency graph, expression dependency closure, scheduling, Provider reselection, runtime graph reconstruction, or runtime behavior has been added.

* Decision: G7 projects only direct Context value-flow topology. Provider/default sources supply Context contracts, Provider expressions read canonical State/Computed nodes, and Consumers depend only on the exact G4-selected Provider/default source; G5 compatibility annotates these facts but never removes them.
* Reason: The canonical expression graph, G4 resolution, and G5 records already establish all direct facts needed by later lifetime/evaluation analysis. Retained indexes answer dependency and reverse-dependency queries without source rewalk, scope traversal, Provider lookup, or runtime discovery.
* Tradeoff: G7 does not merge with the existing reactive graph or create ownership/ancestry/request/candidate edges. It computes no transitive closure, cycle analysis, lifetime result, ordering, schedule, IR, runtime artifact, or execution behavior.

* Decision needed before G8: define Context lifetime domains and identities; the compatibility relation among Context, Provider, Consumer, default, State, and Computed lifetimes; how ownership, scope ancestry, G4 selection, G5 status, and G7 direct dependencies contribute; treatment of unresolved/ambiguous/invalid bindings; canonical result/status records, validation, queries, and export/schema requirements.
* Reason: G8 names component, Provider, and Consumer lifetime examples but does not define what a lifetime means or the analysis result. Assigning lexical, scope, component-instance, provider-selection, runtime-slot, or dependency-derived lifetimes would invent semantics that determine G9 initialization availability and later runtime behavior.
* Tradeoff: G7 remains complete and committed. No lifetime facts, compatibility diagnostics, selection changes, ordering, scheduling, IR, runtime lifetime tracking, or runtime discovery has been added.

* Decision: G8 lifetime is one `ComponentScopeLifetime(ComponentId)` domain. Canonical G6 ownership determines each entity’s lifetime identity, while exact G4 ancestor chains determine outlives compatibility for direct G7 dependencies and selected bindings.
* Reason: This preserves the compiler-only separation: scope topology, ownership, resolution, and direct value flow each remain authoritative in their existing products, while G8 records only availability compatibility and aggregate source status.
* Tradeoff: G8 never changes G4 selection or filters on G5 typing. It adds no lexical/module/runtime-instance lifetime, ancestry inference, ordering, scheduling, IR, artifact, execution, or runtime discovery behavior.

* Decision needed before G9: define the canonical evaluation-plan identity and entry records; deterministic initialization/availability ordering rules; relation to ownership and component-scope ordering; treatment of Provider/default sources, unresolved/ambiguous/invalid resolutions, G5 incompatible/unknown bindings, and G8 incompatible/unknown lifetimes; plan validation, queries, inspection/schema, and whether a plan contains only eligible values or explicit blocked entries.
* Reason: G9 names Provider initialization order, Consumer availability, and ownership ordering but does not define the compiler plan semantics or its failure representation. Choosing source order, component order, scope order, dependency order, eligibility filtering, or blocked-entry behavior would invent language semantics required by G10 lowering and later runtime artifacts.
* Tradeoff: G8 remains complete and committed. No evaluation/availability plan, ordering, scheduling, Provider reselection, IR, runtime record, or execution behavior has been added.

* Decision: G9 retains one immutable initial `ContextEvaluationPlan`. Every canonical Provider/default source and every Consumer receive an entry; executable demand requires the exact G4 selection plus G5 and G8 compatibility. The Phase D scheduler orders only planned sources by canonical scope depth and stable typed source identity.
* Reason: The plan composes existing compiler products without changing their authority: G4 selection is never retried, G5/G8 remain eligibility authorities, G7 remains the direct-dependency authority, and Phase E computed plans are reused as prerequisite metadata.
* Tradeoff: G9 is initial availability only. Unused and blocked sources remain plan facts but have no batch; the slice creates no IR, runtime storage, execution, update propagation, public inspection schema, or runtime Provider discovery.

* Decision needed before G10: define the canonical Context IR product: function, block, operation, value, and storage/load identities; how G9 source and batch identities map into it; whether Provider/default expressions reuse or extend Phase E expression lowering; Consumer-load operands and result semantics; treatment of unavailable entries; computed prerequisite ordering; validation and inspection/schema expectations.
* Reason: The G10 roadmap only says to lower Context initialization, Provider values, and Consumer loads. Selecting SSA versus Context storage semantics, source function boundaries, load targets, initialization effects, value ownership, or unavailable-entry representation would invent the compiler architecture and determine later G11 optimization and G12 runtime artifacts.
* Tradeoff: G9 remains complete and committed. No Context IR, storage/load operation, lowering, optimizer integration, runtime artifact, execution, or runtime Context lookup has been added.

* Decision: G10 uses a distinct compiler-only `ContextValueSlotId` per G9-planned source, a distinct generated `ContextSourceFunctionId`, and a distinct `ContextConsumerLoadId` per available Consumer. The source function produces its result and performs observable `InitializeContextSlot`; Consumer bindings retain typed `LoadContextSlot` records without generating a Consumer function.
* Reason: G4/G9 already supply one exact selected source, and the shared IR supplies function/value/instruction structure. Retaining slot/load identities in an immutable report lets G11 optimize source functions without removing initialization and lets G12 consume only compiler-generated identities.
* Tradeoff: Context slots are not `IrStorage` or runtime allocations. Blocked/unused sources and unavailable Consumers receive no partial IR, and G10 adds no optimizer invocation, runtime registry/artifact, execution, fallback, or lookup behavior.

* Decision: G11 optimizes only the G10-generated Context source functions through an immutable projection, then merges those functions into a clone of the original IR by semantic function identity. `OptimizedContextIrReport` retains the unmodified `ContextIrReport`, one ordered source-evaluation projection per G10 source, and the existing optimizer pass metrics.
* Reason: The existing pipeline can simplify pure producer values without allowing Context semantics to drift. `InitializeContextSlot` is an observable root, so each frozen slot/result pair remains exactly once; G9 batches, G10 Consumer loads, and G4 selection are retained instead of being recomputed or rebound.
* Tradeoff: G11 does not optimize authored methods, Computed functions, effects, or any other module product. It introduces no Context runtime artifact, evaluator, update plan, slot aliases, fallback, inspection schema change, Provider lookup, or runtime graph reconstruction.

* Decision: G12 introduces `RuntimeContextRegistry` with schema contract version 1 as a deterministic projection of G9 planned sources, G11 optimized source functions, and available G10 Consumer bindings. Source records are ordered by canonical source identity; Consumer records by `ConsumerId`; batches retain G9 order.
* Reason: The runtime needs exact slot/function/batch/type metadata, but no semantic authority. Projecting only existing compiler products retains G4 selection, G5 typing, G8 lifetime eligibility, and G9 scheduling without creating a Provider search key, Context name key, ancestry chain, or reverse dependency table.
* Tradeoff: G12 records metadata only. It does not serialize an artifact, allocate or evaluate slots, perform cold boot/update ordering, emit diagnostics/inspection, alias slots, or permit runtime lookup/reconstruction.

* Decision: G13 emits a distinct `context.runtime.json` artifact at schema version 1 and embeds the same serialized artifact under `presolve-context-runtime`. It reuses existing operand instruction encoding and adds only `initialize_context_slot` and `load_context_slot` operations.
* Reason: The emitted artifact carries compiler-generated source, function, slot, batch, type, and Consumer-load identities directly into the generated page, so G14 can execute a closed plan without Context-name matching, Provider searches, or graph rebuilding.
* Tradeoff: G13 serializes programs but does not execute them. It does not unify Context with computed/effect schemas, add lookup instructions, or create any fallback/rebinding behavior.

* Decision: Effects are first-class ASM entities keyed as `component/effect:name`, owned directly by their component and linked to the authored method ID.
* Reason: Effects are reactive consumers in their own right, so ownership, provenance, identity, graph export, and generic inspection must use the existing canonical semantic infrastructure rather than method-decorator lookups or runtime callbacks.
* Tradeoff: F1 only establishes entity metadata. It deliberately creates no effect body lowering, references, types, reactive edges, scheduler entries, IR, optimization, runtime records, or diagnostics; F2 through F18 own those products.

* Decision: Every F1 effect is client-bound with one compiler policy, `AfterInitialRenderAndCompletedActionBatch`.
* Reason: Initial execution and post-batch execution are language timing semantics, not an opt-in runtime convention. Storing the complete policy on the entity prevents a later runtime layer from choosing timing dynamically.
* Tradeoff: F1 does not yet determine whether a particular completed batch triggers an effect or execute it. F6--F9 will derive trigger and scheduler placement from canonical dependency products.

* Decision needed before F2: define the canonical supported effect-statement vocabulary and how its expression operands enter the existing expression graph.
* Resolution: The supplied F2 contract defines a flat, ordered body vocabulary: static-member assignments, direct static calls, final bare returns, and structured unsupported forms. All statement operands use the existing expression graph; statement IDs remain distinct and scoped to their owning effect.
* Tradeoff: F2 deliberately retains unresolved identifiers, `this` reads, member paths, cleanup-return candidates, and prohibited forms without classifying them. F3 owns references, F4/F5 own typing/validation, and F6 onward own reactive and runtime products.

* Decision: Effect-body expression nodes use the effect ID as their existing expression-graph owner, while statements use a separate `effect/statement:<index>` identity domain.
* Reason: Values retain one canonical expression topology, but statement ordering and side-effect operations need independently stable identities for later diagnostics and IR lowering.
* Tradeoff: Effect statement records are compiler-owned body nodes rather than generic ASM semantic entities. F2 exposes them through the owning effect body only; F17 owns dedicated inspection output.

* Decision: F3 emits one deduplicated `EffectState` or `EffectComputed` reference per resolved effect-to-target pair, retaining the first operand provenance.
* Reason: Effect ownership remains the reactive consumer identity while individual expression spans still support later diagnostics.
* Tradeoff: F3 resolves only direct `this.<name>` reads. It does not classify call targets, validate assignments, infer types, or build reactive edges.

* Decision: F4 consumes registry version 1 as the sole authority for external effect operations.
* Reason: Stable capability and operation IDs, exact static paths, signatures, client boundaries, argument serialization, and runtime-lowering identities must originate in one immutable compiler product. `EffectStatementTypeRecord` projects its facts by statement identity without browser-global inspection or duplicated recognition in diagnostics/IR/runtime layers.
* Tradeoff: The initial registry is deliberately limited to `document.title`, console logging, and local/session storage writes. Unknown paths are never `any`; F4 records their typed operands plus incompatible/unknown compatibility evidence, while F5 owns semantic rejection. No capability extensions, reactive edges, scheduler placement, IR, runtime metadata, or execution are added here.

* Decision: F5 stores effect legality as canonical `EffectValidation` and ordered `EffectSemanticViolation` facts on the first-class effect entity, while deferring user-facing diagnostic codes to F18.
* Reason: F5 can reject invalid entities for all future graph, scheduler, IR, and runtime consumers without duplicating validation or prematurely freezing the Phase F diagnostic catalog. It reuses F4's statement records and existing method metadata rather than rereading source or invoking runtime behavior.
* Tradeoff: F5 produces no new `ComponentDiagnostic` entries yet. F18 will project these immutable violation facts into prescriptive, source-provenanced diagnostics; F6 must consume only valid effects when integrating the reactive graph.

* Decision: `ComponentDiagnostic` is the sole shared compiler diagnostic envelope and now supports severity, optional typed `EffectId`/`EffectStatementId`, and deterministic secondary source labels.
* Reason: Effect diagnostics are component/compiler diagnostics. A parallel effect envelope would duplicate canonical ordering, serialization, CLI rendering, ASM projection, and future IDE integration.
* Tradeoff: This is an explicit serialized diagnostic-shape evolution. Check JSON advances from v1 to v2 and ASM inspection advances from v3 to v4; legacy diagnostics remain valid with no effect/statement identity and empty label collections.

* Decision: F6 represents valid effects as terminal `Effect` nodes in the existing reactive graph, with `Reads` to direct state/computed inputs and inverse `Invalidates` edges from those inputs.
* Reason: Effects need compiler-owned dependency topology for later triggers and scheduling, but may never become reactive producers. Reusing the graph's established read/invalidation direction lets F7 derive closures without a second effect graph or runtime discovery.
* Tradeoff: Invalid effects are omitted, and existing cycle/scheduler passes intentionally remain computed-only. F6 creates no trigger classification, scheduler placement, IR, runtime metadata, or execution behavior.

* Decision: F7 projects existing transitive graph analysis into `EffectReactiveAnalysis` records keyed by valid effect semantic ID.
* Reason: Trigger and scheduler slices need stable semantic identities for complete dependencies without traversing graph strings or source expressions. The projection preserves the terminal invariant: effects can have dependencies but no dependents.
* Tradeoff: F7 adds no effect-to-action trigger mapping, ordering, scheduler batches, IR, or runtime behavior.

* Decision: F8 keys trigger eligibility by authored action-method `ActionBatch` identity, never individual write identity.
* Reason: A batch retains ordered write records and a deduplicated state set, then emits at most one trigger relation per batch/effect with stable matching-state evidence. Initial triggers are a separate explicit valid-effect list.
* Tradeoff: F8 proves eligibility only. It does not infer runtime value equality, schedule effects, lower IR, or emit runtime metadata.

* Decision: F9 uses minimal dependency-complete computed prerequisites. Initial schedules include every executable direct/transitive computed dependency of F8-initial effects; action schedules intersect those dependencies with computed values invalidated by that action batch.
* Reason: F8 remains the sole eligibility authority and E9 remains the sole computed-ordering authority. F9 filters existing E9 batches, records their source batch indexes, and invokes the Phase D scheduler only for terminal effect batches, so unrelated computed work is neither removed from the global plan nor duplicated for effects.
* Tradeoff: Effects with unavailable computed prerequisites are explicitly unplanned rather than observing stale values. F9 adds no IR, runtime registry/artifact, execution, value-equality check, or runtime dependency discovery; F10 owns effect IR lowering.

* Decision: F10 represents each lowered effect with one `IrEffectExecution` and one separate effect-ID-keyed `IrFunction`; recognized operations lower only to generic `CapabilityCall` or `CapabilityAssign` instructions carrying the canonical F4 operation ID.
* Reason: This reuses the existing IR module/function/block/value identity domains and shared operand lowering while preserving F4 registry IDs as the extensibility boundary. Capability instruction enum variants make observability explicit to F11 without built-in-specific IR or raw source-path matching.
* Tradeoff: Effects retain one flat entry block and normal completion with no semantic result. F10 does not duplicate F9 schedules, optimize operands, emit registry/artifact data, execute capabilities, add cleanup/async behavior, or perform runtime dependency discovery; F11 owns immutable optimization.

* Decision: F11 reuses `computed_optimization_pipeline` unchanged by projecting only F10 effect functions into its input and merging those optimized functions back into the complete IR by semantic ID.
* Reason: Effect optimization stays under the existing immutable optimizer and pass ordering, while neither authored-method nor computed IR is accidentally changed by an effect-only slice. Existing capability instruction operands participate in canonical use/liveness tracking; their observable void forms are not candidates for elimination or common-subexpression rewriting.
* Tradeoff: The optimization report's pass metrics describe the effect-only projection, while its output preserves the full IR. F11 does not alter F9 schedules, add runtime records/artifacts, execute capabilities, or introduce runtime dependency discovery; F12 owns runtime effect registry metadata.

* Decision: F12 keys runtime effect records exclusively by canonical `IrEffectExecution` membership, then projects F8 trigger evidence and F9 scheduled prerequisite data into each effect record.
* Reason: F10 lowering is the authoritative executable-membership boundary. Intersecting existing F7 computed dependencies with F9's already-selected scheduled prerequisites yields record-local prerequisite metadata without rereading bodies, rebuilding dependency closures, or assigning scheduler positions.
* Tradeoff: An unplanned or invalid effect has no runtime record even if earlier semantic products mention it. F12 is metadata only: it does not produce a serialized artifact, interpret IR, dispatch capabilities, run initial or action-batch effects, compare values, or introduce runtime dependency discovery; F13 owns the artifact.

* Decision: F13 emits schema-v1 effect programs by reusing the existing computed-artifact encoding for pure IR instructions and adding only explicit F10 capability instruction roots.
* Reason: A shared operand/value instruction encoding keeps F10/F11 IR lowering canonical across computed and effect artifacts. Capability operation IDs resolve to runtime-lowering IDs only through the immutable F4 registry, so no emitted field or runtime path matcher depends on authored static member paths.
* Tradeoff: The artifact carries canonical IDs and executable programs, not source provenance or raw paths. F13 emits no page integration or execution behavior, does not recompute F8/F9 products, and does not perform runtime discovery; F14 owns one-time initial execution.

* Decision: F14 groups initial effects only by their compiler-emitted F9 batch index, preserving artifact order within a batch, and dispatches capability operations solely by their compiler-emitted runtime-lowering ID.
* Reason: F13 already records initial trigger membership and batch position, so grouping consumes explicit compiler products rather than observing dependencies or reconstructing eligibility. Sharing the established pure-IR evaluator ensures effect operands see the completed compiler-generated computed initialization before external synchronization.
* Tradeoff: Debug evidence records effect and capability IDs, never runtime dependency observations or arbitrary local state. F14 intentionally does not consult action trigger records after an action; F15 owns completed-action batching, computed-flush composition, and exactly-once action-triggered execution.

* Decision: F16 represents initial effect resumability with a separate component-qualified `EffectActivationSlotId` and explicit `Pending`/`Completed`/`Failed` status, projected only from F1 effect facts, F9 plan membership, and F12 runtime records.
* Reason: Effect identity, executable-function identity, action-batch identity, and mutable initial-activation lifecycle are different compiler domains. A stable activation slot lets a future runtime restore an explicit lifecycle state without inspecting external capability targets, DOM, values, dependencies, or action history.
* Tradeoff: F16 advances the serialized resume manifest to schema v2 and preserves canonical completed-action batch references, but it neither persists nor restores live browser state, mutates slots at runtime, replays/suppresses effects, captures capability state, changes inspection/diagnostics, or introduces runtime dependency discovery.

* Decision: F17 defines one core `EffectInspection` projection and advances both full and selected ASM inspection documents to schema v3.
* Reason: Validation, topology, trigger, schedule, IR, runtime, and resumability facts must be observed from their existing compiler-owned products, not reconstructed separately by CLI or explain rendering. A single serializable projection keeps full ASM, selected ASM, and explain byte-consistent.
* Tradeoff: F17 exposes stable internal violation categories rather than F18 user-facing codes and omits no semantic state by inventing runtime membership. It does not alter validation, schedule, artifacts, runtime behavior, restoration, dependency discovery, or inspection of raw source/capability paths.

* Decision: F15 advances the template manifest to schema v2 and uses each compiler-emitted template action binding as the canonical event-to-F8-batch bridge: `method_id` identifies the implementation and `action_batch_id` identifies the completed action batch.
* Reason: The template manifest is already the compiler-owned browser event contract. Lowering resolves IDs only through the existing F8 action-batch map, so neither the runtime nor a later phase parses names, observes writes, or rebuilds eligibility/dependencies.
* Tradeoff: A v2 manifest rejects missing or mismatched action IDs. A legacy v1 manifest remains readable for legacy action execution, but is rejected when paired with an F13 effect artifact containing completed-action plans; rebuilding is required. F15 does not add runtime dependency discovery, value equality checks, initial-plan replay, or scheduler reconstruction.

* Decision needed before F8: define the canonical identity for a completed action batch when one authored action method lowers to multiple `ComponentAction` state-write records.
* Reason: Effects run once per completed action batch, but existing component actions are individual state operations. F8 must either map effects to method/batch identity and deduplicate changed dependencies there, or map effects to individual writes and require F9/runtime layers to reconstruct the batch. The choice determines trigger metadata, F9 ordering, F12 registry identity, and F15 batching behavior.
* Tradeoff: F7 remains complete and committed. No trigger mapping, scheduling, runtime metadata, or execution has been introduced; await a contract for batch/method identity, multi-write deduplication, and the representation of actions that change no effect dependency.

* Decision: ASM validation resolves typed subjects through either canonical semantic entities or canonical expression-graph nodes, using the subject's authoritative provenance in either case.
* Reason: Expression nodes are first-class compiler products with stable IDs and spans, but are intentionally not modeled as generic semantic entities. Validation must preserve that separation while accepting their canonical type assignments.
* Tradeoff: E21 does not add expression nodes to ownership navigation or inspection entity lists. Their type/provenance contracts remain available through the expression graph, preserving the existing semantic-entity inspection surface.

* Decision: Computed cycle and semantic diagnostics now assemble through one shared helper in both ASM construction modes, and ASM inspection schema v2 is represented by one CLI constant.
* Reason: The canonical build paths and inspection documents cannot drift in ordering or version by maintaining duplicate assembly logic.
* Tradeoff: Inspection remains frozen at v2. Any incompatible record change requires an explicit schema evolution rather than an implicit CLI variation.

* Decision: E20 adds narrow source fixtures for every roadmap scenario and reuses the existing browser harness only for behaviors that execute at runtime.
* Reason: Compiler-only contracts (cycles, folding, serialization, and multi-file identity) stay deterministic and cheap to diagnose, while chain, batching, and diamond cache refreshes are proven in a real browser against compiler-generated artifacts.
* Tradeoff: Fixture coverage validates existing Phase E products; it does not add new language semantics, make computed template bindings dynamically refresh, or introduce runtime dependency discovery. E21 owns the final stability audit and contract freeze.

* Decision: `ComputedDiagnosticCode` centralizes the stable `PSC1034`--`PSC1040` catalog, while E19 projects invalid declarations, unsupported bodies, unresolved reads, type compatibility, and serialization from existing canonical ASM products.
* Reason: Diagnostics share the same computed entities, expression graph, purity facts, reactive cycle analysis, and semantic type model used by the rest of the compiler; no runtime observation or CLI-specific analysis determines an error.
* Tradeoff: E19 reports only behavior already represented by the current parser and semantic products. It does not widen E2 getter-body support, add runtime validation, or create the comprehensive fixture matrix; E20 owns that fixture work.

* Decision: ASM inspection schema v2 adds a computed-only record that projects existing E4/E5/E7/E9/E10 compiler products without reconstructing source facts.
* Reason: Inspection consumers receive deterministic computed type, transitive topology, schedule placement, purity, boundary compatibility, and IR identity from the same canonical products used by later compiler stages.
* Tradeoff: E18 reports zero-based schedule positions and `null` for values with no E9 placement or E10 function. It does not add diagnostics, infer metadata in the CLI, execute computed values, add runtime discovery, or expand fixtures.

* Decision: Resume plans include only E12 registry records that are structurally serializable under E4, with stable cache-slot and dirty-flag metadata rather than speculative cache payloads.
* Reason: Resume and serialization consumers can identify exactly which compiler-lowered caches may cross a resume boundary without treating cyclic, impure, unresolved, unlowered, or non-serializable values as partially resumable.
* Tradeoff: E17 plans cache capture and restoration but does not persist live values or restore them in the browser runtime. Existing state instance serialization remains unchanged; a runtime snapshot transport/restore protocol requires its own contract.

* Decision: E16 emits a schema-v3 storage-to-transitive-computed invalidation table and consumes the existing E9 update batches only after an action completes.
* Reason: State writes can mark every compiler-known dependent dirty and execute one scheduler-directed flush without inspecting runtime getter source or rebuilding reverse dependency edges.
* Tradeoff: State bindings still update during individual action steps, but computed values remain cached until the action loop ends. The runtime does not update template output from computed caches, serialize them, or coalesce independent browser events; E17 owns serialization planning.

* Decision: E15 evolves `computed.runtime.json` to schema v2, adding canonical state-storage initialization and E10-derived instruction programs to each emitted evaluation.
* Reason: The runtime can execute cached values from compiler-owned operations, storage IDs, and E9 order without parsing getter source or discovering dependencies dynamically.
* Tradeoff: The runtime executes the plan only once at boot and exposes cache values for inspection. It does not update template bindings from computed caches, mark dirty on a state write, re-evaluate values, or consume the emitted update batches; E16 owns invalidation and batching.

* Decision: `computed.runtime.json` is a separate schema-versioned build artifact generated from the E12 runtime registry and E9 evaluation plan.
* Reason: Runtime consumers receive stable cache, dirty, dependency, evaluation-function, serialization, order, and batching metadata without parsing source or discovering dependencies dynamically.
* Tradeoff: E14 serializes metadata only and filters the existing plan to registry entries with lowered evaluation functions. The current runtime neither loads the file nor executes a computed function, mutates a cache, or invalidates a dependency; E15 owns execution.

* Decision: Direct template `this.<name>` uses resolve to a distinct `template-computed` reference only when the owning component has a matching first-class computed entity and no same-named state field.
* Reason: Template consumers receive a stable declaration-to-use relation and canonical computed type without reparsing source or conflating computed reads with state dependencies; existing state resolution retains its established precedence.
* Tradeoff: E13 resolves only exact direct member forms in existing binding, dynamic-attribute, conditional, and list positions. Member access, calls, local composition, static HTML evaluation, generated runtime artifacts, cache execution, and invalidation behavior remain outside this slice.

* Decision: Runtime computed metadata is a deterministic registry keyed by computed semantic ID and derived from canonical ASM references plus E10 evaluation records.
* Reason: Cache slots, dirty flags, direct dependencies, evaluation functions, serialization, and provenance are all compiler-owned before any runtime artifact or runtime discovery exists.
* Tradeoff: E12 creates records only for values that received E10 evaluation IR. It initializes every dirty flag to true but does not execute a function, mutate a cache, emit an artifact, or propagate invalidation; later runtime slices own those behaviors.

* Decision: Computed IR optimization runs through one fixed immutable pipeline and returns an `IrOptimizationReport` rather than mutating E10 canonical lowering.
* Reason: Backends and inspection consumers can compare authored lowering to the compiler-owned optimized product with stable pass metrics, while future runtime artifacts can choose the optimized result explicitly.
* Tradeoff: E11 optimizes only the current E10 IR operations. Computed evaluation result values are explicit DCE roots, so the pipeline preserves them even when their intermediates fold away; no runtime execution, caching, or metadata is added.

* Decision: A computed evaluation lowers to an IR function keyed by the first-class computed semantic ID, with a separate canonical `IrComputedEvaluation` record naming its result value.
* Reason: The computed entity—not its authored getter method—is the compiler-owned derived-value subject for scheduling and future runtime records, while the evaluation record makes the produced value inspectable without adding a terminator before the IR roadmap evolves that contract.
* Tradeoff: E10 lowers only pure computed values that E9 can plan and whose E2 expression root resolves through existing canonical references. E5/E8 diagnostics already reject impure/cyclic values; unresolved reads remain diagnostic-free until E19 and produce no evaluation IR. E10 does not optimize, execute, cache, or emit runtime metadata.

* Decision: Computed evaluation planning projects only computed-to-computed `Invalidates` edges into the existing Phase D scheduler; state remains an external invalidation trigger rather than a scheduled computed node.
* Reason: The E6 graph intentionally records both `Reads` and inverse `Invalidates` edges, so this projection preserves one canonical reactive source while giving the scheduler its dependency-to-dependent direction without introducing artificial two-edge cycles.
* Tradeoff: E9 emits order, batches, and explicitly unplanned nodes only. It does not execute getters, allocate caches, lower computed IR, emit runtime metadata, or recover a plan for cycle-blocked nodes; E10 owns canonical IR lowering.

* Decision: Computed cycles are canonical strongly connected components of direct computed `Reads` edges, with node membership and cycle ordering derived in stable semantic-ID order.
* Reason: Diagnostics and later scheduling can consume one immutable compiler-owned cycle product without treating state invalidation edges, source names, or runtime observation as cycle authority.
* Tradeoff: E8 reports one `PSC1035` diagnostic per cycle at the first member's provenance. It does not reject or alter graph construction, infer an evaluation order, create update batches, lower computed IR, or add runtime behavior; E9 owns planning.

* Decision: Transitive reactive topology is an immutable ASM analysis with deterministic dependency and dependent adjacency maps for every canonical reactive node.
* Reason: Later cycle detection, scheduler, runtime, and inspection consumers can query compiler-produced closures without traversing source expressions or rediscovering reactive paths.
* Tradeoff: E7 derives reachability only. It does not classify or diagnose cycles, choose evaluation order, create update batches, add new edge kinds, or lower runtime artifacts; E8 owns deterministic cycle diagnostics.

* Decision: The reactive graph is an immutable compiler-owned projection of canonical computed semantic references: `Reads` goes from a computed value to a direct state/computed dependency, and `Invalidates` is the reverse edge with the same relation provenance.
* Reason: Later dependency analysis and scheduler consumers can query one deterministic topology without rediscovering getter reads or relying on runtime observation.
* Tradeoff: E6 records only direct state/computed topology. It computes no transitive closures, detects no cycles, plans no updates, and adds no action/template/runtime edges; E7 owns transitive dependency and dependent analysis.

* Decision: Purity is a compiler-owned computed-value classification with ordered violation records, while `PSC1034` diagnostics project each violation using its source provenance.
* Reason: The compiler can reject behavior before any reactive or runtime product consumes the getter, and later tooling can query canonical purity facts rather than reanalyzing method bodies.
* Tradeoff: E5 detects retained direct method-call facts and existing mutation/async metadata only. It does not model arbitrary nested control flow, resource declarations, or full call-graph effects; E19 will extend the diagnostic catalog beyond this purity contract.

* Decision: The computed entity receives the inferred expression type as its canonical typed subject, while the method retains its existing declared return contract for validation.
* Reason: Runtime, reactive, and template consumers can address one derived-value type without conflating the value's inferred semantics with its authored declaration.
* Tradeoff: Declared mismatches are recorded as deterministic compatibility metadata without diagnostics until E19. Current computed entities are client-bound, so boundary compatibility is evaluated against the current client output surface; cross-boundary declarations remain future work.

* Decision: Computed read references are canonical entity-to-entity edges with the computed entity as their source, and repeated reads of one target collapse into one deterministic relation.
* Reason: Later reactive dependency and scheduler consumers need stable dependency topology rather than a source-text use list; individual read nodes and spans remain available in the canonical expression graph.
* Tradeoff: Unresolved reads produce no relation until diagnostics are introduced in E19. Reference provenance remains the computed declaration provenance to preserve the existing source-entity reference contract.

* Decision: Supported computed getter returns lower under the computed entity ID, while `this.<name>` remains an unresolved expression node and nested static access remains explicit expression topology.
* Reason: E3 can resolve state and computed reads into canonical semantic references without reparsing getter source or conflating source names with stable semantic targets.
* Tradeoff: E2 accepts only a single direct supported return expression. Locals, control flow, calls, dynamic/optional member access, mutations, types, diagnostics, and evaluation are intentionally deferred to their roadmap slices.

* Decision: A decorator-marked computed getter has a distinct ASM entity whose stable ID is derived from its component ID and getter name, while its existing method entity remains the authored execution declaration.
* Reason: Dependencies, runtime records, IR lowering, and inspection can address one compiler-owned derived value without conflating it with the method syntax that declares it.
* Tradeoff: E1 establishes identity and static policy metadata only. Getter bodies are not lowered, reads are not resolved, purity remains unclassified, and no reactive/runtime behavior is emitted.

* Decision: State initializer expressions have stable graph roots and recursively keyed canonical nodes in the ASM.
* Reason: Folding and inspection now consume the same compiler-owned topology instead of independently traversing field-local lowering structures.
* Tradeoff: B10 retains legacy field-local trees only as lowering compatibility data; every semantic consumer added in this phase reads the canonical graph. General expression owners remain later work.

* Decision: Every lowered IR function begins with one stable, empty entry basic block whose identity is derived from the canonical method ID.
* Reason: Subsequent CFG slices can name branch and loop edges against compiler-owned nodes without backend-specific or source-offset-generated block identities.
* Tradeoff: D2-A creates only entry regions; it does not lower statements, define terminators, or create normal/branch/loop edges.

* Decision: Conditional branch arms are represented as source-provenanced directed edges owned by the enclosing IR function.
* Reason: Later condition lowering and CFG analyses can use structural true/false connectivity without coupling to a backend or recovering topology from source control-flow syntax.
* Tradeoff: D2-B models edge shape only. No source branches are lowered, conditions are not represented as operands, and unconditional/loop edges remain later work.

* Decision: Natural loops are explicit function-owned regions with stable loop IDs and canonical block topology.
* Reason: Dominator, post-dominator, liveness, and scheduling consumers can reason about loop boundaries without recognizing source syntax or inferring loops from backend artifacts.
* Tradeoff: D2-C stores loop structure only. It does not derive regions from CFG edges, validate natural-loop invariants, lower source loops, or assign loop-specific instructions.

* Decision: Dominators are derived immutably from IR block and branch-edge topology rather than stored as mutable IR state.
* Reason: CFG analysis remains repeatable and backend-independent, and later optimization passes can consume deterministic analysis output without changing canonical lowering artifacts.
* Tradeoff: D2-D considers only declared conditional branch edges and entry reachability. It does not validate malformed CFGs, include loop-back/unconditional edges, or expose higher-level dominance query helpers yet.

* Decision: Post-dominators are derived immutably in reverse from blocks without declared branch successors.
* Reason: Later code motion, control-dependence, and cleanup passes gain a deterministic reverse-flow relation without requiring backend code generation or mutating canonical IR.
* Tradeoff: D2-E treats blocks without declared branch successors as exits. It does not model explicit terminators, non-terminating control flow, loop-back/unconditional edges, or post-dominator query helpers yet.

* Decision: CFG connectivity and dominance are exposed through read-only function and analysis-tree queries.
* Reason: Future data-flow and optimization consumers can navigate compiler-owned control flow without indexing public vectors directly or rebuilding predecessor/successor relationships.
* Tradeoff: D2-F exposes only current branch-edge topology. Loop-region membership, edge filtering, explicit terminators, reachability, and data-flow queries remain later work.

* Decision: Authored semantic entities, lowered instructions, transient values, and storage slots use separate typed identity domains.
* Reason: A single semantic entity may lower to several operations and values, while optimization may rewrite IR artifacts without changing authored meaning or provenance.
* Tradeoff: D2-G1 defines stable identity only. Existing instructions and state initialization retain their prior representations until later D2-G slices migrate them.

* Decision: Executable operands form a closed enum of value, inline primitive constant, and storage references.
* Reason: Data-flow can distinguish value uses from storage access while retaining constants inline, without an ambiguous catch-all semantic operand.
* Tradeoff: D2-G2 excludes function, template, aggregate, and runtime-allocated operands until concrete lowering needs them.

* Decision: An instruction identity, optional produced value, and optional authored semantic origin are independent fields on canonical IR instructions.
* Reason: Value-producing operations can now participate in data flow without conflating an operation instance, its result, and the semantic entity from which lowering originated.
* Tradeoff: D2-G3 adds operation shapes but does not lower load/store/arithmetic instructions from source, and result values are not yet registered or validated.

* Decision: Transient values are owned by the IR function in a deterministic registry and identify their defining instruction, parameter, or future block parameter explicitly.
* Reason: Definition/use, liveness, and optimization can inspect one canonical value model without inferring definitions from operation shape or conflating values with semantic entities.
* Tradeoff: D2-G4 creates empty registries during current lowering. Parameter/block-parameter lowering and consistency validation are deferred.

* Decision: State fields lower to `IrStorage` slots separate from both semantic identity and transient values.
* Reason: Storage reads, writes, promotion, resumability, and reactive partitioning can evolve without treating an authored field as a runtime slot or computed value.
* Tradeoff: D2-G5 only lowers storage declarations and initialization references; load/store source lowering and storage validation remain deferred.

* Decision: IR integrity is validated as an immutable compiler-owned query over canonical IR, before data-flow consumers run.
* Reason: Definition/use and optimization passes can reject malformed IDs, dangling operands, missing result records, and invalid storage references instead of silently producing misleading analysis.
* Tradeoff: D2-G6 validates current structural contracts only; it does not yet validate source lowering coverage, type operation compatibility, terminators, or loop invariants.

* Decision: Expression graph nodes own `SourceProvenance` rather than an unqualified source span.
* Reason: Tooling and diagnostics need the canonical expression node itself to provide a path-aware authored location without reconstructing it from a state field.
* Tradeoff: B11 derives provenance only for the existing state-initializer graph. Template, action, and general JavaScript expressions are not lowered yet.

* Decision: Expression graph queries return compiler-owned references in stable semantic-ID order and expose direct graph edges only.
* Reason: Language tooling and future optimizations can navigate one canonical graph without reparsing expression trees or inferring ownership from identifier strings.
* Tradeoff: B12 provides no graph mutation, transitive dependency closure, query language, or non-state expression support.

* Decision: `SemanticType` is a compiler-owned algebra with an initially empty ASM assignment model, separate from legacy raw declared-type metadata.
* Reason: Later inference, alias resolution, assignability, and tooling can share one canonical semantic representation without prematurely treating TypeScript text as type semantics.
* Tradeoff: C1 lowers no annotations, infers no values or expressions, and adds no type diagnostics, identities, provenance, normalization, or query APIs.

* Decision: Canonical type assignments identify the typed subject and separately retain the semantic origin, declared-or-inferred status, and source provenance.
* Reason: Aliases, imported types, and inferred expressions will need stable attribution without collapsing their type meaning into raw TypeScript text.
* Tradeoff: C2 defines metadata only; no assignment is populated from source, and identity does not yet model named aliases or cross-module declarations.

* Decision: C3 lowers only exact primitive and literal state annotation text into canonical semantic types.
* Reason: The compiler gains durable primitive and literal semantics while later slices add structured parsing for arrays, tuples, objects, unions, aliases, and imports.
* Tradeoff: Unsupported annotation text produces no canonical assignment or diagnostic yet; C3 does not infer from initial values or alter existing compatibility diagnostics.

* Decision: C4 represents tuples directly and lowers unresolved array element names as `unknown` rather than raw TypeScript names.
* Reason: Collection topology is canonical now, while alias and import slices can later replace unknown element meaning without revising the array contract.
* Tradeoff: Tuple parsing is limited to comma-separated current annotation forms; object elements, nested delimiter-aware parsing, and named-type resolution remain later work.

* Decision: C5 lowers semicolon-separated structural object properties into deterministic maps.
* Reason: Object shape is canonical before template member resolution and list-item typing need it.
* Tradeoff: Object-property declarations have no individual identity or provenance yet; nested delimiter-aware syntax and member-access validation remain later work.

* Decision: C6 preserves top-level union member order without normalizing or deduplicating.
* Reason: The compiler can represent authored unions now while C28 later defines the one canonical normalization policy.
* Tradeoff: Literal parsing inside union members is supported, but aliases, imports, and diagnostics remain later work.

* Decision: C7 gives each local type alias a module-qualified semantic ID and uses that ID as the origin of resolved state assignments.
* Reason: Alias source identity remains available to tooling while consumers receive the alias's canonical semantic type meaning.
* Tradeoff: C7 resolves supported local aliases only; nested alias dependencies, exports, imports, generic aliases, and cycles remain later work.

* Decision: C8 indexes type aliases as module symbols and reuses named relative import/re-export bindings for unit-level type resolution.
* Reason: Imported types now share deterministic module identity and re-export behavior with the compiler's existing frontend infrastructure.
* Tradeoff: C8 excludes external packages, namespace imports, default type imports, generic aliases, and imported alias cycles.

* Decision: C9 widens direct serializable state literals to primitive semantic types and infers empty arrays as `Array<unknown>`.
* Reason: State APIs receive useful stable type contracts without prematurely encoding literal-value constraints or assuming an empty collection element type.
* Tradeoff: C9 infers only direct serializable values; constant expression nodes, actions, and non-serializable values remain later work.

* Decision: C10 uses a canonical state-initializer assignability relation and preserves `PSC1016` for incompatibilities.
* Reason: Arrays, tuples, objects, unions, and nullability now receive one compiler semantic compatibility check instead of primitive-only special cases.
* Tradeoff: C10 is limited to state initializers; C29 will establish the final general assignability engine for all compiler consumers.

* Decision: Expression graph nodes receive their inferred type as ordinary canonical ASM assignments keyed by the existing expression semantic ID.
* Reason: All consumers can query the same owned, provenanced expression topology and type information without re-evaluating authored syntax.
* Tradeoff: C11 propagates types only for the current constant state-initializer expression language. C12 still defines operand validity, while state reads, locals, templates, actions, and arbitrary JavaScript expressions remain later work.

* Decision: Operator typing is one compiler-owned relation over semantic operands and returns an explicit result or invalidity.
* Reason: Expression propagation, later diagnostics, and future optimizations can share defined Presolve semantics rather than inheriting JavaScript coercion behavior.
* Tradeoff: C12 covers only the current constant-expression operators. Invalid operations become `unknown` without new diagnostics until C32; strings, truthiness, calls, and non-state expression forms remain unsupported.

* Decision: Existing method-local declaration entities receive inferred type assignments directly from their lowered serializable values.
* Reason: Local bindings, template references, and tooling now meet at one canonical typed entity without inventing a second local-variable model.
* Tradeoff: C13 covers the currently lowered serializable `const` forms. Local expressions, state reads, annotations, flow, mutation, destructuring, and action/local references require later language lowering.

* Decision: Method parameters are first-class method-owned semantic entities with declaration annotations lowered through the canonical type model.
* Reason: Action, computed, IDE, and type-query consumers can address a stable parameter entity rather than interpret method metadata ad hoc.
* Tradeoff: C14 supports currently retained identifier parameters and supported annotations only. Defaults, destructuring, rest, optionality semantics, parameter value flow, and call-site validation remain later work.

* Decision: Method semantic type assignments represent their return contract, declared when annotated and inferred from currently supported serializable returns otherwise.
* Reason: Derived values, loaders, actions, and tooling can query one canonical method result contract before their dedicated phases add richer producers.
* Tradeoff: C15 considers only top-level serializable return expressions. JSX, state/local expressions, async promises, branches, throws, implicit returns, and return compatibility diagnostics remain later work.

* Decision: Direct action assignment compatibility is evaluated by the immutable ASM folding pass using canonical type assignments.
* Reason: State initialization and mutation now share the same compiler-owned compatibility relation rather than duplicating primitive frontend checks.
* Tradeoff: C16 covers only currently lowered direct literal assignments. Compound mutation, arbitrary expressions, parameter/local values, and final general assignability remain later work.

* Decision: Compound mutation typing uses canonical boolean/number compatibility in the same immutable ASM pass as direct assignment validation.
* Reason: Toggle and arithmetic mutation contracts now share compiler-owned types rather than frontend primitive classifiers.
* Tradeoff: C17 covers only currently lowered literal operands and mutation forms; arbitrary expressions, coercion, and generalized operation typing remain later work.

* Decision: Direct template text-binding entities inherit the canonical type of their resolved state/local target and are validated during immutable ASM folding.
* Reason: Rendering, diagnostics, and later tooling can query one typed template entity rather than infer renderability from expression strings.
* Tradeoff: C18 covers direct text bindings only. Attribute/property contracts, list scope, member access, arbitrary expressions, and non-direct template references remain later work.

* Decision: Supported dynamic DOM bindings use compiler-owned contracts that distinguish HTML attributes from DOM properties.
* Reason: Template validation and later IDE/schema work share deterministic type expectations without inheriting browser coercions.
* Tradeoff: C19 covers only `disabled`, `href`, and `value` direct bindings. Element-specific contracts, event payloads, spreads, styles, and arbitrary expressions remain later work.

* Decision: Template conditions are boolean-only compiler semantics and carry the resolved condition type on their canonical entity.
* Reason: Conditional output is predictable across backends and tooling does not need to reproduce JavaScript truthiness.
* Tradeoff: C20 covers direct resolved conditions only. Composite expressions, list scope, member access, and custom condition coercions remain later work.

* Decision: Template list entities carry canonical iterable types and a dedicated item/index scope type record.
* Reason: List-body member access, rendering, and tooling can consume stable scope semantics without re-inferring callback variables from source.
* Tradeoff: C21 supports direct state-backed array/tuple lists only. Member access validation, arbitrary iterables, callback expressions, and list control flow remain later work.

* Decision: Supported list-item member paths resolve through canonical object types and retain successful or failed access records in the type model.
* Reason: Templates and tooling can query member result types or deterministic failures without rescanning expression strings.
* Tradeoff: C22 currently resolves uniquely named list-item roots and dot-member object paths only. State/local member access, unions, optionality, indexes, methods, and arbitrary expressions remain later work.

* Decision: A computed value is a decorator-marked getter with a canonical result record that reuses the method's return type assignment.
* Reason: Computed consumers can query a durable typed contract before the full dependency/runtime computed phase exists.
* Tradeoff: C23 establishes metadata and result typing only. Dependency tracking, getter evaluation, template computed reads, caching, async computed values, and runtime behavior remain later work.

* Decision: An action signature is a decorator-marked method whose existing typed parameters and return contract are assembled into one canonical action record.
* Reason: Forms, server actions, and tooling can query input/output contracts before action transport/runtime semantics are introduced.
* Tradeoff: C24 establishes signature metadata only. Promise/generic resolution, input validation, invocation, transport, server boundaries, and runtime action behavior remain later work.

* Decision: Resources are represented directly in the semantic type algebra with explicit data, error, pending, serializability, and execution-boundary metadata.
* Reason: Later resources, resumability, and backend planning can share one durable contract rather than wrap untyped runtime values.
* Tradeoff: C25 provides only type representation. Resource declaration lowering, loading/execution, error propagation, serialization enforcement, and boundary validation remain later work.

* Decision: Serialization compatibility is one recursive canonical type query, with unknown/never treated as incompatible until proven otherwise.
* Reason: Resumability and backend planning can make deterministic decisions from semantic types instead of runtime value guesses.
* Tradeoff: C26 defines compatibility only; no source declarations are rejected and no boundary-specific diagnostics or generation behavior is introduced yet.

* Decision: Cross-boundary compatibility is a canonical query layered on serialization compatibility, with resource execution boundaries enforced explicitly.
* Reason: Backend planning and resumability can reject impossible crossings before code generation without inspecting runtime values.
* Tradeoff: C27 provides query semantics only. Source boundary annotations, diagnostics, backend enforcement, and resource declaration lowering remain later work.

* Decision: Semantic types normalize before ASM consumers observe assignments, aliases, scopes, accesses, computed values, and action signatures.
* Reason: Equality, caching, inspection, and later assignability operate on one deterministic representation rather than authored union order or nesting.
* Tradeoff: C28 defines canonical representation only. General assignability remains C29, and source aliases retain their separate identities/provenance.

* Decision: One normalized `is_assignable` engine owns semantic compatibility, while the C10 state-initializer API remains a forwarding compatibility surface.
* Reason: Diagnostics, templates, actions, and future consumers no longer scatter independent type relations.
* Tradeoff: C29 centralizes existing supported forms only. Generic variance, functions, conditional types, and language-level subtyping remain outside the current model.

* Decision: Type knowledge is exposed through read-only ASM queries rather than backend/parser-specific lookup paths.
* Reason: IDE, language services, inspection, and later optimization can consume the same canonical type facts.
* Tradeoff: C30 exposes direct queries only. CLI inspection output, richer type-declaration navigation, and composite predicates remain later work.

* Decision: Type inspection uses compiler-owned stable type text and attaches assignment provenance, status, and origin to ASM entities.
* Reason: Tooling can explain canonical type facts without decoding Rust debug output or re-deriving inference attribution from parser metadata.
* Tradeoff: C31 exposes assignment-backed entity types only. It does not add a standalone type-declaration browser, alias navigation UI, or source-summary type inference outside entity-scoped ASM inspection.

* Decision: Type diagnostics use an exported compiler-owned code/family vocabulary, while existing detailed codes remain stable.
* Reason: Diagnostics, IDE tooling, and future type consumers can categorize semantic failures without relying on message text or scattered string literals.
* Tradeoff: C32 reports unresolved declared state types now. Non-serializable-state has a reserved stable family but awaits source-level resource/boundary declarations before it can be emitted meaningfully.

* Decision: Type-system integrity is enforced by the existing ASM validator using deterministic `PSASM1101` through `PSASM1106` diagnostics.
* Reason: Type consumers fail early on corrupted canonical identities, ownership, provenance, or alias origins instead of relying on unchecked map contents.
* Tradeoff: Semantic types are value-owned, so recursive type cycles are unrepresentable; unresolved aliases are reported at lowering as `PSC1032`, while the validator checks the resulting model's origins and identities.

* Decision: Browser runtime integration tests acquire one process-wide lock before creating a Chrome probe, and the probe deadline allows 20 seconds for a cold Chrome start.
* Reason: Cargo's workspace runner schedules tests concurrently, and the previous five-second harness deadline could kill an otherwise healthy cold-start probe with SIGKILL.
* Tradeoff: The browser test binary runs serially under both workspace and dedicated e2e commands; independent non-browser tests remain parallel, while an actually hung probe still has a bounded deadline.

* Decision: Constant evaluation is an idempotent immutable transformation from raw ASM to a newly constructed folded ASM, rather than a side effect of parser or component-graph lowering.
* Reason: Compiler services and backend products now share one canonical evaluated result while retaining authored expression trees and preserving a read-only input model for optimization.
* Tradeoff: B9 folds only existing supported state initializer expressions and refreshes their template values. General expression graph nodes, local-expression folding, action expressions, runtime evaluation, and type propagation remain later work.

* Decision: Method locals receive stable method-child semantic IDs and resolve only from normal `render()` template scope through `template-local` references.
* Reason: Tooling and later optimizations need canonical declaration and use edges, while lexical resolution must not guess across list callback scopes, duplicate declarations, member access, calls, or closures.
* Tradeoff: Exact, uniquely declared identifiers are the only resolved form. Their known serializable values may seed static output, but runtime binding evaluation and all broader JavaScript scope behavior remain intentionally absent.

* Decision: Constrained method parameters are method-owned canonical metadata rather than runtime slots or standalone semantic entities.
* Reason: Parameter declarations need deterministic ownership and provenance for compiler services now, while B7 deliberately establishes no execution, closure, or binding-resolution behavior.
* Tradeoff: Only direct identifier declarations are retained in authored order; destructuring, defaults, rest parameters, captured values, render bindings, action values, and type semantics remain absent until an explicit later slice.

* Decision: Template descendants are lowered into canonical semantic entities separate from backend-local `n*` template IDs.
* Reason: Developer tools and compiler analyses need typed, owned, provenance-backed template semantics without taking a dependency on DOM emission details.
* Tradeoff: Template entity paths are deterministic traversal paths, while generated HTML/template-manifest contracts retain their existing local anchor IDs.

* Decision: Only direct `this.<stateField>` template reads resolve to `TemplateState` references in C3-B.
* Reason: The ASM gains reliable dependency edges for supported template behavior without introducing a general expression evaluator or speculative partial references.
* Tradeoff: Member access, computed expressions, and unresolved field names remain absent from the relation graph; keyed-list iterables are resolved by the dedicated C3-C extension.

* Decision: Keyed-list iterable dependencies reuse `TemplateState` with the list semantic entity as their source.
* Reason: A list's iterable is a direct template state read, so it has the same component ownership, provenance, and invalidation semantics as other direct template reads.
* Tradeoff: Item/index scope, keys, item members, and nested list-item expressions remain outside the component-state dependency model.

* Decision: Template event attributes reuse `EventMethod` with the canonical event-attribute entity as their source.
* Reason: The existing render-handler edge remains for backend compatibility, while ASM consumers can now trace an event directly from its authored template entity to the resolved method.
* Tradeoff: Both legacy render-handler and canonical template-event sources point at the same method until the backend-facing graph is migrated.

* Decision: `psc asm --format json` owns an explicit schema-versioned inspection document rather than serializing compiler structs directly.
* Reason: CLI consumers need a stable, deterministic interface that can evolve independently of Rust data-layout changes.
* Tradeoff: The document exposes generic entity kinds, owners, provenance, relations, and diagnostics, not every compiler-internal field or backend artifact.

* Decision: `psc asm` accepts explicit source paths and constructs a `CompilationUnit` in compiler path order.
* Reason: Multi-file semantic inspection must share the compiler's application input boundary rather than independently aggregating file-local outputs.
* Tradeoff: The command does not discover project files. Multi-file JSON retains the C4-A primary `file` field and adds an ordered `files` field only when more than one input is supplied.

* Decision: Parser summaries retain type annotations only when a property is recognized as a `state(...)` declaration.
* Reason: Explicit state types are the first type-semantic input needed by the canonical model, without expanding this slice to general TypeScript declaration analysis.
* Tradeoff: Annotation text is captured verbatim apart from outer whitespace and the leading colon; no inference, imports, compatibility checks, or non-state property types are modeled yet.

* Decision: Declared state types are attached to `StateField` as metadata with independent source provenance.
* Reason: The canonical component and ASM models can expose the actual authored type and its source location without treating a type annotation as a separate executable semantic entity.
* Tradeoff: Declared type metadata is descriptive only; it creates no type references, diagnostics, runtime behavior, or compatibility requirements.

* Decision: ASM JSON exposes declared type data only on state entities that actually have a declaration.
* Reason: The optional field extends the stable inspection document without changing the representation of existing untyped programs.
* Tradeoff: The document reports raw declared text and provenance; it does not classify, resolve, or validate the type expression.

* Decision: Primitive declared type classification recognizes only exact `string`, `number`, `boolean`, and `null` text.
* Reason: The compiler gains a reliable first typed vocabulary without silently interpreting unions, aliases, generics, literals, or imported names.
* Tradeoff: Any other valid TypeScript type remains available as raw declared text but has no classification or checking semantics yet.

* Decision: ASM JSON serializes an optional primitive `declared_type.kind` directly from canonical declared-state metadata.
* Reason: Inspection consumers receive a stable, reliable classification field without duplicating TypeScript interpretation in the CLI or inventing an `unknown` category.
* Tradeoff: Unclassified declarations omit `kind`; this remains descriptive metadata and has no validation, runtime, manifest, or backend effect.

* Decision: Primitive initializer validation compares only exact recognized declared types with statically known primitive initializer values.
* Reason: The canonical compiler can provide immediately reliable diagnostics from authored source without implying general TypeScript assignment compatibility or runtime flow analysis.
* Tradeoff: Unclassified declarations, arrays, objects, missing values, action updates, inferred types, aliases, imports, and unions do not produce type diagnostics in this slice.

* Decision: Compiler diagnostics carry optional provenance, and `PSC1016` locates the declared type annotation that establishes the incompatible contract.
* Reason: Developer tools can navigate from a semantic diagnostic to authoritative authored source while legacy diagnostics remain compatible when no reliable location exists.
* Tradeoff: Only primitive initializer mismatches populate diagnostic provenance in this slice; other compiler and ASM validation diagnostics intentionally omit the optional JSON field.

* Decision: Direct literal action assignments use a distinct `PSC1017` diagnostic and the parser's action-expression span.
* Reason: Initializer and action failures have different authored causes, so tooling can explain the operation precisely without collapsing them into one generic type mismatch.
* Tradeoff: Only `this.<field> = <primitive literal>` participates; increments, compound assignments, toggles, composite literals, variable flow, and unclassified types remain unvalidated.

* Decision: Boolean toggle validation uses distinct `PSC1018` diagnostics for exact primitive declared fields that are not `boolean`.
* Reason: The recognized toggle action has a fixed boolean result, so the compiler can validate it without interpreting arbitrary expressions.
* Tradeoff: Only the exact self-toggle form participates; numeric operators, compound assignments, variable flow, and unclassified types remain unvalidated.

* Decision: Increment and decrement validation uses distinct `PSC1019` diagnostics for exact primitive declared fields that are not `number`.
* Reason: The recognized numeric update operations have fixed numeric requirements, so the compiler can validate their targets without evaluating application state.
* Tradeoff: Compound arithmetic operands, arbitrary expressions, variable flow, and unclassified types remain unvalidated.

* Decision: Compound arithmetic uses `PSC1020` for non-number exact primitive targets and `PSC1021` for non-number literal operands.
* Reason: Target and operand failures are independent compiler facts, so separate diagnostics give tools actionable, source-provenanced evidence without expression evaluation.
* Tradeoff: Only serializable literal operands are classified; arbitrary expressions, variable flow, and unclassified declarations remain outside the type system.

* Decision: Text ASM inspection appends deterministic compiler diagnostic details only when compiler diagnostics exist.
* Reason: Command-line users can see the same canonical source evidence as JSON consumers without changing successful zero-diagnostic output.
* Tradeoff: ASM validation diagnostics are still count-only in text output; JSON remains the complete machine-readable inspection interface.

* Decision: Text ASM inspection appends deterministic ASM validation diagnostic details only when validation failures exist.
* Reason: Inspectors can now see every diagnostic class in the text surface while normal source compilation stays compact and compatibility-safe.
* Tradeoff: Standard source-driven `psc asm` inputs generally have no ASM validation failures; direct formatter coverage exercises this defensive contract without inventing invalid CLI inputs.

* Decision: Canonical ASM/frontend consumers use module-qualified semantic IDs, while the existing backend-facing graph retains legacy component-scoped IDs until its runtime contracts are deliberately migrated.
* Reason: A canonical application model must distinguish semantically equivalent components from different modules, but the established HTML/template runtime protocol does not serialize these IDs and should not be changed implicitly.
* Tradeoff: Two identity entry points coexist temporarily: `build_component_graph_for_module` for canonical compiler products and `build_component_graph` for legacy backend compatibility.

* Decision: Relative re-exports are flattened with a bounded fixed-point pass over the compilation unit.
* Reason: It resolves named and export-all chains through the same `ModuleGraph` and `BindingTable` products without recursive parser work or order-dependent results.
* Tradeoff: External and namespace re-exports remain unresolved, and the current component-scoped semantic IDs still require the next module-qualified identity migration.

* Decision: C2-B resolves only local exports and imports whose module-graph target is a relative file in the current `CompilationUnit`.
* Reason: Compiler bindings must use the same source/module/symbol products as the rest of `psco`, while package resolution and re-export chains require additional well-defined frontend semantics.
* Tradeoff: External packages remain unbound, and `export { ... } from` / `export * from` chains remain C2-C work.

* Decision: C2-A indexes only declarations local to each source module using class-qualified member names.
* Reason: Local declaration identity must be stable before imports, exports, aliases, and package resolution can form cross-module bindings.
* Tradeoff: Imports and export aliases are intentionally absent from `SymbolTable`; resolving them is the next C2-B slice.

* Decision: Module edges are derived from parsed import and re-export declarations in `CompilationUnit`, not from component provenance.
* Reason: File relationships are frontend semantics that must exist before symbol resolution and must be shared by all ASM consumers.
* Tradeoff: C1-B resolves only relative source files already present in the unit. Package resolution, tsconfig aliases, extension remapping, and symbol bindings remain C2 or later frontend work.

* Decision: The compiler frontend now accepts a deterministic `CompilationUnit` before application-level semantic construction.
* Reason: `psco` needs a project-wide input boundary so every later graph, analysis, plan, and developer product can consume one semantic model rather than independently reparsing files.
* Tradeoff: C1-A aggregates existing file-local semantics only. Import/export declarations, resolved module edges, duplicate semantic identity diagnostics, and symbols remain later compiler-front-end work.

* Decision: Existing route, module, layout, and resume metadata remain experimental compiler consumers rather than evidence that application-platform semantics are complete.
* Reason: The revised roadmap prioritizes real multi-file frontend and symbol resolution before adding further platform graphs.
* Tradeoff: Era V-D and adjacent application-platform slices are deferred until the canonical ASM has the necessary compiler foundations.

* Decision: Every completed slice updates both this handoff and the active weekly progress log before its checkpoint is finalized.
* Reason: The handoff preserves immediate continuation context while the progress log preserves the durable implementation chronology.
* Tradeoff: Documentation-only recovery commits may be required when a prior checkpoint omitted the weekly entry.

* Decision: `psc check` defaults parser failures to `error` and keeps that policy command-scoped until a project configuration format is deliberately designed.
* Reason: The compiler can establish a predictable default without implying that an undocumented configuration file is accepted or that compiler/ASM integrity findings are suppressible.
* Tradeoff: Teams must pass a policy threshold in their command invocation; project presets and policy-file discovery remain future work.

* Decision: Browser e2e recipe entry points run with one Rust test thread.
* Reason: Each test launches a real Chrome process, and serial execution prevents host-resource contention and stale profile locks during the documented verification commands.
* Tradeoff: The browser suite takes longer to run, but `pnpm test:e2e` and `just e2e` now produce a reproducible result on constrained development hosts.

* Decision: `psc check` projects parser label provenance as a deterministic array of source coordinates.
* Reason: CLI and automation consumers can navigate from a parser diagnostic to every parser-provided span without reparsing the source or depending on backend-specific diagnostics.
* Tradeoff: Labels currently provide only positional spans. Label messages, source excerpts, code frames, and compiler/ASM provenance in check JSON remain separate follow-up work.

* Decision: `psc check --format json` reuses the ASM source-provenance shape for compiler diagnostics and omits it when unavailable.
* Reason: Check consumers receive the same canonical coordinates as ASM inspection without representing missing provenance as an invented location or a misleading null contract.
* Tradeoff: Only diagnostics with reliable compiler provenance include the field. Source remapping, code frames, and provenance for ASM validation diagnostics remain future work.

* Decision: ASM ownership traversal exposes application roots and direct children as semantic IDs ordered by the canonical ownership map.
* Reason: Tooling can navigate the compiler-owned hierarchy without rebuilding ownership from public fields or depending on source declaration order.
* Tradeoff: C7-A is intentionally direct-only; transitive traversal, entity-kind filters, and source-to-semantic lookup remain follow-up query capabilities.

* Decision: Transitive ASM ownership traversal uses depth-first pre-order.
* Reason: Consumers encounter each semantic parent before its complete owned subtree while every sibling order remains the canonical semantic-ID order.
* Tradeoff: The query returns IDs only and does not encode depth, paths, filters, or source lookup results.

* Decision: ASM entity filtering uses a typed `SemanticEntityKind` enum and ownership-map ordering.
* Reason: Tooling can request stable semantic categories without stringly typed kinds or independent scans of graph-specific collections.
* Tradeoff: C7-C filters only the broad canonical entity categories; template subkinds, composite predicates, and source-location predicates remain follow-up work.

* Decision: ASM source lookup uses exact provenance paths and half-open byte spans.
* Reason: The compiler preserves source coordinates in this form, so tooling can map source selections to all overlapping semantic entities without inventing line/column conversions or boundary ambiguity.
* Tradeoff: Path normalization, line/column inputs, source remapping, nearest-entity ranking, and editor-range protocols remain future work.

* Decision: ASM reference-kind queries order results by source and target semantic IDs.
* Reason: Tooling receives a stable relation filter independent of construction order.
* Tradeoff: C7-E filters a single reference kind only; endpoint, provenance, and composite relation queries remain follow-up work.

* Decision: ASM reference provenance lookup reuses exact paths and half-open byte spans, with source/target ordering.
* Reason: Tooling can map a source selection to every canonical relation without reparsing source or relying on relation construction order.
* Tradeoff: C7-F does not provide line/column, path normalization, source remapping, composite predicates, or CLI query syntax.

* Decision: Entity inspection is an `asm --entity` extension, not a change to the legacy `explain` command.
* Reason: It exposes the canonical semantic model through an existing inspection surface without breaking the established source-summary explain contract.
* Tradeoff: Users must first obtain a semantic ID from full ASM output; source-position selection and migration of `explain` remain follow-up work.

* Decision: Source-selected inspection chooses the uniquely narrowest overlapping semantic span.
* Reason: Source selection is useful for nested semantic entities while remaining deterministic and refusing equal-specificity ambiguity rather than silently guessing.
* Tradeoff: The CLI accepts exact compiler paths and byte offsets only; line/column input, path normalization, source remapping, and user-selectable candidate lists remain future work.

* Decision: Entity inspection filters use typed broad entity and reference kinds, and only operate after a semantic entity has been selected.
* Reason: The CLI can reuse canonical ASM categories while making the result boundary unambiguous: direct child lists and incoming/outgoing relation lists are filtered without changing the selected entity or its ownership traversal.
* Tradeoff: C8-C accepts one child-kind and one reference-kind filter. Composite predicates, descendant filtering, diagnostics filtering, line/column selection, and path normalization remain future work.

* Decision: `psc explain` delegates entity inspection to the same canonical ASM inspection runner as `psc asm`.
* Reason: The developer-facing source-summary command can expose compiler semantics without duplicating selection, ordering, filtering, diagnostic, or schema behavior.
* Tradeoff: Plain `explain` remains a legacy source-summary surface; only explicit entity-selection or entity-filter options activate ASM inspection.

* Decision: Parent navigation is a canonical ASM query and reports the ancestor chain nearest-first through the application root.
* Reason: Semantic tooling can traverse ownership outward from any entity without reconstructing containment from entity-local fields or reversing the ownership map.
* Tradeoff: Parent navigation reports semantic IDs only. It does not add path metadata, transitive child filtering, reference endpoint predicates, or model mutation.

* Decision: Semantic graph export uses a roots collection plus parent-to-child ownership edges instead of a synthetic application node.
* Reason: The export preserves the ASM's distinction between real semantic entities and application ownership while still providing complete deterministic graph topology.
* Tradeoff: The graph is a JSON artifact for canonical roots, typed nodes, provenance, ownership, and resolved references only; diagnostics, parser facts, backend-local node IDs, manifests, runtime artifacts, and graph mutation are outside this slice.

* Decision: Canonical ASM ownership is structurally derived for component-level semantics and consumed through the centralized ownership map.
* Reason: Compiler analyses and semantic lowering should not silently depend on duplicate owner fields that can drift from the canonical application model.
* Tradeoff: Legacy ComponentGraph and template-entity lowering records still retain owner fields for compatibility and initial template containment ingestion; symbol-table and backend paths remain outside this ownership migration.

* Decision: Compiler analyses implement `ImmutableAsmPass`, while `AnalysisPass::analyze` remains a compatibility wrapper.
* Reason: New compiler work receives one explicit read-only ASM transformation boundary without forcing a breaking migration on existing analysis consumers.
* Tradeoff: Current passes produce immutable analysis products rather than rewritten ASM values. Semantic rewrite passes will use the same contract when a compiler-owned language transformation requires them.

* Decision: Constant numeric state initializer arithmetic is a compiler-owned expression model that evaluates during canonical lowering.
* Reason: Authored numeric semantics remain inspectable in the compiler while established HTML, manifest, and runtime paths receive an already-computed serializable initial value.
* Tradeoff: B1 accepts only numeric literals, parentheses, and `+`, `-`, `*`, `/`, or `%` inside `state(...)`. State reads, local variables, calls, coercions, action expressions, comparisons, and expression typing remain later language slices.

* Decision: Arithmetic and comparison initializers share one canonical `ConstantExpression` slot on a state field.
* Reason: The compiler can extend expression semantics without parallel per-operator metadata, while preserving one inspectable authored expression and one evaluated initial value for every backend.
* Tradeoff: B2 comparisons are numeric-only and static: operands are numeric literals or B1 arithmetic, and supported operators are `===`, `!==`, `<`, `<=`, `>`, and `>=`. String/boolean comparisons, coercion, state reads, calls, local variables, logical operators, and action expressions remain unsupported.

* Decision: Constant logical expressions use explicit boolean operands and compiler-time short-circuit evaluation.
* Reason: The compiler can statically preserve `&&`/`||` reachability and avoid emitting runtime expression intelligence or diagnostics from unreachable branches.
* Tradeoff: B3 accepts only boolean literals, B2 comparisons, and nested logical expressions. Unary negation, truthiness, coercion, state reads, local variables, calls, nullish coalescing, and action expressions remain unsupported.

* Decision: Conditional nodes are first-class parser/render/template children with a conditional node ID plus separate start/end boundary IDs.
* Reason: The compiler needs stable branch identity for tooling and runtime updates, while the DOM needs comment anchors that can bound branch replacement without a wrapper element.
* Tradeoff: Runtime manifests serialize branch HTML snippets for this first slice instead of recursively hydrating dynamic bindings/events inside branch snippets.
* Decision: Logical-and shorthand reuses the same conditional model with an empty false branch.
* Reason: It keeps ternary and shorthand update behavior identical once the branch anchor path is stable.
* Decision: Keyed list nodes are represented before arrays, static list HTML, manifest serialization, or runtime reconciliation.
* Reason: The compiler needs a stable semantic contract for list identity before committing to array values or DOM update behavior.
* Tradeoff: List item templates are visible in parser/component/template tooling, but they intentionally emit no initial DOM content or runtime manifest records yet.
* Decision: Serializable arrays may contain serializable primitives or nested arrays, but static list item binding resolution is limited to the exact item variable and optional index variable.
* Reason: It enables initial list rendering without introducing object-value semantics or arbitrary expression evaluation.
* Tradeoff: Object entries and member expressions such as `item.id` remain semantic metadata only; their initial values are not rendered in 7F-B.
* Decision: A list manifest owns a start/end anchor pair, the iterable state dependency, item/key variables, an item-root template ID, and placeholder HTML for a new root.
* Reason: The runtime can build a key-to-element index from static HTML and reconcile roots without an application-specific virtual DOM.
* Tradeoff: Runtime reconciliation is intentionally constrained to one root element per item and local item/index text substitution; nested dynamic behavior inside list items is not hydrated yet.
* Decision: Before member-expression evaluation, keyed lists were constrained to their direct item variable as the key.
* Reason: The initial runtime could prove direct primitive key identity while object/member access semantics were isolated into a dedicated follow-on slice.
* Tradeoff: That temporary constraint delayed `item.id` support until 7F-F established the compiler evaluator and 7F-G mirrored it in the browser runtime.
* Decision: Serializable object values use ordered maps with static identifier, string, or numeric keys.
* Reason: The compiler needs recursively traversable data that serializes to deterministic JSON objects in manifests without introducing a new dependency or arbitrary expression evaluation.
* Tradeoff: Spreads, computed keys, shorthand references, methods, accessors, and non-literal property values remain unsupported until later language slices define their semantics.
* Decision: Static list member evaluation accepts non-empty dot paths rooted at the list item variable.
* Reason: A small, compiler-owned evaluator can resolve object members for static HTML, member-derived IDs, and diagnostics without becoming a general JavaScript interpreter.
* Tradeoff: The compiler evaluator covers static rendering and diagnostics; 7F-G mirrors it only for runtime keys and insertion-time text, leaving retained item refresh to 7F-H.
* Decision: The browser runtime mirrors the compiler's dot-member semantics for list keys and insertion-time text bindings.
* Reason: Initial compiler IDs, manifest templates, and future object list assignments need one consistent object path contract to retain roots and materialize new rows correctly.
* Tradeoff: Runtime member evaluation is intentionally limited to list keys and text binding comments. It does not yet refresh retained item content, attributes, events, or nested dynamic behavior.
* Decision: Scoped list bindings emit a compiler-owned end anchor after each binding comment, plus element-local metadata for dynamic item attributes.
* Reason: A retained item can refresh text without swallowing adjacent static punctuation, and attribute updates can reuse the runtime's existing attribute semantics without adding a client-side expression evaluator.
* Tradeoff: The metadata accepts only the direct item/index variables and non-empty dot-member paths; arbitrary expressions remain unsupported.
* Decision: List-item click actions are discovered from retained or inserted DOM roots and registered in the delegated event map; removed roots explicitly unregister their nodes.
* Reason: Keyed root identity remains stable while events work for both compiler-rendered and runtime-inserted rows.
* Tradeoff: Only the existing delegated click/action contract is hydrated. Conditional branch replacement inside a list item still does not register new dynamic behavior.
* Decision: Semantic IDs are typed compiler values with readable component-scoped paths, such as `component:x-counter/state:count`.
* Reason: Existing template IDs are local backend anchors. The ASM needs stable identities that survive unrelated declaration ordering and can be shared by future semantic consumers.
* Tradeoff: Component identity uses `@component(...)` when available and falls back to the class name for invalid components. Duplicate identity validation and source provenance are deferred to later ASM slices.
* Decision: Actions use their source order within the owning method as their final semantic-ID segment.
* Reason: The parser does not yet retain per-action source spans, but ordered action steps are already a compiler contract.
* Tradeoff: Inserting an earlier action step changes later action IDs; source-provenance-backed refinement remains deferred to ASM-4.
* Decision: Semantic IDs do not change template manifests, static HTML, or runtime artifacts in ASM-1.
* Reason: This establishes a compiler-platform contract without forcing a backend schema change before ownership and cross-reference semantics exist.
* Tradeoff: Semantic IDs are inspectable through compiler APIs only until the planned `presolve explain` CLI slice.
* Decision: Ownership is a typed relationship with either the application root or a direct owning `SemanticId`.
* Reason: The compiler can distinguish top-level component roots from semantic children without reserving a synthetic application ID before the Application Semantic Model exists.
* Tradeoff: Ownership is currently stored on existing graph entities rather than in a centralized ASM relation table; the query API and ASM shell will consolidate access later.
* Decision: Component state, methods, and rendered templates are component-owned; action steps are method-owned.
* Reason: These are the direct lexical/semantic containment relationships already established by the current frontend and action model.
* Tradeoff: Render-tree descendants do not yet have semantic IDs, so their ownership remains deferred until a later ASM slice extends semantic identity below the template root.
* Decision: Cross references are directed resolved edges stored on `ComponentGraph`, with a kind, source semantic ID, and target semantic ID.
* Reason: Existing graph consumers can access relationships without an ASM shell, while later ASM/query slices can migrate this stable relation shape intact.
* Tradeoff: There is no reverse index or centralized relation table yet; query-oriented traversal remains deferred to ASM-6.
* Decision: Event handlers use deterministic component-scoped IDs in render traversal order and are owned by the rendered template.
* Reason: Event-to-method references need a distinct semantic source even though general render descendants still lack semantic identity.
* Tradeoff: Inserting an earlier event changes later event IDs; source-provenance-backed refinement remains deferred to ASM-4.
* Decision: Only fully resolved action/state and event/method pairs create semantic references.
* Reason: Reference consumers can rely on every target ID existing in the graph, while existing diagnostics remain the source of unresolved-reference feedback.
* Tradeoff: Unresolved attempts are not retained as partial relation records until diagnostics and provenance gain their planned ASM models.
* Decision: Source provenance is stored once in a `SemanticId`-keyed registry on `ComponentGraph`.
* Reason: Every current semantic consumer shares one authoritative path/span record instead of duplicating source fields across graph structures.
* Tradeoff: The registry is not yet a cross-file application index; the Application Semantic Model shell will own aggregation later.
* Decision: State update spans originate in the parser from update/assignment expressions.
* Reason: Action semantic provenance must identify the actual operation rather than approximating it from the enclosing method.
* Tradeoff: The span excludes the expression statement semicolon, matching the AST operation span used by the compiler.
* Decision: Resolved references carry the provenance of their source entity.
* Reason: Tooling and diagnostics can trace an edge to the authored action or event handler without requiring a reverse lookup first.
* Tradeoff: Target-side provenance remains available through the semantic provenance registry; relations intentionally store only their own origin.
* Decision: `ApplicationSemanticModel` assembles copies of the existing graph outputs instead of replacing backend-facing graph builders immediately.
* Reason: Existing HTML, manifest, and runtime paths keep their stable contracts while all future compiler consumers gain one application-level semantic entry point.
* Tradeoff: Graph data is temporarily duplicated at assembly time; later backend migration and query slices can eliminate redundant construction when the API is mature.
* Decision: ASM ownership is centralized as a `SemanticId` to `SemanticOwner` map while entity-local ownership fields remain available.
* Reason: The model provides the first global ownership view without forcing all existing consumers through a new lookup API in this slice.
* Tradeoff: Both representations coexist until the query API and backend migration establish a single consumption path.

Known limitations

* Item: Phase I is complete through I6. Canonical Form declarations, Fields, control bindings, declaration ownership, retained validation candidates, valid Validation Rules, direct same-Form dependency facts, duplicate/contradiction/cycle exclusion, and the immutable validation graph exist. Cross-Field invalidation/scheduling, validation execution/state/messages, tracking, submission, serialization/reset plans, IR, runtime products, public inspection/diagnostics, fixtures, and resumability planning do not.
* Item: I7 cannot be implemented from the roadmap's one-line cross-Field dependency-planning entry without inventing invalidation propagation, dependency scheduling, update triggers, derived plan identities/products, ordering, execution boundaries, runtime state, or schema policy. I6's direct dependency edges and cycle facts are immutable inputs only.
* Item: Phase H is frozen through H21. Semantic graph v5 intentionally omits Phase H entities, live component restoration remains deferred until Phase J, and every unsupported component behavior in `docs/component-contract.md` requires a later authoritative roadmap slice.
* Item: Conditional rendering only supports simple `this.<stateField>` conditions with JSX element or fragment branches.
* Item: Conditional branch snippets are replaced as static HTML. Bindings, events, and nested dynamic behavior inside swapped-in branch snippets are not re-registered yet.
* Item: Keyed lists currently accept only `iterable.map((item, index?) => <element>...</element>)` with identifier parameters and an expression-bodied callback. Static and runtime reconciliation support a direct primitive item key or a dot-member key such as `item.id` that resolves to a unique primitive.
* Item: Missing keys, index keys, unsupported expressions, duplicate statically-known primitive keys, and missing/non-primitive member keys emit `PSC1011` through `PSC1015`.
* Item: List item templates must have one root element. Retained and inserted items refresh direct item/index/member text and direct item/index/member attributes, and hydrate delegated click actions. Nested dynamic behavior still does not refresh or re-register.
* Item: Duplicate runtime list keys that arise from dynamic state still produce `PSR_DUPLICATE_LIST_KEY` and the later duplicate is skipped. A missing/non-primitive dynamic member key falls back to index identity; compiler diagnostics cover only statically-known initial items.
* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, binding callback text/attribute updates, and conditional branch replacement.
* Item: Static and `this.<stateField>` dynamic JSX attributes are preserved, but `className`/`htmlFor` normalization policy is still intentionally undecided.
* Item: Dynamic attributes are limited to primitive state-field bindings; arbitrary expressions, method calls, spread attributes, arrays, and objects are not emitted yet.
* Item: The serializable value model supports primitives, recursive arrays, and recursive object literals. Static and runtime list paths resolve direct item/index values and non-empty item-member paths; arbitrary JavaScript expressions remain unsupported.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Source spans are available on parser/render/template structures and CLI development output, but runtime manifests intentionally omit source metadata for now.
* Item: Fragment nodes are visible in compiler/template output but intentionally omitted from runtime manifests until a runtime range-anchor use case appears.
* Item: Semantic IDs, direct ownership, and provenance cover components, state fields, methods, action steps, rendered templates, event handlers, and authored template descendants. Backend HTML/template-manifest nodes still use local `n*` IDs as a compatibility contract.
* Item: Resolved references cover action-to-state, event-to-method, and exact direct text-binding/dynamic-attribute/conditional/keyed-list-iterable pairs to state or computed entities. Routes, member expressions, calls, computed evaluation, and unresolved reference attempts have no semantic relation records yet.
* Item: Canonical compiler products now include module-qualified template entities, direct template state dependencies, and direct template event-method dependencies, while `BindingTable` resolves local/relative re-export chains plus named/default/namespace imports. External and namespace re-exports, external package bindings, tsconfig aliases, source remapping, and type semantics are still absent. Legacy backend graph identity remains a compatibility path.
* Item: `psc asm` accepts explicit source files and exposes generic JSON and text inspection. Text includes compiler and ASM validation diagnostic detail when present. Project discovery, tsconfig resolution, source remapping, typed action payloads, and machine-readable backend plans remain future slices.
* Item: Declared state types include canonical primitive classification, optional ASM JSON `declared_type.kind`, and source-provenanced `PSC1016` through `PSC1021` diagnostics for supported initializer and action forms. Other compiler/ASM diagnostics may omit provenance. Arbitrary action expressions, variable flow, manifests, runtime, imported types, non-state annotations, inference, unions, aliases, generics, and general assignment compatibility remain outside current type validation.
* Item: Browser e2e requires a local Chrome binary or `PRESOLVE_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.
* Item: Check policy is selected per CLI invocation. Project policy files, presets, and policy discovery are not interpreted yet.
* Item: Parser diagnostic labels expose only `line`, `column`, `start`, and `end`; parser label messages and rendered source excerpts are not available yet. Compiler provenance in check JSON is optional, and ASM validation diagnostics still have no provenance field.
* Item: ASM query APIs expose nearest-first parent traversal through the application root, direct and transitive ownership traversal, broad entity kinds, entity/reference provenance lookup, and reference-kind filtering. `asm` and explicit `explain` inspection mode support semantic-ID or source-byte selection plus parent, direct-child, and incoming/outgoing reference navigation, with one typed child and relation filter; composite predicates, descendant/diagnostic filtering, line/column input, path normalization, and source remapping remain future work.
* Item: `psc asm --format graph` exports a schema-versioned canonical semantic graph with roots, typed nodes, provenance, ownership edges, and resolved reference edges. It intentionally does not discover project files, include diagnostics, expose parser/backend/runtime artifacts, or provide graph filtering, mutation, or alternate serialization formats.
* Item: Canonical ASM ownership now drives template entity lookup, template dependency lowering, and dead-action analysis. Legacy ComponentGraph, TemplateSemanticEntity construction, and SymbolTable records still carry owner fields as compatibility/lowering data; their removal or migration requires a later dedicated frontend/backend compatibility slice.
* Item: Constant `state(...)` initializers use one compiler-owned expression model. Numeric arithmetic, comparisons, boolean logic, nullish coalescing, and unary `!`, `+`, and `-` evaluate statically. State reads, local variables, calls, coercions, truthiness, control flow, and semantic expression typing remain later Phase B work.
* Item: Method parameters are compiler-owned identifier declarations with canonical source provenance only. They do not execute, close over values, resolve local/template/action references, or support destructuring, defaults, rest declarations, or semantic type checking.
* Item: Method-local resolution accepts only exact, uniquely declared supported locals from `render()` template scope. List-item scopes, duplicate local names, member access, arbitrary expressions, calls, closures, action references, runtime updates, and semantic typing remain unresolved.
* Item: Constant folding handles only the existing supported state initializer expression language. It does not fold local-variable values, evaluate actions or templates generically, perform flow/type analysis, or introduce runtime evaluation.
* Item: The canonical expression graph covers supported state initializer expressions and direct supported computed getter returns. Computed expression nodes and entities have inferred canonical types through resolved state/computed reads; exact template binding uses can resolve to computed entities, but do not evaluate them. Computed values are pure or impure with `PSC1034` purity diagnostics, compiler-owned direct/transitive reactive topology is available, computed cycles emit `PSC1035`, and evaluation plans expose stable order/batches. Pure scheduled E2 expressions lower to canonical IR functions, a separate immutable optimized IR product, and emitted `computed.runtime.json` programs. Runtime state writes mark compiler-emitted transitive dependents dirty and one post-action scheduler flush refreshes caches. Resume plans now identify serializable computed caches, but no live cache snapshot or restore transport exists yet; template updates remain later work.
* Item: C30 exposes direct ASM type queries only. CLI inspection output, source diagnostics, backend enforcement, resource declaration lowering, and final type diagnostic families remain later Phase C work.
* Item: Authored method IR functions still contain only empty entry basic blocks plus structural branch-edge and natural-loop records. Computed E10 functions lower the supported E2 expression graph into value-producing instructions, but neither form has source-lowered branches/loops, explicit terminators, or general statement instructions.
* Item: Authored method lowering still creates empty function value registries and no method load/store instructions. Computed E10 functions register their defining values and resolved state/computed loads; D3-A analyses apply to both canonical forms without adding general source statement lowering.

Exact next step

Phase K is complete and frozen through K21. Phase L is complete through
L17-B, but L18 is paused for the owner-directed Presolve identity migration.
The active migration contract supersedes the retained-identity exceptions:
all active compiler/runtime namespaces, diagnostics, generated marker names,
fixtures, and implementation paths must move together. The first migration
slice makes `presolve explain` the sole inspection command (`--inspect` for
complete inspection; the retired short command exits 6). Compiler/runtime
diagnostics, generated marker names, fixture bytes, browser globals, browser
assertions, crate paths, imports, and implementation diagnostic families now use
the Presolve namespace. Next: run the full identity-migration audit and resume
L18 only after every active legacy representation is absent.

Useful commands

* `cargo fmt --all --check`
* `cargo test -p presolve_parser`
* `cargo test -p presolve_compiler`
* `cargo test -p presolve_cli`
* `RUST_TEST_THREADS=1 cargo test -p presolve_cli --test runtime_browser -- --nocapture`
* `cargo clippy --workspace --all-targets -- -D warnings`
* `pnpm test:e2e`
* `just e2e`
* `RUST_TEST_THREADS=1 cargo test --workspace`
* `cargo run -p presolve_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/presolve-manual/double-binding-counter`
* `cargo run -p presolve_cli -- build fixtures/0009-decrement-counter/input/DecrementCounter.tsx --out target/presolve-manual/decrement-counter`
* `cargo run -p presolve_cli -- build fixtures/0010-add-subtract-assign/input/StepCounter.tsx --out target/presolve-manual/step-counter`
* `cargo run -p presolve_cli -- build fixtures/0011-direct-assignment/input/ResetCounter.tsx --out target/presolve-manual/reset-counter`
* `cargo run -p presolve_cli -- build fixtures/0012-boolean-toggle/input/ToggleFlag.tsx --out target/presolve-manual/toggle-flag`
* `cargo run -p presolve_cli -- build fixtures/0013-multi-step-action/input/BatchActionCounter.tsx --out target/presolve-manual/batch-action-counter`
* `cargo run -p presolve_cli -- build fixtures/0014-static-attributes/input/StaticAttributePanel.tsx --out target/presolve-manual/static-attributes`
* `cargo run -p presolve_cli -- build fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx --out target/presolve-manual/dynamic-attributes`
* `cargo run -p presolve_cli -- build fixtures/0016-fragments/input/FragmentPanel.tsx --out target/presolve-manual/fragments`
* `cargo run -p presolve_cli -- build fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx --out target/presolve-manual/conditional-rendering`
* `cargo run -p presolve_cli -- build fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx --out target/presolve-manual/logical-and-conditional`
* `cargo run -p presolve_cli -- template fixtures/0019-keyed-list-semantics/input/KeyedList.tsx`
* `cargo run -p presolve_cli -- html fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx`
* `RUST_TEST_THREADS=1 cargo test -p presolve_cli --test runtime_browser keyed_lists_reconcile_in_a_real_browser -- --nocapture`
* `cargo run -p presolve_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/presolve-manual/runtime-contract`

Changed but uncommitted files

* None after the L11-B strict-reader commit.
